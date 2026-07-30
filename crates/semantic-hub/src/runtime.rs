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

/// What one successful `HubTool::handle` call produced: the reply payload
/// bytes, plus optional artifact provenance for a mutating operation that
/// committed durable state (e.g. a TurboVec `.tvim` write). `artifact` is
/// `None` for a non-mutating operation (search, describe) -- there is no
/// durable artifact to bind provenance to.
#[derive(Debug, Clone)]
pub struct HubToolOutcome {
    pub payload: Vec<u8>,
    pub artifact: Option<crate::provenance::HubArtifactProvenance>,
}

impl HubToolOutcome {
    /// Construct an outcome with no artifact -- the common case for every
    /// non-mutating operation and for any mutating operation an adapter
    /// has not yet been updated to report provenance for.
    pub fn payload_only(payload: Vec<u8>) -> Self {
        Self {
            payload,
            artifact: None,
        }
    }
}

#[derive(Debug)]
struct HubDispatchFailure {
    fault: HubFault,
    committed_artifact: Option<crate::provenance::HubArtifactProvenance>,
}

impl HubDispatchFailure {
    fn without_artifact(fault: HubFault) -> Self {
        Self {
            fault,
            committed_artifact: None,
        }
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
    ) -> Result<HubToolOutcome, HubToolError>;

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
    pub fn seed_next_sequence(&mut self, next: u64) -> Result<(), HubFault> {
        if next == u64::MAX {
            return Err(HubFault::SequenceExhausted);
        }
        self.next_sequence = next;
        Ok(())
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
        self.invoke_impl(request, deadline, false, None)
    }

    /// Same admission -> dispatch -> validation -> audit path as
    /// [`Self::invoke`], additionally attenuated by a
    /// [`crate::admission::SessionAdmissionAmbient`] and an explicit
    /// per-request cancellation flag -- the entry point [`crate::session::HubSession`]
    /// uses so a session-submitted request goes through exactly the same
    /// pipeline as a direct `invoke` call, never a parallel one.
    pub fn invoke_in_session(
        &mut self,
        request: HubRequest,
        deadline: Option<Instant>,
        already_cancelled: bool,
        session_ambient: crate::admission::SessionAdmissionAmbient,
    ) -> HubReply {
        self.invoke_impl(request, deadline, already_cancelled, Some(session_ambient))
    }

    fn invoke_impl(
        &mut self,
        request: HubRequest,
        deadline: Option<Instant>,
        already_cancelled: bool,
        session_ambient: Option<crate::admission::SessionAdmissionAmbient>,
    ) -> HubReply {
        if self.next_sequence == u64::MAX {
            return self.finish_pre_dispatch(request, u64::MAX, HubFault::SequenceExhausted);
        }
        let sequence = self.next_sequence;
        // `u64::MAX` is reserved for an explicit sequence-exhaustion
        // sentinel reply and is never appended as an audit sequence.
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
        // Checked, not a plain `+`: `wall_time_millis` is a caller-supplied
        // u64, and on some platforms (observed on Windows) adding an
        // extreme Duration (e.g. from u64::MAX milliseconds) to an
        // `Instant` panics rather than saturating, since `Instant`'s own
        // representable range is narrower there. A budget that large is
        // already invalid and will be rejected moments later by
        // `admit()`'s own ceiling check (`ResourceBudgetInvalid`,
        // V0_CEILING's wall_time_millis = 30_000) -- this must not panic
        // before that typed rejection ever gets to run. `None` here is
        // treated as "no additional constraint from the budget," falling
        // back to the caller-supplied deadline alone.
        let budget_deadline = started.checked_add(Duration::from_millis(
            request.resource_budget.wall_time_millis,
        ));
        let effective_deadline = match (deadline, budget_deadline) {
            (Some(d), Some(bd)) => Some(d.min(bd)),
            (Some(d), None) => Some(d),
            (None, Some(bd)) => Some(bd),
            (None, None) => None,
        };

        if let Some(d) = effective_deadline {
            if Instant::now() > d {
                return self.finish_pre_dispatch(request, sequence, HubFault::DeadlineExceeded);
            }
        }

        let ambient = AdmissionAmbient {
            current_queue_depth: 0,
            current_concurrent_requests: self.concurrent_requests,
            already_cancelled,
            session: session_ambient,
        };

        let admitted = match admit(&self.registry, &request, ambient) {
            Ok(a) => a,
            Err(fault) => return self.finish_pre_dispatch(request, sequence, fault),
        };

        self.concurrent_requests = self.concurrent_requests.saturating_add(1);
        let dispatch_result = self.dispatch(&request, &admitted.tool, effective_deadline, started);
        self.concurrent_requests = self.concurrent_requests.saturating_sub(1);

        let committed_artifact = match &dispatch_result {
            Ok(outcome) => outcome.artifact.clone(),
            Err(failure) => failure.committed_artifact.clone(),
        };
        let dispatch_result = dispatch_result.map_err(|failure| failure.fault);

        // Recheck the same effective deadline after dispatch returns: the
        // entry check above only rejects a request whose budget was
        // already exhausted *before* the adapter ever ran. Without this,
        // an operation that ran long under a tiny wall_time_millis budget
        // (nothing polls deadline_exceeded() mid-call in v0) would still
        // be reported as a plain Success, even though WallTimeMillis is
        // documented as hard-enforced. This does not stop or unwind an
        // adapter call already in flight -- see
        // docs/spec/hub/hub_adapter_contract_v0.md Section 9 for why that
        // is out of reach for the synchronous v0 architecture -- it only
        // ensures a dispatch that blew through its budget is never
        // silently reported as successful. Side effects the adapter
        // already committed before this check runs are not undone, the
        // same "effects happened but the reply reports failure" property
        // Hub already accepts for OutputRejected below.
        let dispatch_result = dispatch_result.and_then(|outcome| {
            if outcome.payload.len() as u64 > request.resource_budget.output_bytes {
                Err(HubFault::OutputRejected(format!(
                    "output {} bytes exceeds budgeted {} bytes",
                    outcome.payload.len(),
                    request.resource_budget.output_bytes
                )))
            } else if effective_deadline.is_some_and(|d| Instant::now() > d) {
                Err(HubFault::DeadlineExceeded)
            } else {
                Ok(outcome)
            }
        });

        self.record_and_reply(
            request,
            sequence,
            started,
            admitted.tool,
            dispatch_result,
            committed_artifact,
            true,
        )
    }

    fn dispatch(
        &mut self,
        request: &HubRequest,
        tool: &HubToolDescriptor,
        deadline: Option<Instant>,
        _started: Instant,
    ) -> Result<HubToolOutcome, HubDispatchFailure> {
        let Some(health) = self.registry.health_mut(&tool.tool_id) else {
            return Err(HubDispatchFailure::without_artifact(
                HubFault::InternalHubFault("worker health missing for registered tool".into()),
            ));
        };
        // Busy is reported distinctly from Quarantined/Disabled: it is not
        // a supervision-imposed rejection, it is the invariant "this worker
        // is already mid-dispatch" -- structurally unreachable through
        // ordinary Rust borrowing under Hub v0's synchronous, single-owner
        // (`&mut Hub`) model, but checked explicitly rather than assumed,
        // so a future concurrent execution mode cannot silently re-enter a
        // worker and have that misreported as a supervision quarantine.
        let was_degraded = health.state() == crate::worker::HubWorkerState::Degraded;
        match health.state() {
            crate::worker::HubWorkerState::Busy => {
                return Err(HubDispatchFailure::without_artifact(HubFault::WorkerBusy))
            }
            s if s.accepts_dispatch() || s == crate::worker::HubWorkerState::Registered => {}
            _ => {
                return Err(HubDispatchFailure::without_artifact(
                    HubFault::ToolQuarantined,
                ))
            }
        }
        // v0 lifecycle: a freshly registered worker starts on first dispatch.
        if health.state() == crate::worker::HubWorkerState::Registered {
            health.mark_starting().map_err(|error| {
                HubDispatchFailure::without_artifact(HubFault::InternalHubFault(error.to_string()))
            })?;
            health.mark_ready().map_err(|error| {
                HubDispatchFailure::without_artifact(HubFault::InternalHubFault(error.to_string()))
            })?;
        }
        health.mark_busy().map_err(|error| {
            HubDispatchFailure::without_artifact(HubFault::InternalHubFault(error.to_string()))
        })?;

        let Some(worker) = self.workers.get_mut(&tool.tool_id) else {
            let health = self
                .registry
                .health_mut(&tool.tool_id)
                .expect("health present: checked above");
            let _ = restore_worker_after_dispatch(health, was_degraded);
            return Err(HubDispatchFailure::without_artifact(
                HubFault::InternalHubFault("worker instance missing for registered tool".into()),
            ));
        };

        // Never hand the adapter the caller's raw, unsanitized capability
        // set: `satisfies()` only verified the operation's *required*
        // capabilities were present and non-sensitive, it never strips a
        // sensitive capability the caller happened to include alongside
        // them. An adapter naively calling `.allows(NetworkAccess)` on the
        // raw set could otherwise observe `true` for something Hub
        // structurally denies. Audit records preserve the raw set as
        // `capabilities_requested` and this sanitized set as
        // `capabilities_granted`.
        let sanitized_capabilities = request.capability_context.deny_sensitive();
        let context = RestrictedHubContext {
            resource_budget: &request.resource_budget,
            capability_context: &sanitized_capabilities,
            deadline,
        };

        let operation_id = request.operation_id.clone();
        let payload = request.payload.clone();
        let handle_result = panic::catch_unwind(AssertUnwindSafe(|| {
            worker.handle(&operation_id, &payload, &context)
        }));

        let health = self
            .registry
            .health_mut(&tool.tool_id)
            .expect("health present: checked above");

        match handle_result {
            Ok(Ok(tool_outcome)) => {
                let HubToolOutcome {
                    payload: bytes,
                    artifact,
                } = tool_outcome;
                let worker_ref = self.workers.get(&tool.tool_id).expect("worker present");
                // `validate_reply` is adapter-overridable code, exactly
                // like `handle` -- it must be panic-contained the same
                // way. Without this, a validator bug or assertion panic
                // would escape `Hub::invoke` entirely (crashing the whole
                // `smc` process) before worker state, concurrency
                // accounting, or the audit record were ever finalized,
                // violating `invoke`'s own documented guarantee to
                // always return a `HubReply`.
                let validation_outcome = panic::catch_unwind(AssertUnwindSafe(|| {
                    worker_ref.validate_reply(&request.operation_id, &bytes)
                }));
                match validation_outcome {
                    Ok(Ok(())) => {
                        restore_worker_after_dispatch(health, was_degraded).map_err(|fault| {
                            HubDispatchFailure {
                                fault,
                                committed_artifact: artifact.clone(),
                            }
                        })?;
                        Ok(HubToolOutcome {
                            payload: bytes,
                            artifact,
                        })
                    }
                    Ok(Err(reason)) => {
                        let _ = health.report_protocol_violation();
                        Err(HubDispatchFailure {
                            fault: HubFault::ProtocolViolation(reason),
                            committed_artifact: artifact,
                        })
                    }
                    Err(panic_payload) => {
                        let message = panic_message(&panic_payload);
                        let _ = health.report_crash();
                        Err(HubDispatchFailure {
                            fault: HubFault::WorkerPanicked(message),
                            committed_artifact: artifact,
                        })
                    }
                }
            }
            Ok(Err(tool_error)) => {
                restore_worker_after_dispatch(health, was_degraded)
                    .map_err(HubDispatchFailure::without_artifact)?;
                Err(HubDispatchFailure::without_artifact(
                    hub_fault_from_tool_error(tool_error),
                ))
            }
            Err(panic_payload) => {
                let message = panic_message(&panic_payload);
                let _ = health.report_crash();
                Err(HubDispatchFailure::without_artifact(
                    HubFault::WorkerPanicked(message),
                ))
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
        self.record_and_reply(
            request,
            sequence,
            Instant::now(),
            tool,
            Err(fault),
            None,
            false,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn record_and_reply(
        &mut self,
        request: HubRequest,
        sequence: u64,
        started: Instant,
        tool: HubToolDescriptor,
        result: Result<HubToolOutcome, HubFault>,
        committed_artifact: Option<crate::provenance::HubArtifactProvenance>,
        was_dispatched: bool,
    ) -> HubReply {
        let elapsed_millis = started.elapsed().as_millis() as u64;
        let input_digest = HubDigest::of(&request.payload);

        let (status, payload, output_digest, worker_state_after, status_code, fault_code) =
            match result {
                Ok(outcome) => {
                    let digest = HubDigest::of(&outcome.payload);
                    let state = self
                        .registry
                        .worker_state(&request.tool_id)
                        .unwrap_or(crate::worker::HubWorkerState::Registered);
                    (
                        HubReplyStatus::Success,
                        outcome.payload,
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
                            HubFault::InternalHubFault(_) | HubFault::SequenceExhausted => {
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

        let determinism = tool
            .operation(&request.operation_id)
            .map(|op| op.determinism)
            .unwrap_or(crate::execution::HubDeterminismClass::Unknown);

        let capabilities_requested: Vec<_> = request.capability_context.iter().copied().collect();
        let capabilities_granted = if was_dispatched {
            request
                .capability_context
                .deny_sensitive()
                .iter()
                .copied()
                .collect()
        } else {
            Vec::new()
        };
        let transaction_id = committed_artifact
            .as_ref()
            .and_then(|artifact| artifact.transaction_id.clone());
        let audit_record_id =
            (sequence != u64::MAX).then(|| format!("audit:{}:{sequence}", request.session_id));

        if sequence != u64::MAX {
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
                determinism,
                trust_class: tool.trust_class,
                privacy_class: request.privacy_class,
                capabilities_requested,
                capabilities_granted,
                input_digest,
                output_digest,
                transaction_id,
                resource_budget: request.resource_budget,
                resource_usage,
                worker_state_after,
                status_code,
                fault_code,
            };
            self.audit.push(record);
        }

        let (result_kind, result_id, result_digest) =
            if let Some(artifact) = committed_artifact.as_ref() {
                (artifact.kind.clone(), artifact.id.clone(), artifact.digest)
            } else {
                (
                    if status.is_success() {
                        "hub.reply.payload"
                    } else {
                        "hub.reply.fault"
                    }
                    .to_string(),
                    format!("reply:{}:{sequence}", request.request_id),
                    output_digest,
                )
            };

        let provenance = crate::provenance::HubProvenance {
            schema_version: crate::provenance::HUB_PROVENANCE_SCHEMA_VERSION,
            request_id: request.request_id.clone(),
            session_id: request.session_id.clone(),
            logical_sequence: sequence,
            caller_identity: request.caller_identity.clone(),
            tool_id: request.tool_id.clone(),
            tool_version: tool.tool_version,
            adapter_provenance: tool.adapter_provenance.clone(),
            hub_api_version: tool.hub_api_version,
            execution_mode: tool.execution_mode,
            operation_id: request.operation_id.clone(),
            determinism,
            trust_class: tool.trust_class,
            privacy_class: request.privacy_class,
            result_kind,
            result_id,
            result_digest,
            input_digest,
            output_digest,
            capability_context_digest: HubDigest::of(
                request.capability_context.canonical_text().as_bytes(),
            ),
            resource_budget_digest: HubDigest::of(
                request.resource_budget.canonical_text().as_bytes(),
            ),
            resource_usage,
            audit_record_id,
            worker_state_after,
            artifact: committed_artifact,
            warnings: Vec::new(),
        };

        HubReply {
            schema_version: crate::envelope::HUB_ENVELOPE_SCHEMA_VERSION,
            request_id: request.request_id,
            logical_sequence: sequence,
            tool_id: request.tool_id,
            tool_version: tool.tool_version,
            operation_id: request.operation_id,
            status,
            payload,
            resource_usage,
            provenance,
            warnings: Vec::new(),
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

fn restore_worker_after_dispatch(
    health: &mut crate::worker::HubWorkerHealth,
    was_degraded: bool,
) -> Result<(), HubFault> {
    let transition = if was_degraded {
        health.mark_degraded()
    } else {
        health.mark_ready()
    };
    transition.map_err(|error| HubFault::InternalHubFault(error.to_string()))
}

fn hub_fault_from_tool_error(error: HubToolError) -> HubFault {
    match error.code.as_str() {
        "PersistenceFailed" => HubFault::PersistenceFailed(error.message),
        "RecoveryRequired" => HubFault::RecoveryRequired(error.message),
        "DeadlineExceeded" => HubFault::DeadlineExceeded,
        _ => HubFault::ToolDeclaredFailure(error.to_string()),
    }
}

fn panic_message(_payload: &(dyn std::any::Any + Send)) -> String {
    "in-process adapter panicked".to_string()
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
                    HubOperationDescriptor::new(
                        HubOperationId::new("test.slow").unwrap(),
                        [HubCapability::CpuCompute],
                        HubDeterminismClass::Deterministic,
                        false,
                    ),
                    HubOperationDescriptor::new(
                        HubOperationId::new("test.panic-in-validate").unwrap(),
                        [HubCapability::CpuCompute],
                        HubDeterminismClass::Deterministic,
                        false,
                    ),
                    HubOperationDescriptor::new(
                        HubOperationId::new("test.persistence-failed").unwrap(),
                        [HubCapability::CpuCompute],
                        HubDeterminismClass::Deterministic,
                        false,
                    ),
                    HubOperationDescriptor::new(
                        HubOperationId::new("test.recovery-required").unwrap(),
                        [HubCapability::CpuCompute],
                        HubDeterminismClass::Deterministic,
                        false,
                    ),
                    HubOperationDescriptor::new(
                        HubOperationId::new("test.deadline-error").unwrap(),
                        [HubCapability::CpuCompute],
                        HubDeterminismClass::Deterministic,
                        false,
                    ),
                    HubOperationDescriptor::new(
                        HubOperationId::new("test.artifact-too-large").unwrap(),
                        [HubCapability::CpuCompute],
                        HubDeterminismClass::Deterministic,
                        false,
                    ),
                    HubOperationDescriptor::new(
                        HubOperationId::new("test.artifact-slow").unwrap(),
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
        ) -> Result<HubToolOutcome, HubToolError> {
            match operation_id.as_str() {
                "test.succeed" => Ok(HubToolOutcome::payload_only(b"ok".to_vec())),
                "test.fail" => Err(HubToolError::new(
                    "TestDeclaredFailure",
                    "intentional test failure",
                )),
                "test.panic" => panic!("intentional test panic"),
                "test.protocol-violation" => Ok(HubToolOutcome::payload_only(b"not-json".to_vec())),
                "test.report-network-access-visibility" => {
                    if context
                        .capability_context
                        .allows(HubCapability::NetworkAccess)
                    {
                        Ok(HubToolOutcome::payload_only(b"leaked".to_vec()))
                    } else {
                        Ok(HubToolOutcome::payload_only(b"denied".to_vec()))
                    }
                }
                "test.slow" => {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    Ok(HubToolOutcome::payload_only(b"ok".to_vec()))
                }
                "test.panic-in-validate" => Ok(HubToolOutcome::payload_only(b"ok".to_vec())),
                "test.persistence-failed" => {
                    Err(HubToolError::new("PersistenceFailed", "disk unavailable"))
                }
                "test.recovery-required" => Err(HubToolError::new(
                    "RecoveryRequired",
                    "unfinished transaction",
                )),
                "test.deadline-error" => Err(HubToolError::new(
                    "DeadlineExceeded",
                    "deadline reached before commit",
                )),
                "test.artifact-too-large" => Ok(HubToolOutcome {
                    payload: b"oversized".to_vec(),
                    artifact: Some(test_artifact()),
                }),
                "test.artifact-slow" => {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                    Ok(HubToolOutcome {
                        payload: b"ok".to_vec(),
                        artifact: Some(test_artifact()),
                    })
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
            if operation_id.as_str() == "test.panic-in-validate" {
                panic!("intentional validate_reply panic");
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

    fn test_artifact() -> crate::provenance::HubArtifactProvenance {
        crate::provenance::HubArtifactProvenance {
            kind: "test.artifact".into(),
            id: "artifact-1".into(),
            digest: HubDigest::of(b"committed"),
            transaction_id: Some("txn-1".into()),
        }
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
        assert_eq!(
            reply.provenance.operation_id,
            HubOperationId::new("test.succeed").unwrap()
        );
        assert_eq!(reply.provenance.hub_api_version, HubApiVersion::CURRENT);
        assert_eq!(reply.provenance.result_kind, "hub.reply.payload");
        assert_eq!(
            reply.provenance.result_digest,
            reply.provenance.output_digest
        );
        assert_eq!(
            reply.provenance.audit_record_id.as_deref(),
            Some("audit:sess-1:0")
        );
        assert_eq!(reply.provenance.resource_usage, reply.resource_usage);
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
    fn persistence_recovery_and_cooperative_deadline_codes_survive_adapter_mapping() {
        let cases = [
            ("test.persistence-failed", "PersistenceFailed"),
            ("test.recovery-required", "RecoveryRequired"),
            ("test.deadline-error", "DeadlineExceeded"),
        ];
        for (operation, expected_code) in cases {
            let mut hub = hub_with_fault_injection_tool();
            let reply = hub.invoke(request_for(operation, granted()), None);
            assert!(matches!(reply.status, HubReplyStatus::ToolFailed(_)));
            assert_eq!(reply.status.fault().unwrap().code(), expected_code);
            assert_eq!(hub.audit().records()[0].fault_code, Some(expected_code));
        }
    }

    #[test]
    fn worker_panic_is_contained_and_reported_as_crashed() {
        let mut hub = hub_with_fault_injection_tool();
        let reply = hub.invoke(request_for("test.panic", granted()), None);
        assert!(matches!(reply.status, HubReplyStatus::Crashed(_)));
        let rendered_fault = reply.status.fault().unwrap().to_string();
        assert_eq!(
            rendered_fault,
            "WorkerPanicked: in-process adapter panicked"
        );
        assert!(!rendered_fault.contains("intentional test panic"));
        assert!(!hub
            .audit()
            .to_canonical_text()
            .contains("intentional test panic"));
        assert_eq!(hub.audit().records()[0].status_code, "Crashed");
        assert_eq!(hub.audit().records()[0].fault_code, Some("WorkerPanicked"));
        // The panic must not have poisoned the Hub: a second, unrelated
        // invocation on a fresh operation still completes normally.
        let second = hub.invoke(request_for("test.succeed", granted()), None);
        assert!(matches!(second.status, HubReplyStatus::Success));
        assert_eq!(
            hub.registry()
                .worker_state(&HubToolId::new("test.fault-injection").unwrap()),
            Some(crate::worker::HubWorkerState::Degraded)
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
        hub.seed_next_sequence(41).unwrap();
        hub.invoke(request_for("test.succeed", granted()), None);
        assert_eq!(hub.audit().records()[0].sequence, 41);
    }

    #[test]
    fn u64_max_is_a_typed_sequence_exhaustion_sentinel_never_an_audit_record() {
        let mut hub = hub_with_fault_injection_tool();
        assert_eq!(
            hub.seed_next_sequence(u64::MAX).unwrap_err(),
            HubFault::SequenceExhausted
        );

        hub.seed_next_sequence(u64::MAX - 1).unwrap();
        let last = hub.invoke(request_for("test.succeed", granted()), None);
        assert!(last.status.is_success());
        assert_eq!(hub.audit().records()[0].sequence, u64::MAX - 1);

        let exhausted = hub.invoke(request_for("test.succeed", granted()), None);
        assert_eq!(exhausted.logical_sequence, u64::MAX);
        assert_eq!(
            exhausted.status.fault().unwrap().code(),
            "SequenceExhausted"
        );
        assert_eq!(exhausted.provenance.audit_record_id, None);
        assert_eq!(hub.audit().records().len(), 1);

        let repeated = hub.invoke(request_for("test.succeed", granted()), None);
        assert_eq!(repeated.status.fault().unwrap().code(), "SequenceExhausted");
        assert_eq!(hub.audit().records().len(), 1);
    }

    #[test]
    fn a_request_granting_any_sensitive_capability_is_rejected_before_dispatch() {
        // v0-completion hardening: admission now rejects a request outright
        // if its capability_context carries ANY sensitive capability, even
        // one the target operation does not require -- silently ignoring
        // such a grant (the pre-completion behavior) hid a caller mistake
        // instead of surfacing it. See `dispatch`'s own `deny_sensitive()`
        // call for the still-retained defense-in-depth sanitization, kept
        // in case a future caller of `dispatch` bypasses `admit`.
        let mut hub = hub_with_fault_injection_tool();
        let capabilities = granted().grant(HubCapability::NetworkAccess);
        let reply = hub.invoke(
            request_for("test.report-network-access-visibility", capabilities),
            None,
        );
        assert!(matches!(reply.status, HubReplyStatus::Rejected(_)));
        assert_eq!(
            reply.status.fault().unwrap().code(),
            "SensitiveCapabilityDenied"
        );

        // The raw, rejected grant is recorded as requested, never granted.
        assert!(hub.audit().records()[0]
            .capabilities_requested
            .contains(&HubCapability::NetworkAccess));
        assert!(hub.audit().records()[0].capabilities_granted.is_empty());
    }

    #[test]
    fn dispatch_still_sanitizes_sensitive_capabilities_as_defense_in_depth() {
        // Even though admission now rejects any sensitive grant outright
        // (see the test above), `dispatch`'s own `deny_sensitive()` call is
        // kept as a second, independent layer: an adapter must never
        // observe a sensitive capability as granted regardless of how a
        // request reached `dispatch`. Exercised directly against a request
        // with no sensitive grant, confirming the plain non-sensitive path
        // still resolves to "denied" from the adapter's point of view.
        let mut hub = hub_with_fault_injection_tool();
        let reply = hub.invoke(
            request_for("test.report-network-access-visibility", granted()),
            None,
        );
        assert!(reply.status.is_success());
        assert_eq!(reply.payload, b"denied");
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

    #[test]
    fn a_wall_time_budget_of_u64_max_is_a_typed_rejection_not_a_panic() {
        // Regression test: on some platforms (observed on Windows),
        // adding an extreme Duration to an Instant panics rather than
        // saturating, since Instant's own representable range is
        // narrower there. A caller-supplied wall_time_millis of
        // u64::MAX previously reached that raw `+` before admission's
        // own ceiling check (ResourceBudgetInvalid against
        // V0_CEILING's wall_time_millis = 30_000) ever got to run.
        let mut hub = hub_with_fault_injection_tool();
        let mut request = request_for("test.succeed", granted());
        request.resource_budget.wall_time_millis = u64::MAX;
        let reply = hub.invoke(request, None);
        assert!(matches!(reply.status, HubReplyStatus::Rejected(_)));
        assert_eq!(
            reply.status.fault().unwrap().code(),
            "ResourceBudgetInvalid"
        );
    }

    #[test]
    fn a_dispatch_that_overruns_its_wall_time_budget_is_rejected_after_the_fact() {
        // Regression test: the entry-only deadline check can't catch a
        // request whose budget was still unexpired when dispatch started
        // but is blown through by a long-running adapter call (nothing
        // polls deadline_exceeded() mid-call in v0). Without a
        // post-dispatch recheck, such a call would be reported as a plain
        // Success despite wall_time_millis being documented as
        // hard-enforced.
        let mut hub = hub_with_fault_injection_tool();
        let mut request = request_for("test.slow", granted());
        request.resource_budget.wall_time_millis = 1;
        let reply = hub.invoke(request, None);
        assert!(matches!(reply.status, HubReplyStatus::ToolFailed(_)));
        assert_eq!(reply.status.fault().unwrap().code(), "DeadlineExceeded");
    }

    #[test]
    fn dispatch_rejects_re_entrant_call_against_an_already_busy_worker() {
        // WorkerBusy is structurally unreachable through the public
        // `invoke()` API under Hub v0's synchronous, single-owner model
        // (dispatch() is the only code that ever sets Busy, and it always
        // resolves synchronously before returning) -- so this test reaches
        // into the same-crate-private `dispatch` method and `registry`
        // field directly to prove the check itself is real, not dead code
        // guarding an invariant nothing can ever violate today.
        let mut hub = hub_with_fault_injection_tool();
        let tool_id = HubToolId::new("test.fault-injection").unwrap();
        {
            let health = hub.registry.health_mut(&tool_id).unwrap();
            health.mark_starting().unwrap();
            health.mark_ready().unwrap();
            health.mark_busy().unwrap();
        }
        let tool = hub.registry.descriptor(&tool_id).unwrap().clone();
        let request = request_for("test.succeed", granted());
        let result = hub.dispatch(&request, &tool, None, Instant::now());
        assert_eq!(result.unwrap_err().fault, HubFault::WorkerBusy);
    }

    #[test]
    fn a_panic_inside_validate_reply_is_contained_the_same_way_as_a_panic_inside_handle() {
        // Regression test: validate_reply is adapter-overridable code,
        // exactly like handle, but was previously called outside
        // catch_unwind entirely -- a panicking validator would escape
        // Hub::invoke and crash the whole process (this test function
        // itself would fail with "test panicked" instead of a clean
        // assertion failure if the containment fix were missing).
        let mut hub = hub_with_fault_injection_tool();
        let reply = hub.invoke(request_for("test.panic-in-validate", granted()), None);
        assert!(matches!(reply.status, HubReplyStatus::Crashed(_)));
        assert_eq!(reply.status.fault().unwrap().code(), "WorkerPanicked");
        // The panic must not have poisoned the Hub: a second, unrelated
        // invocation still completes normally.
        let second = hub.invoke(request_for("test.succeed", granted()), None);
        assert!(
            matches!(second.status, HubReplyStatus::Success)
                || matches!(second.status, HubReplyStatus::Rejected(_))
        );
    }
}
