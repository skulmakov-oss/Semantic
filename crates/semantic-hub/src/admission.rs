//! Single request-admission path. No convenience method may dispatch a
//! request to a tool worker without going through [`admit`] first.

use crate::capability::{HubCapability, HubCapabilitySet};
use crate::descriptor::{HubOperationDescriptor, HubToolDescriptor};
use crate::envelope::{HubRequest, MAX_PAYLOAD_BYTES};
use crate::execution::HubPrivacyClass;
use crate::fault::HubFault;
use crate::ids::HubCallerIdentity;
use crate::registry::ToolRegistry;
use crate::resource::{HubBudgetExceeded, HubResourceBudget, HubResourceKind};
use crate::worker::HubWorkerState;

/// The rolling, whole-session ceiling a `HubSession` (see `crate::session`)
/// attenuates every admitted request's own per-request budget with.
/// Distinct from `HubResourceBudget`, which is per-request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HubSessionCeiling {
    pub max_request_count: u32,
    pub max_cumulative_input_bytes: u64,
    pub max_cumulative_output_bytes: u64,
    pub max_cumulative_wall_time_millis: u64,
    pub max_queue_depth: u32,
    pub max_concurrent_requests: u32,
}

impl HubSessionCeiling {
    /// A conservative default ceiling for one CLI-driven batch session --
    /// generous enough for a large, legitimate batch while still bounding
    /// runaway input (e.g. a malformed NDJSON file with millions of lines).
    pub const V0_DEFAULT: HubSessionCeiling = HubSessionCeiling {
        max_request_count: 10_000,
        max_cumulative_input_bytes: 256 * 1024 * 1024,
        max_cumulative_output_bytes: 256 * 1024 * 1024,
        max_cumulative_wall_time_millis: 10 * 60 * 1000,
        max_queue_depth: 1,
        max_concurrent_requests: 1,
    };
}

/// Session-scoped ambient state threaded into admission when a request is
/// submitted through a `HubSession` rather than directly via `Hub::invoke`.
/// `*_so_far` fields reflect state accumulated strictly BEFORE the request
/// currently being admitted (never including it) -- `admit` itself decides
/// whether adding this request would cross the ceiling.
#[derive(Debug, Clone)]
pub struct SessionAdmissionAmbient {
    pub ceiling: HubSessionCeiling,
    pub caller_identity: HubCallerIdentity,
    pub capability_ceiling: HubCapabilitySet,
    pub privacy_ceiling: HubPrivacyClass,
    pub requests_submitted_so_far: u32,
    pub cumulative_input_bytes_so_far: u64,
    pub cumulative_output_bytes_so_far: u64,
    pub cumulative_wall_time_millis_so_far: u64,
}

/// Ambient state the admission path needs beyond the registry and request
/// itself: current queue occupancy, whether cancellation was already
/// requested for this request id before admission ran, and -- only when
/// admission is running inside a `HubSession` -- the session's own
/// cumulative ceiling state.
#[derive(Debug, Clone)]
pub struct AdmissionAmbient {
    pub current_queue_depth: u32,
    pub current_concurrent_requests: u32,
    pub already_cancelled: bool,
    pub session: Option<SessionAdmissionAmbient>,
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
    if ambient.already_cancelled {
        return Err(HubFault::Cancelled);
    }

    // 2. A session fixes identity, capability, privacy, and cumulative
    // ceilings before its first dispatch. A later request may narrow these
    // values but may never widen or replace them.
    if let Some(session) = &ambient.session {
        macro_rules! session_check {
            ($so_far:expr, $delta:expr, $limit:expr, $kind:expr) => {
                let attempted =
                    $so_far
                        .checked_add($delta)
                        .ok_or(HubFault::SessionLimitExceeded(HubBudgetExceeded {
                            kind: $kind,
                            limit: $limit as u64,
                            attempted: u64::MAX,
                        }))?;
                if attempted > $limit as u64 {
                    return Err(HubFault::SessionLimitExceeded(HubBudgetExceeded {
                        kind: $kind,
                        limit: $limit as u64,
                        attempted,
                    }));
                }
            };
        }
        session_check!(
            session.requests_submitted_so_far as u64,
            1u64,
            session.ceiling.max_request_count,
            HubResourceKind::SessionRequestCount
        );
        session_check!(
            session.cumulative_input_bytes_so_far,
            request.payload.len() as u64,
            session.ceiling.max_cumulative_input_bytes,
            HubResourceKind::SessionInputBytes
        );
        session_check!(
            session.cumulative_output_bytes_so_far,
            0u64,
            session.ceiling.max_cumulative_output_bytes,
            HubResourceKind::SessionOutputBytes
        );
        session_check!(
            session.cumulative_wall_time_millis_so_far,
            request.resource_budget.wall_time_millis,
            session.ceiling.max_cumulative_wall_time_millis,
            HubResourceKind::SessionWallTimeMillis
        );

        if request.caller_identity != session.caller_identity {
            return Err(HubFault::InputRejected(
                "request caller identity does not match the session caller identity".into(),
            ));
        }
        if !request
            .capability_context
            .is_subset_of(&session.capability_ceiling)
        {
            return Err(HubFault::CapabilityDenied(
                "request capability context exceeds the session capability ceiling".into(),
            ));
        }
        if request.privacy_class > session.privacy_ceiling {
            return Err(HubFault::InputRejected(format!(
                "request privacy class {} exceeds session ceiling {}",
                request.privacy_class, session.privacy_ceiling
            )));
        }
    }

    // The budget check below only verifies the requested budget itself;
    // enforce the actual payload against the (possibly session-attenuated)
    // per-request input budget too. Session cumulative input is checked
    // first so exhausting that ceiling retains its distinct fault.
    if request.payload.len() as u64 > request.resource_budget.input_bytes {
        return Err(HubFault::InputRejected(format!(
            "payload {} bytes exceeds the request's own declared input_bytes budget {}",
            request.payload.len(),
            request.resource_budget.input_bytes
        )));
    }

    // 3. Registry lookup: unknown tool vs. unknown operation are distinct.
    let tool = registry
        .descriptor(&request.tool_id)
        .ok_or(HubFault::UnknownTool)?
        .clone();
    let operation = tool
        .operation(&request.operation_id)
        .ok_or(HubFault::UnknownOperation)?
        .clone();

    // 4. Worker health/lifecycle gate. A mutating operation against a
    // Degraded worker is rejected distinctly from one that only reads:
    // supervision already tolerates a Degraded worker continuing to serve
    // reads (see HubWorkerState::accepts_dispatch), but letting it keep
    // mutating durable state while its own crash count is elevated widens
    // the blast radius of whatever is making it unstable.
    match registry.worker_state(&request.tool_id) {
        Some(HubWorkerState::Disabled) => return Err(HubFault::ToolDisabled),
        Some(HubWorkerState::Quarantined) => return Err(HubFault::ToolQuarantined),
        Some(HubWorkerState::Stopped) => return Err(HubFault::ToolDisabled),
        Some(HubWorkerState::Degraded) if operation.mutates_tool_state => {
            return Err(HubFault::WorkerDegraded)
        }
        _ => {}
    }

    // 5. Capability policy: deny-by-default, checked before dispatch. A
    // request carrying ANY sensitive capability in its own capability
    // context is rejected outright, even if that capability is not
    // required by the operation and would otherwise just be silently
    // stripped by `deny_sensitive()` before the adapter ever saw it --
    // asking for a capability Hub v0 structurally never grants to any
    // tool is a caller error worth surfacing plainly, not absorbing.
    if let Some(sensitive) = request.capability_context.iter().find(|c| c.is_sensitive()) {
        return Err(HubFault::SensitiveCapabilityDenied(format!(
            "request capability_context grants sensitive capability {sensitive}, which is denied by default and not supported by any Hub v0 tool"
        )));
    }
    check_capabilities(
        &request.capability_context,
        &operation.required_capabilities,
    )?;

    // 6. Resource budget: immutable ceiling check, checked arithmetic only.
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

    // 7. Queue/concurrency admission.
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
            session: None,
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

    #[test]
    fn a_request_granting_a_sensitive_capability_is_rejected_even_if_unrequired() {
        let reg = registry_with_turbovec();
        let req = base_request(
            HubCapabilitySet::empty()
                .grant(HubCapability::VectorSearch)
                .grant(HubCapability::NetworkAccess),
        );
        assert!(matches!(
            admit(&reg, &req, base_ambient()).unwrap_err(),
            HubFault::SensitiveCapabilityDenied(_)
        ));
    }

    fn registry_with_mutating_turbovec() -> ToolRegistry {
        let mut reg = ToolRegistry::new(HubApiVersion::CURRENT);
        reg.register(HubToolDescriptor {
            tool_id: HubToolId::new("vector.turbovec").unwrap(),
            name: "TurboVec".into(),
            tool_version: HubToolVersion::new(0, 1, 0),
            hub_api_version: HubApiVersion::CURRENT,
            execution_mode: HubExecutionMode::InProcess,
            trust_class: HubTrustClass::InProcessUnisolated,
            operations: vec![
                HubOperationDescriptor::new(
                    HubOperationId::new("vector.search").unwrap(),
                    [HubCapability::VectorSearch],
                    HubDeterminismClass::Unknown,
                    false,
                ),
                HubOperationDescriptor::new(
                    HubOperationId::new("vector.index.insert").unwrap(),
                    [HubCapability::VectorIndexMutate],
                    HubDeterminismClass::Unknown,
                    true,
                ),
            ],
            resource_ceiling: HubResourceBudget::V0_CEILING,
            adapter_provenance: "test".into(),
        })
        .unwrap();
        reg
    }

    #[test]
    fn a_mutating_operation_against_a_degraded_worker_is_rejected() {
        let mut reg = registry_with_mutating_turbovec();
        reg.health_mut(&HubToolId::new("vector.turbovec").unwrap())
            .unwrap()
            .mark_starting()
            .unwrap();
        reg.health_mut(&HubToolId::new("vector.turbovec").unwrap())
            .unwrap()
            .mark_ready()
            .unwrap();
        reg.health_mut(&HubToolId::new("vector.turbovec").unwrap())
            .unwrap()
            .mark_busy()
            .unwrap();
        reg.health_mut(&HubToolId::new("vector.turbovec").unwrap())
            .unwrap()
            .report_crash()
            .unwrap(); // -> Degraded (first crash, under the conservative default's threshold of 3)

        let mut req =
            base_request(HubCapabilitySet::empty().grant(HubCapability::VectorIndexMutate));
        req.operation_id = HubOperationId::new("vector.index.insert").unwrap();
        assert_eq!(
            admit(&reg, &req, base_ambient()).unwrap_err(),
            HubFault::WorkerDegraded
        );
    }

    #[test]
    fn a_read_operation_against_a_degraded_worker_is_still_admitted() {
        let mut reg = registry_with_mutating_turbovec();
        let tool_id = HubToolId::new("vector.turbovec").unwrap();
        reg.health_mut(&tool_id).unwrap().mark_starting().unwrap();
        reg.health_mut(&tool_id).unwrap().mark_ready().unwrap();
        reg.health_mut(&tool_id).unwrap().mark_busy().unwrap();
        reg.health_mut(&tool_id).unwrap().report_crash().unwrap(); // -> Degraded

        let req = base_request(HubCapabilitySet::empty().grant(HubCapability::VectorSearch));
        assert!(admit(&reg, &req, base_ambient()).is_ok());
    }
}
