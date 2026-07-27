//! Single request-admission path. No convenience method may dispatch a
//! request to a tool worker without going through [`admit`] first.

use crate::capability::HubCapability;
use crate::descriptor::{HubOperationDescriptor, HubToolDescriptor};
use crate::envelope::{HubRequest, MAX_PAYLOAD_BYTES};
use crate::fault::HubFault;
use crate::registry::ToolRegistry;
use crate::resource::HubResourceBudget;
use crate::worker::HubWorkerState;

/// Ambient state the admission path needs beyond the registry and request
/// itself: current queue occupancy and whether cancellation was already
/// requested for this request id before admission ran.
#[derive(Debug, Clone, Copy)]
pub struct AdmissionAmbient {
    pub current_queue_depth: u32,
    pub current_concurrent_requests: u32,
    pub already_cancelled: bool,
}

/// The result of successful admission: everything dispatch needs, resolved
/// once so the worker never has to re-derive trust decisions.
#[derive(Debug, Clone)]
pub struct AdmittedInvocation {
    pub tool: HubToolDescriptor,
    pub operation: HubOperationDescriptor,
}

fn check_capabilities(
    request_grants: &crate::capability::HubCapabilitySet,
    required: &std::collections::BTreeSet<HubCapability>,
) -> Result<(), HubFault> {
    let required: Vec<HubCapability> = required.iter().copied().collect();
    if request_grants.satisfies(&required) {
        Ok(())
    } else {
        let missing: Vec<&str> = required
            .iter()
            .filter(|c| c.is_sensitive() || !request_grants.allows(**c))
            .map(|c| c.as_str())
            .collect();
        Err(HubFault::CapabilityDenied(format!(
            "missing or denied capabilities: {}",
            missing.join(", ")
        )))
    }
}

/// Run the full admission path for one request against the current registry
/// state. On success, returns everything needed for dispatch; the caller
/// (the Hub runtime) must not call the adapter except with an
/// [`AdmittedInvocation`] produced here.
pub fn admit(
    registry: &ToolRegistry,
    request: &HubRequest,
    ambient: AdmissionAmbient,
) -> Result<AdmittedInvocation, HubFault> {
    // 1. Envelope-level checks that don't need the registry at all.
    if !crate::ids::HubApiVersion::CURRENT.is_compatible_with(request.api_version) {
        return Err(HubFault::ApiVersionUnsupported);
    }
    if request.schema_version != crate::envelope::HUB_ENVELOPE_SCHEMA_VERSION {
        return Err(HubFault::SchemaVersionUnsupported);
    }
    if request.payload.len() > MAX_PAYLOAD_BYTES {
        return Err(HubFault::InputRejected(format!(
            "payload {} bytes exceeds maximum {} bytes",
            request.payload.len(),
            MAX_PAYLOAD_BYTES
        )));
    }
    // The budget check below (step 5) only verifies the *requested budget
    // itself* does not exceed the tool/global ceiling -- it does not check
    // that the actual payload conforms to the caller's own declared
    // input_bytes. Without this, a caller narrowing input_bytes below
    // MAX_PAYLOAD_BYTES got no enforcement at all: any payload under the
    // global ceiling was silently dispatched regardless of what budget was
    // requested for it.
    if request.payload.len() as u64 > request.resource_budget.input_bytes {
        return Err(HubFault::InputRejected(format!(
            "payload {} bytes exceeds the request's own declared input_bytes budget {}",
            request.payload.len(),
            request.resource_budget.input_bytes
        )));
    }
    if ambient.already_cancelled {
        return Err(HubFault::Cancelled);
    }

    // 2. Registry lookup: unknown tool vs. unknown operation are distinct.
    let tool = registry
        .descriptor(&request.tool_id)
        .ok_or(HubFault::UnknownTool)?
        .clone();
    let operation = tool
        .operation(&request.operation_id)
        .ok_or(HubFault::UnknownOperation)?
        .clone();

    // 3. Worker health/lifecycle gate.
    match registry.worker_state(&request.tool_id) {
        Some(HubWorkerState::Disabled) => return Err(HubFault::ToolDisabled),
        Some(HubWorkerState::Quarantined) => return Err(HubFault::ToolQuarantined),
        Some(HubWorkerState::Stopped) => return Err(HubFault::ToolDisabled),
        _ => {}
    }

    // 4. Capability policy: deny-by-default, checked before dispatch.
    check_capabilities(
        &request.capability_context,
        &operation.required_capabilities,
    )?;

    // 5. Resource budget: immutable ceiling check, checked arithmetic only.
    if let Some(violation) = request
        .resource_budget
        .first_violation(&tool.resource_ceiling)
        .or_else(|| {
            request
                .resource_budget
                .first_violation(&HubResourceBudget::V0_CEILING)
        })
    {
        return Err(HubFault::ResourceBudgetInvalid(violation));
    }

    // 6. Queue/concurrency admission.
    if ambient.current_queue_depth >= request.resource_budget.queue_depth {
        return Err(HubFault::QueueFull);
    }
    if ambient.current_concurrent_requests >= request.resource_budget.concurrent_requests {
        return Err(HubFault::QueueFull);
    }

    Ok(AdmittedInvocation { tool, operation })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::HubCapabilitySet;
    use crate::descriptor::HubOperationDescriptor;
    use crate::execution::{HubDeterminismClass, HubExecutionMode, HubPrivacyClass, HubTrustClass};
    use crate::ids::{
        HubApiVersion, HubCallerIdentity, HubOperationId, HubRequestId, HubSessionId, HubToolId,
        HubToolVersion,
    };
    use crate::registry::ToolRegistry;

    fn base_ambient() -> AdmissionAmbient {
        AdmissionAmbient {
            current_queue_depth: 0,
            current_concurrent_requests: 0,
            already_cancelled: false,
        }
    }

    fn registry_with_turbovec() -> ToolRegistry {
        let mut reg = ToolRegistry::new(HubApiVersion::CURRENT);
        reg.register(HubToolDescriptor {
            tool_id: HubToolId::new("vector.turbovec").unwrap(),
            name: "TurboVec".into(),
            tool_version: HubToolVersion::new(0, 1, 0),
            hub_api_version: HubApiVersion::CURRENT,
            execution_mode: HubExecutionMode::InProcess,
            trust_class: HubTrustClass::InProcessUnisolated,
            operations: vec![HubOperationDescriptor::new(
                HubOperationId::new("vector.search").unwrap(),
                [HubCapability::VectorSearch],
                HubDeterminismClass::Unknown,
                false,
            )],
            resource_ceiling: HubResourceBudget::V0_CEILING,
            adapter_provenance: "test".into(),
        })
        .unwrap();
        reg
    }

    fn base_request(capabilities: HubCapabilitySet) -> HubRequest {
        HubRequest {
            schema_version: crate::envelope::HUB_ENVELOPE_SCHEMA_VERSION,
            api_version: HubApiVersion::CURRENT,
            request_id: HubRequestId::new("req-1").unwrap(),
            session_id: HubSessionId::new("sess-1").unwrap(),
            caller_identity: HubCallerIdentity::new("cli:local").unwrap(),
            tool_id: HubToolId::new("vector.turbovec").unwrap(),
            operation_id: HubOperationId::new("vector.search").unwrap(),
            capability_context: capabilities,
            privacy_class: HubPrivacyClass::ProjectLocal,
            resource_budget: HubResourceBudget::V0_CEILING,
            payload: b"{}".to_vec(),
        }
    }

    #[test]
    fn well_formed_request_with_granted_capability_is_admitted() {
        let reg = registry_with_turbovec();
        let req = base_request(HubCapabilitySet::empty().grant(HubCapability::VectorSearch));
        assert!(admit(&reg, &req, base_ambient()).is_ok());
    }

    #[test]
    fn unknown_tool_is_rejected_distinctly() {
        let reg = registry_with_turbovec();
        let mut req = base_request(HubCapabilitySet::empty().grant(HubCapability::VectorSearch));
        req.tool_id = HubToolId::new("solver.z3").unwrap();
        assert_eq!(
            admit(&reg, &req, base_ambient()).unwrap_err(),
            HubFault::UnknownTool
        );
    }

    #[test]
    fn unknown_operation_on_known_tool_is_rejected_distinctly() {
        let reg = registry_with_turbovec();
        let mut req = base_request(HubCapabilitySet::empty().grant(HubCapability::VectorSearch));
        req.operation_id = HubOperationId::new("vector.remove").unwrap();
        assert_eq!(
            admit(&reg, &req, base_ambient()).unwrap_err(),
            HubFault::UnknownOperation
        );
    }

    #[test]
    fn missing_capability_is_denied_before_dispatch() {
        let reg = registry_with_turbovec();
        let req = base_request(HubCapabilitySet::empty());
        assert!(matches!(
            admit(&reg, &req, base_ambient()).unwrap_err(),
            HubFault::CapabilityDenied(_)
        ));
    }

    #[test]
    fn oversized_payload_is_rejected_before_registry_lookup() {
        let reg = registry_with_turbovec();
        let mut req = base_request(HubCapabilitySet::empty().grant(HubCapability::VectorSearch));
        req.payload = vec![0u8; MAX_PAYLOAD_BYTES + 1];
        assert!(matches!(
            admit(&reg, &req, base_ambient()).unwrap_err(),
            HubFault::InputRejected(_)
        ));
    }

    #[test]
    fn payload_exceeding_the_requests_own_input_bytes_budget_is_rejected() {
        // Regression test: a caller narrowing resource_budget.input_bytes
        // below MAX_PAYLOAD_BYTES previously got no enforcement at all --
        // only the requested *budget itself* was checked against the
        // ceiling, never the actual payload against that narrower budget.
        let reg = registry_with_turbovec();
        let mut req = base_request(HubCapabilitySet::empty().grant(HubCapability::VectorSearch));
        req.payload = vec![0u8; 100];
        req.resource_budget.input_bytes = 10;
        assert!(matches!(
            admit(&reg, &req, base_ambient()).unwrap_err(),
            HubFault::InputRejected(_)
        ));
    }

    #[test]
    fn payload_within_the_requests_own_input_bytes_budget_is_admitted() {
        let reg = registry_with_turbovec();
        let mut req = base_request(HubCapabilitySet::empty().grant(HubCapability::VectorSearch));
        req.payload = vec![0u8; 10];
        req.resource_budget.input_bytes = 100;
        assert!(admit(&reg, &req, base_ambient()).is_ok());
    }

    #[test]
    fn queue_full_is_rejected_when_ambient_depth_meets_budgeted_depth() {
        let reg = registry_with_turbovec();
        let req = base_request(HubCapabilitySet::empty().grant(HubCapability::VectorSearch));
        let ambient = AdmissionAmbient {
            current_queue_depth: req.resource_budget.queue_depth,
            ..base_ambient()
        };
        assert_eq!(admit(&reg, &req, ambient).unwrap_err(), HubFault::QueueFull);
    }

    #[test]
    fn already_cancelled_request_is_rejected_before_any_other_check() {
        let reg = registry_with_turbovec();
        let mut req = base_request(HubCapabilitySet::empty());
        req.tool_id = HubToolId::new("solver.z3").unwrap(); // would otherwise be UnknownTool
        let ambient = AdmissionAmbient {
            already_cancelled: true,
            ..base_ambient()
        };
        assert_eq!(admit(&reg, &req, ambient).unwrap_err(), HubFault::Cancelled);
    }

    #[test]
    fn unsupported_api_version_is_rejected() {
        let reg = registry_with_turbovec();
        let mut req = base_request(HubCapabilitySet::empty().grant(HubCapability::VectorSearch));
        req.api_version = HubApiVersion::new(0, 999);
        assert_eq!(
            admit(&reg, &req, base_ambient()).unwrap_err(),
            HubFault::ApiVersionUnsupported
        );
    }

    #[test]
    fn unsupported_schema_version_is_rejected() {
        let reg = registry_with_turbovec();
        let mut req = base_request(HubCapabilitySet::empty().grant(HubCapability::VectorSearch));
        req.schema_version = 9999;
        assert_eq!(
            admit(&reg, &req, base_ambient()).unwrap_err(),
            HubFault::SchemaVersionUnsupported
        );
    }
}
