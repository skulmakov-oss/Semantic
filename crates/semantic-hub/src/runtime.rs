//! The Hub runtime: wires registry lookup, admission, dispatch, panic
//! containment, output validation, and audit recording into one path that
//! every invocation must go through.

use std::collections::HashMap;
use std::panic::{self, AssertUnwindSafe};
use std::time::{Duration, Instant};

use crate::admission::{admit, AdmissionAmbient};
use crate::audit::HubAuditRecord;
use crate::capability::HubCapabilitySet;
use crate::descriptor::HubToolDescriptor;
use crate::envelope::{HubReply, HubReplyStatus, HubRequest};
use crate::fault::{HubFault, HubToolError};
use crate::ids::{HubApiVersion, HubOperationId, HubToolId};
use crate::provenance::HubDigest;
use crate::registry::ToolRegistry;
use crate::resource::HubResourceUsage;

/// The bounded, read-only view an adapter receives while handling one
/// request. Adapters never receive a mutable reference to the registry,
/// audit trail, or any other adapter's state -- only their own operation
/// input and this restricted context.
pub struct RestrictedHubContext<'a> {
    pub resource_budget: &'a crate::resource::HubResourceBudget,
    pub capability_context: &'a HubCapabilitySet,
    pub deadline: Option<Instant>,
}

impl<'a> RestrictedHubContext<'a> {
    pub fn deadline_exceeded(&self) -> bool {
        self.deadline.is_some_and(|d| Instant::now() > d)
    }
}

/// The adapter contract every in-process Hub tool implements. `&mut self`
/// gives the adapter private mutable state; it receives no reference to
/// Semantic internals, the registry, or any other tool.
pub trait HubTool: Send {
    fn descriptor(&self) -> &HubToolDescriptor;

    fn handle(
        &mut self,
        operation_id: &HubOperationId,
        payload: &[u8],
        context: &RestrictedHubContext,
    ) -> Result<Vec<u8>, HubToolError>;

    /// Optional structural check on this tool's own reply shape, run after
    /// `handle` succeeds and before the reply is accepted. A `Err` here is
    /// classified as `ProtocolViolation`, distinct from a tool-declared
    /// operation failure. Default: no additional structural check.
    fn validate_reply(
        &self,
        _operation_id: &HubOperationId,
        _payload: &[u8],
    ) -> Result<(), String> {
        Ok(())
    }
}

/// Errors from registering a tool's live worker instance, distinct from
/// descriptor-only `RegistryError` (which validates the static shape).
#[derive(Debug)]
pub enum HubRegistrationError {
    Descriptor(crate::registry::RegistryError),
}

impl std::fmt::Display for HubRegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HubRegistrationError::Descriptor(e) => write!(f, "{e}"),
        }
    }
}

/// The Hub runtime for one process. Owns the tool registry, live worker
/// instances, and the append-only audit trail.
pub struct Hub {
    registry: ToolRegistry,
    workers: HashMap<HubToolId, Box<dyn HubTool>>,
    audit: crate::audit::HubAuditTrail,
    concurrent_requests: u32,
    next_sequence: u64,
}

impl Hub {
    pub fn new() -> Self {
        Self {
            registry: ToolRegistry::new(HubApiVersion::CURRENT),
            workers: HashMap::new(),
            audit: crate::audit::HubAuditTrail::new(),
            concurrent_requests: 0,
            next_sequence: 0,
        }
    }

    /// Advance this Hub's audit sequence counter to `next`. The Hub CLI is
    /// one short-lived process per invocation, so each process starts a
    /// fresh, empty in-memory `Hub`/`HubAuditTrail` -- without this, every
    /// invocation would emit a record starting at sequence 0, breaking the
    /// monotonic-sequence invariant of the *persisted* audit log the CLI
    /// appends to on disk. The caller is responsible for reading that
    /// persisted log's next sequence number and seeding it here before the
    /// first `invoke`.
    pub fn seed_next_sequence(&mut self, next: u64) {
        self.next_sequence = next;
    }

    pub fn register_tool(&mut self, tool: Box<dyn HubTool>) -> Result<(), HubRegistrationError> {
        let descriptor = tool.descriptor().clone();
        self.registry
            .register(descriptor.clone())
            .map_err(HubRegistrationError::Descriptor)?;
        self.workers.insert(descriptor.tool_id, tool);
        Ok(())
    }

    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }

    pub fn audit(&self) -> &crate::audit::HubAuditTrail {
        &self.audit
    }

    /// Run the full admission -> dispatch -> validation -> audit path for
    /// one request. Always returns a `HubReply` (never panics or silently
    /// drops evidence) -- rejection, tool failure, and crash are all
    /// represented as reply statuses with audit records, not exceptions.
    pub fn invoke(&mut self, request: HubRequest, deadline: Option<Instant>) -> HubReply {
        let sequence = self.next_sequence;
        self.next_sequence += 1;
        let started = Instant::now();

        // Derive a deadline from the admitted resource_budget.wall_time_millis
        // and combine it with any caller-supplied deadline (the tighter of
        // the two wins). Honest scope: this makes wall_time_millis actually
        // participate in the deadline check at admission/dispatch entry --
        // it previously did not, so declaring even a 1ms budget had no
        // effect at all. It is still NOT preemptive: a single native
        // adapter call already in flight (e.g. one TurboVec operation)
        // cannot be interrupted mid-computation once dispatch has started,
        // since that would require detached background execution with
        // shared-ownership worker state, a larger change than an
        // in-process, synchronous-per-invocation v0 architecture supports.
        // See docs/spec/hub/hub_adapter_contract_v0.md for this limitation
        // stated explicitly.
        let budget_deadline =
            started + Duration::from_millis(request.resource_budget.wall_time_millis);
        let effective_deadline = Some(match deadline {
            Some(d) => d.min(budget_deadline),
            None => budget_deadline,
        });

        if let Some(d) = effective_deadline {
            if Instant::now() > d {
                return self.finish_pre_dispatch(request, sequence, HubFault::DeadlineExceeded);
            }
        }

        let ambient = AdmissionAmbient {
            current_queue_depth: 0,
            current_concurrent_requests: self.concurrent_requests,
            already_cancelled: false,
        };

        let admitted = match admit(&self.registry, &request, ambient) {
            Ok(a) => a,
            Err(fault) => return self.finish_pre_dispatch(request, sequence, fault),
        };

        self.concurrent_requests = self.concurrent_requests.saturating_add(1);
        let result = self.dispatch(&request, &admitted.tool, effective_deadline, started);
        self.concurrent_requests = self.concurrent_requests.saturating_sub(1);
        self.record_and_reply(request, sequence, started, admitted.tool, result)
    }

    fn dispatch(
        &mut self,
        request: &HubRequest,
        tool: &HubToolDescriptor,
        deadline: Option<Instant>,
        _started: Instant,
    ) -> Result<Vec<u8>, HubFault> {
        let Some(health) = self.registry.health_mut(&tool.tool_id) else {
            return Err(HubFault::InternalHubFault(
                "worker health missing for registered tool".into(),
            ));
        };
        if !health.state().accepts_dispatch()
            && health.state() != crate::worker::HubWorkerState::Registered
        {
            return Err(HubFault::ToolQuarantined);
        }
        // v0 lifecycle: a freshly registered worker starts on first dispatch.
        if health.state() == crate::worker::HubWorkerState::Registered {
            let _ = health.mark_starting();
            let _ = health.mark_ready();
        }
        let _ = health.mark_busy();

        let Some(worker) = self.workers.get_mut(&tool.tool_id) else {
            return Err(HubFault::InternalHubFault(
                "worker instance missing for registered tool".into(),
            ));
        };

        // Never hand the adapter the caller's raw, unsanitized capability
        // set: `satisfies()` only verified the operation's *required*
        // capabilities were present and non-sensitive, it never strips a
        // sensitive capability the caller happened to include alongside
        // them. An adapter naively calling `.allows(NetworkAccess)` on the
        // raw set could otherwise observe `true` for something Hub
        // structurally denies. The unsanitized set is still what the audit
        // record captures (via `request.capability_context` directly).
        let sanitized_capabilities = request.capability_context.deny_sensitive();
        let context = RestrictedHubContext {
            resource_budget: &request.resource_budget,
            capability_context: &sanitized_capabilities,
            deadline,
        };

        let operation_id = request.operation_id.clone();
        let payload = request.payload.clone();
        let outcome = panic::catch_unwind(AssertUnwindSafe(|| {
            worker.handle(&operation_id, &payload, &context)
        }));

        let health = self
            .registry
            .health_mut(&tool.tool_id)
            .expect("health present: checked above");

        match outcome {
            Ok(Ok(bytes)) => {
                if bytes.len() as u64 > request.resource_budget.output_bytes {
                    let _ = health.mark_ready();
                    return Err(HubFault::OutputRejected(format!(
                        "output {} bytes exceeds budgeted {} bytes",
                        bytes.len(),
                        request.resource_budget.output_bytes
                    )));
                }
                let worker_ref = self.workers.get(&tool.tool_id).expect("worker present");
                if let Err(reason) = worker_ref.validate_reply(&request.operation_id, &bytes) {
                    let _ = health.report_protocol_violation();
                    return Err(HubFault::ProtocolViolation(reason));
                }
                let _ = health.mark_ready();
                Ok(bytes)
            }
            Ok(Err(tool_error)) => {
                let _ = health.mark_ready();
                Err(HubFault::ToolDeclaredFailure(tool_error.to_string()))
            }
            Err(panic_payload) => {
                let message = panic_message(&panic_payload);
                let _ = health.report_crash();
                Err(HubFault::WorkerPanicked(message))
            }
        }
    }

    fn finish_pre_dispatch(
        &mut self,
        request: HubRequest,
        sequence: u64,
        fault: HubFault,
    ) -> HubReply {
        // A pre-dispatch rejection (e.g. capability denial) still knows the
        // real tool if the tool_id was recognized -- look it up so the
        // reply/audit record the actual registered tool_version/adapter
        // provenance instead of a placeholder. Only a genuinely unknown
        // tool_id falls back to the placeholder descriptor.
        let tool = self
            .registry
            .descriptor(&request.tool_id)
            .cloned()
            .unwrap_or_else(placeholder_descriptor);
        self.record_and_reply(request, sequence, Instant::now(), tool, Err(fault))
    }

    fn record_and_reply(
        &mut self,
        request: HubRequest,
        sequence: u64,
        started: Instant,
        tool: HubToolDescriptor,
        result: Result<Vec<u8>, HubFault>,
    ) -> HubReply {
        let elapsed_millis = started.elapsed().as_millis() as u64;
        let input_digest = HubDigest::of(&request.payload);

        let (status, payload, output_digest, worker_state_after, status_code, fault_code) =
            match result {
                Ok(bytes) => {
                    let digest = HubDigest::of(&bytes);
                    let state = self
                        .registry
                        .worker_state(&request.tool_id)
                        .unwrap_or(crate::worker::HubWorkerState::Registered);
                    (
                        HubReplyStatus::Success,
                        bytes,
                        digest,
                        state,
                        "Success",
                        None,
                    )
                }
                Err(fault) => {
                    let digest = HubDigest::of(&[]);
                    let state = self
                        .registry
                        .worker_state(&request.tool_id)
                        .unwrap_or(crate::worker::HubWorkerState::Registered);
                    let fault_code = fault.code();
                    let status = if fault.is_pre_dispatch_rejection() {
                        HubReplyStatus::Rejected(fault.clone())
                    } else {
                        match &fault {
                            HubFault::WorkerPanicked(_) => HubReplyStatus::Crashed(fault.clone()),
                            HubFault::InternalHubFault(_) => {
                                HubReplyStatus::HubFault(fault.clone())
                            }
                            _ => HubReplyStatus::ToolFailed(fault.clone()),
                        }
                    };
                    // `status_code` is the reply-status discriminant
                    // ("Rejected"/"ToolFailed"/"Crashed"/"HubFault"), kept
                    // distinct from `fault_code` (the specific fault, e.g.
                    // "CapabilityDenied") -- conflating the two here once
                    // broke `HubAuditRecord::from_canonical_line`, which
                    // only recognizes the reply-status set for that field.
                    let status_code = status.as_str();
                    (
                        status,
                        Vec::new(),
                        digest,
                        state,
                        status_code,
                        Some(fault_code),
                    )
                }
            };

        let resource_usage = HubResourceUsage {
            wall_time_millis: Some(elapsed_millis),
            output_bytes: Some(payload.len() as u64),
            input_bytes: Some(request.payload.len() as u64),
            ..HubResourceUsage::default()
        };

        let record = HubAuditRecord {
            sequence,
            request_id: request.request_id.clone(),
            session_id: request.session_id.clone(),
            caller_identity: request.caller_identity.clone(),
            tool_id: request.tool_id.clone(),
            tool_version: tool.tool_version,
            adapter_provenance: tool.adapter_provenance.clone(),
            operation_id: request.operation_id.clone(),
            execution_mode: tool.execution_mode,
            determinism: tool
                .operation(&request.operation_id)
                .map(|op| op.determinism)
                .unwrap_or(crate::execution::HubDeterminismClass::Unknown),
            trust_class: tool.trust_class,
            privacy_class: request.privacy_class,
            capabilities_granted: request.capability_context.iter().copied().collect(),
            input_digest,
            output_digest,
            resource_budget: request.resource_budget,
            resource_usage,
            worker_state_after,
            status_code,
            fault_code,
        };
        self.audit.push(record);

        HubReply {
            schema_version: crate::envelope::HUB_ENVELOPE_SCHEMA_VERSION,
            request_id: request.request_id,
            tool_id: request.tool_id,
            tool_version: tool.tool_version,
            operation_id: request.operation_id,
            status,
            payload,
            resource_usage,
        }
    }
}

impl Default for Hub {
    fn default() -> Self {
        Self::new()
    }
}

fn placeholder_descriptor() -> HubToolDescriptor {
    HubToolDescriptor {
        tool_id: HubToolId::new("hub.internal").unwrap(),
        name: "hub-internal".into(),
        tool_version: crate::ids::HubToolVersion::new(0, 0, 0),
        hub_api_version: HubApiVersion::CURRENT,
        execution_mode: crate::execution::HubExecutionMode::InProcess,
        trust_class: crate::execution::HubTrustClass::InProcessUnisolated,
        operations: vec![crate::descriptor::HubOperationDescriptor::new(
            HubOperationId::new("hub.reject").unwrap(),
            [],
            crate::execution::HubDeterminismClass::Deterministic,
            false,
        )],
        resource_ceiling: crate::resource::HubResourceBudget::V0_CEILING,
        adapter_provenance: "hub-internal (pre-dispatch rejection placeholder)".into(),
    }
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "worker panicked with a non-string payload".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::{HubCapability, HubCapabilitySet};
    use crate::descriptor::HubOperationDescriptor;
    use crate::execution::{HubDeterminismClass, HubExecutionMode, HubPrivacyClass, HubTrustClass};
    use crate::ids::{HubCallerIdentity, HubSessionId, HubToolVersion};

    /// Narrow test-only tool that forces each otherwise-unreachable failure
    /// path (tool-declared failure, panic, protocol violation) on demand by
    /// operation name. Exists only under `#[cfg(test)]` in the generic Hub
    /// crate -- it is not a public product tool and is never reachable
    /// outside this module's tests.
    struct FaultInjectionTool {
        descriptor: HubToolDescriptor,
    }

    impl FaultInjectionTool {
        fn new() -> Self {
            let descriptor = HubToolDescriptor {
                tool_id: HubToolId::new("test.fault-injection").unwrap(),
                name: "fault-injection".into(),
                tool_version: HubToolVersion::new(1, 2, 3),
                hub_api_version: HubApiVersion::CURRENT,
                execution_mode: HubExecutionMode::InProcess,
                trust_class: HubTrustClass::InProcessUnisolated,
                operations: vec![
                    HubOperationDescriptor::new(
                        HubOperationId::new("test.succeed").unwrap(),
                        [HubCapability::CpuCompute],
                        HubDeterminismClass::Deterministic,
                        false,
                    ),
                    HubOperationDescriptor::new(
                        HubOperationId::new("test.fail").unwrap(),
                        [HubCapability::CpuCompute],
                        HubDeterminismClass::Deterministic,
                        false,
                    ),
                    HubOperationDescriptor::new(
                        HubOperationId::new("test.panic").unwrap(),
                        [HubCapability::CpuCompute],
                        HubDeterminismClass::Deterministic,
                        false,
                    ),
                    HubOperationDescriptor::new(
                        HubOperationId::new("test.protocol-violation").unwrap(),
                        [HubCapability::CpuCompute],
                        HubDeterminismClass::Deterministic,
                        false,
                    ),
                    HubOperationDescriptor::new(
                        HubOperationId::new("test.report-network-access-visibility").unwrap(),
                        [HubCapability::CpuCompute],
                        HubDeterminismClass::Deterministic,
                        false,
                    ),
                ],
                resource_ceiling: crate::resource::HubResourceBudget::V0_CEILING,
                adapter_provenance: "test-only fault injection tool".into(),
            };
            Self { descriptor }
        }
    }

    impl HubTool for FaultInjectionTool {
        fn descriptor(&self) -> &HubToolDescriptor {
            &self.descriptor
        }

        fn handle(
            &mut self,
            operation_id: &HubOperationId,
            _payload: &[u8],
            context: &RestrictedHubContext,
        ) -> Result<Vec<u8>, HubToolError> {
            match operation_id.as_str() {
                "test.succeed" => Ok(b"ok".to_vec()),
                "test.fail" => Err(HubToolError::new(
                    "TestDeclaredFailure",
                    "intentional test failure",
                )),
                "test.panic" => panic!("intentional test panic"),
                "test.protocol-violation" => Ok(b"not-json".to_vec()),
                "test.report-network-access-visibility" => {
                    if context
                        .capability_context
                        .allows(HubCapability::NetworkAccess)
                    {
                        Ok(b"leaked".to_vec())
                    } else {
                        Ok(b"denied".to_vec())
                    }
                }
                other => Err(HubToolError::new("UnknownOperation", other)),
            }
        }

        fn validate_reply(
            &self,
            operation_id: &HubOperationId,
            payload: &[u8],
        ) -> Result<(), String> {
            if operation_id.as_str() == "test.protocol-violation" {
                return Err("forced protocol violation for test".into());
            }
            let _ = payload;
            Ok(())
        }
    }

    fn hub_with_fault_injection_tool() -> Hub {
        let mut hub = Hub::new();
        hub.register_tool(Box::new(FaultInjectionTool::new()))
            .unwrap();
        hub
    }

    fn request_for(operation: &str, capabilities: HubCapabilitySet) -> HubRequest {
        HubRequest {
            schema_version: crate::envelope::HUB_ENVELOPE_SCHEMA_VERSION,
            api_version: HubApiVersion::CURRENT,
            request_id: crate::ids::HubRequestId::new("req-1").unwrap(),
            session_id: HubSessionId::new("sess-1").unwrap(),
            caller_identity: HubCallerIdentity::new("cli:local").unwrap(),
            tool_id: HubToolId::new("test.fault-injection").unwrap(),
            operation_id: HubOperationId::new(operation).unwrap(),
            capability_context: capabilities,
            privacy_class: HubPrivacyClass::ProjectLocal,
            resource_budget: crate::resource::HubResourceBudget::V0_CEILING,
            payload: b"{}".to_vec(),
        }
    }

    fn granted() -> HubCapabilitySet {
        HubCapabilitySet::empty().grant(HubCapability::CpuCompute)
    }

    #[test]
    fn successful_invocation_produces_success_reply_and_matching_audit_record() {
        let mut hub = hub_with_fault_injection_tool();
        let reply = hub.invoke(request_for("test.succeed", granted()), None);
        assert!(reply.status.is_success());
        assert_eq!(reply.payload, b"ok");

        let records = hub.audit().records();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].status_code, "Success");
        assert_eq!(records[0].fault_code, None);
        assert_eq!(records[0].tool_version, HubToolVersion::new(1, 2, 3));
    }

    #[test]
    fn capability_denial_records_the_real_tool_version_not_a_placeholder() {
        // Regression test: a pre-dispatch rejection for a KNOWN tool must
        // reflect that tool's real registered version, not the internal
        // placeholder descriptor's 0.0.0.
        let mut hub = hub_with_fault_injection_tool();
        let reply = hub.invoke(request_for("test.succeed", HubCapabilitySet::empty()), None);
        assert!(matches!(reply.status, HubReplyStatus::Rejected(_)));
        assert_eq!(reply.tool_version, HubToolVersion::new(1, 2, 3));

        let records = hub.audit().records();
        assert_eq!(records[0].tool_version, HubToolVersion::new(1, 2, 3));
    }

    #[test]
    fn rejection_audit_record_status_code_and_fault_code_are_distinct_and_round_trip() {
        // Regression test: status_code must be the reply-status
        // discriminant ("Rejected"), never the specific fault code
        // ("CapabilityDenied") -- conflating the two previously made
        // `HubAuditTrail::from_canonical_text` reject its own output.
        let mut hub = hub_with_fault_injection_tool();
        hub.invoke(request_for("test.succeed", HubCapabilitySet::empty()), None);

        let record = &hub.audit().records()[0];
        assert_eq!(record.status_code, "Rejected");
        assert_eq!(record.fault_code, Some("CapabilityDenied"));

        let text = hub.audit().to_canonical_text();
        let reloaded = crate::audit::HubAuditTrail::from_canonical_text(&text)
            .expect("a Hub's own audit output must always parse back");
        assert_eq!(reloaded.records(), hub.audit().records());
    }

    #[test]
    fn tool_declared_failure_is_reported_as_tool_failed_not_crashed() {
        let mut hub = hub_with_fault_injection_tool();
        let reply = hub.invoke(request_for("test.fail", granted()), None);
        assert!(matches!(reply.status, HubReplyStatus::ToolFailed(_)));
        assert_eq!(hub.audit().records()[0].status_code, "ToolFailed");
        assert_eq!(
            hub.audit().records()[0].fault_code,
            Some("ToolDeclaredFailure")
        );
    }

    #[test]
    fn worker_panic_is_contained_and_reported_as_crashed() {
        let mut hub = hub_with_fault_injection_tool();
        let reply = hub.invoke(request_for("test.panic", granted()), None);
        assert!(matches!(reply.status, HubReplyStatus::Crashed(_)));
        assert_eq!(hub.audit().records()[0].status_code, "Crashed");
        assert_eq!(hub.audit().records()[0].fault_code, Some("WorkerPanicked"));
        // The panic must not have poisoned the Hub: a second, unrelated
        // invocation on a fresh operation still completes normally.
        let second = hub.invoke(request_for("test.succeed", granted()), None);
        assert!(
            matches!(second.status, HubReplyStatus::Success)
                || matches!(second.status, HubReplyStatus::Rejected(_))
        );
    }

    #[test]
    fn protocol_violation_quarantines_the_worker_and_is_distinct_from_tool_failure() {
        let mut hub = hub_with_fault_injection_tool();
        let reply = hub.invoke(request_for("test.protocol-violation", granted()), None);
        assert!(matches!(reply.status, HubReplyStatus::ToolFailed(_)));
        assert_eq!(
            hub.audit().records()[0].fault_code,
            Some("ProtocolViolation")
        );
        assert_eq!(
            hub.registry()
                .worker_state(&HubToolId::new("test.fault-injection").unwrap()),
            Some(crate::worker::HubWorkerState::Quarantined)
        );

        // Quarantine must be visible to the NEXT invocation, not just this
        // one's audit record.
        let next = hub.invoke(request_for("test.succeed", granted()), None);
        assert!(matches!(next.status, HubReplyStatus::Rejected(_)));
        assert_eq!(next.status.fault().unwrap().code(), "ToolQuarantined");
    }

    #[test]
    fn seed_next_sequence_advances_subsequent_audit_sequence_numbers() {
        let mut hub = hub_with_fault_injection_tool();
        hub.seed_next_sequence(41);
        hub.invoke(request_for("test.succeed", granted()), None);
        assert_eq!(hub.audit().records()[0].sequence, 41);
    }

    #[test]
    fn a_sensitive_capability_granted_by_the_caller_is_not_visible_to_the_adapter() {
        // Regression test: `dispatch` previously handed the adapter the
        // caller's raw, unsanitized capability set. An admitted request only
        // needs its operation's *required* (non-sensitive) capabilities to
        // satisfy admission -- it may still carry additional, sensitive
        // capabilities alongside them, and the adapter must never observe
        // those as granted.
        let mut hub = hub_with_fault_injection_tool();
        let capabilities = granted().grant(HubCapability::NetworkAccess);
        let reply = hub.invoke(
            request_for("test.report-network-access-visibility", capabilities),
            None,
        );
        assert!(reply.status.is_success());
        assert_eq!(reply.payload, b"denied");

        // The raw, unsanitized grant is still what the audit trail records
        // -- sanitization is a dispatch-boundary concern, not an audit one.
        assert!(hub.audit().records()[0]
            .capabilities_granted
            .contains(&HubCapability::NetworkAccess));
    }

    #[test]
    fn a_tiny_wall_time_budget_produces_deadline_exceeded_before_dispatch() {
        // Regression test: resource_budget.wall_time_millis was documented
        // as hard-enforced but was never actually read anywhere -- a 0ms
        // budget had no effect at all. `invoke` must derive a deadline from
        // it and reject before the adapter ever runs.
        let mut hub = hub_with_fault_injection_tool();
        let mut request = request_for("test.succeed", granted());
        request.resource_budget.wall_time_millis = 0;
        let reply = hub.invoke(request, None);
        // DeadlineExceeded is classified as a post-dispatch fault in the
        // taxonomy (`HubFault::is_pre_dispatch_rejection` is false for it)
        // even though this particular check runs before admission, so the
        // reply status is `ToolFailed`, not `Rejected`.
        assert!(matches!(reply.status, HubReplyStatus::ToolFailed(_)));
        assert_eq!(reply.status.fault().unwrap().code(), "DeadlineExceeded");
        // Never dispatched: no worker state transition, no attempt to run
        // the adapter at all.
        assert_eq!(
            hub.audit().records()[0].fault_code,
            Some("DeadlineExceeded")
        );
    }
}
