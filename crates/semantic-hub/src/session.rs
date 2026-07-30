//! Bounded, deterministic, synchronous multi-request Hub session executor.
//!
//! A `HubSession` proves the property a single `Hub::invoke` call cannot:
//! that one registered worker instance can safely process several ordered
//! requests, with a rolling whole-session ceiling attenuating every
//! individual request's own per-request budget, and with strict ordering
//! guarantees so replies and audit evidence can be correlated
//! deterministically. It adds no concurrency of its own -- `submit` is
//! synchronous and must be called once per request, in the exact order the
//! caller wants them admitted, matching Hub v0's synchronous,
//! single-owner (`&mut Hub`) execution model. No Tokio, no threads, no
//! async runtime: the ordering guarantee below falls out of the type
//! system (`&mut Hub` cannot be called concurrently) rather than being
//! independently enforced.
//!
//! ```text
//! input order = admission order = execution order = reply order
//!             = audit logical sequence order
//! ```
//!
//! Every request a `HubSession` submits still goes through exactly the
//! same [`crate::runtime::Hub::invoke_in_session`] admission/dispatch/audit
//! pipeline as a direct, session-less `Hub::invoke` call -- this module
//! adds ceiling attenuation and bookkeeping around that one pipeline, it
//! does not create a second one.

use std::collections::BTreeSet;
use std::time::Instant;

pub use crate::admission::HubSessionCeiling;
use crate::admission::SessionAdmissionAmbient;
use crate::capability::HubCapabilitySet;
use crate::envelope::{HubReply, HubReplyStatus, HubRequest};
use crate::execution::HubPrivacyClass;
use crate::ids::{HubCallerIdentity, HubRequestId, HubSessionId};
use crate::runtime::Hub;

/// Final, read-only accounting for one session -- everything a caller
/// needs to report a session summary without re-deriving it from the
/// individual replies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HubSessionSummary {
    pub session_id: HubSessionId,
    pub caller_identity: HubCallerIdentity,
    pub capability_ceiling: HubCapabilitySet,
    pub privacy_ceiling: HubPrivacyClass,
    pub ceiling: HubSessionCeiling,
    /// Every `submit` call, regardless of outcome.
    pub requests_submitted: u32,
    /// Requests that passed admission (dispatched, whether they then
    /// succeeded, failed, or crashed) -- distinct from a pre-dispatch
    /// rejection, matching `HubReplyStatus`'s own `Rejected` vs.
    /// `Success`/`ToolFailed`/`Crashed`/`HubFault` distinction.
    pub requests_admitted: u32,
    pub requests_rejected_pre_dispatch: u32,
    /// Sums of each reply's own measured `resource_usage` -- never
    /// fabricated: a reply whose usage field was `None` contributes
    /// nothing, it is not treated as zero.
    pub cumulative_input_bytes: u64,
    pub cumulative_output_bytes: u64,
    pub cumulative_wall_time_millis: u64,
    pub first_logical_sequence: Option<u64>,
    pub last_logical_sequence: Option<u64>,
}

/// A bounded, ordered, synchronous multi-request session against one
/// [`Hub`]. Construct once per logical batch (e.g. one `smc hub session`
/// CLI invocation), then call [`Self::submit`] once per request in the
/// exact order they should be admitted.
pub struct HubSession {
    session_id: HubSessionId,
    caller_identity: HubCallerIdentity,
    capability_ceiling: HubCapabilitySet,
    privacy_ceiling: HubPrivacyClass,
    ceiling: HubSessionCeiling,
    requests_submitted: u32,
    requests_admitted: u32,
    requests_rejected_pre_dispatch: u32,
    cumulative_input_bytes: u64,
    cumulative_output_bytes: u64,
    cumulative_wall_time_millis: u64,
    first_logical_sequence: Option<u64>,
    last_logical_sequence: Option<u64>,
    /// Request ids the caller has asked to treat as pre-cancelled. Session
    /// v0's cancellation model is cooperative and pre-admission-only: a
    /// request whose id is in this set when `submit` runs for it is
    /// rejected with `Cancelled` before ever reaching dispatch. There is
    /// no mechanism to cancel a request already admitted/dispatched (v0 is
    /// synchronous -- by the time `submit` returns, the request has
    /// already fully completed or failed).
    cancelled_request_ids: BTreeSet<HubRequestId>,
}

impl HubSession {
    pub fn new(
        session_id: HubSessionId,
        caller_identity: HubCallerIdentity,
        capability_ceiling: HubCapabilitySet,
        privacy_ceiling: HubPrivacyClass,
        ceiling: HubSessionCeiling,
    ) -> Self {
        Self {
            session_id,
            caller_identity,
            capability_ceiling,
            privacy_ceiling,
            ceiling,
            requests_submitted: 0,
            requests_admitted: 0,
            requests_rejected_pre_dispatch: 0,
            cumulative_input_bytes: 0,
            cumulative_output_bytes: 0,
            cumulative_wall_time_millis: 0,
            first_logical_sequence: None,
            last_logical_sequence: None,
            cancelled_request_ids: BTreeSet::new(),
        }
    }

    pub fn session_id(&self) -> &HubSessionId {
        &self.session_id
    }

    pub fn ceiling(&self) -> HubSessionCeiling {
        self.ceiling
    }

    pub fn caller_identity(&self) -> &HubCallerIdentity {
        &self.caller_identity
    }

    pub fn capability_ceiling(&self) -> &HubCapabilitySet {
        &self.capability_ceiling
    }

    pub fn privacy_ceiling(&self) -> HubPrivacyClass {
        self.privacy_ceiling
    }

    /// Mark a request id as cancelled before it is submitted. Has no
    /// effect on a request already submitted (v0's cancellation is
    /// pre-admission-only and cooperative -- see the struct-level doc
    /// comment). Calling this after the id has already been submitted is
    /// a harmless no-op, not an error: the caller may not know submission
    /// order in advance when building a cancel list from an external
    /// source.
    pub fn cancel(&mut self, request_id: HubRequestId) {
        self.cancelled_request_ids.insert(request_id);
    }

    /// Submit one request through this session's ceiling and the given
    /// `Hub`, in the same admission/dispatch/audit pipeline as a direct
    /// `Hub::invoke` call. Must be called once per request, synchronously,
    /// in the exact order the caller wants them admitted -- the session's
    /// own ordering guarantee (input order = admission order = execution
    /// order = reply order = audit logical sequence order) depends on
    /// this, not on any ordering enforced independently by this type.
    pub fn submit(
        &mut self,
        hub: &mut Hub,
        request: HubRequest,
        deadline: Option<Instant>,
    ) -> HubReply {
        // Session identity is enforced by construction, not validated as
        // a new caller-facing error path: every request submitted through
        // this session carries exactly this session's id, regardless of
        // what the caller happened to set on the `HubRequest` itself.
        let mut request = request;
        request.session_id = self.session_id.clone();
        request.resource_budget.input_bytes = request.resource_budget.input_bytes.min(
            self.ceiling
                .max_cumulative_input_bytes
                .saturating_sub(self.cumulative_input_bytes),
        );
        request.resource_budget.output_bytes = request.resource_budget.output_bytes.min(
            self.ceiling
                .max_cumulative_output_bytes
                .saturating_sub(self.cumulative_output_bytes),
        );
        request.resource_budget.wall_time_millis = request.resource_budget.wall_time_millis.min(
            self.ceiling
                .max_cumulative_wall_time_millis
                .saturating_sub(self.cumulative_wall_time_millis),
        );
        request.resource_budget.queue_depth = request
            .resource_budget
            .queue_depth
            .min(self.ceiling.max_queue_depth);
        request.resource_budget.concurrent_requests = request
            .resource_budget
            .concurrent_requests
            .min(self.ceiling.max_concurrent_requests);

        let already_cancelled = self.cancelled_request_ids.contains(&request.request_id);
        let ambient = SessionAdmissionAmbient {
            ceiling: self.ceiling,
            caller_identity: self.caller_identity.clone(),
            capability_ceiling: self.capability_ceiling.clone(),
            privacy_ceiling: self.privacy_ceiling,
            requests_submitted_so_far: self.requests_submitted,
            cumulative_input_bytes_so_far: self.cumulative_input_bytes,
            cumulative_output_bytes_so_far: self.cumulative_output_bytes,
            cumulative_wall_time_millis_so_far: self.cumulative_wall_time_millis,
        };

        self.requests_submitted = self.requests_submitted.saturating_add(1);
        let reply = hub.invoke_in_session(request, deadline, already_cancelled, ambient);

        let admitted = match &reply.status {
            HubReplyStatus::Rejected(_) => {
                self.requests_rejected_pre_dispatch =
                    self.requests_rejected_pre_dispatch.saturating_add(1);
                false
            }
            _ => {
                self.requests_admitted = self.requests_admitted.saturating_add(1);
                true
            }
        };

        // Cumulative usage is derived only from what the reply itself
        // measured -- a `None` field contributes nothing, never a
        // fabricated zero or an assumed worst case.
        if let Some(bytes) = reply.resource_usage.input_bytes {
            self.cumulative_input_bytes = self.cumulative_input_bytes.saturating_add(bytes);
        }
        if let Some(bytes) = reply.resource_usage.output_bytes {
            self.cumulative_output_bytes = self.cumulative_output_bytes.saturating_add(bytes);
        }
        if let Some(millis) = reply.resource_usage.wall_time_millis {
            self.cumulative_wall_time_millis =
                self.cumulative_wall_time_millis.saturating_add(millis);
        }

        if admitted {
            if self.first_logical_sequence.is_none() {
                self.first_logical_sequence = Some(reply.logical_sequence);
            }
            self.last_logical_sequence = Some(reply.logical_sequence);
        }

        reply
    }

    pub fn summary(&self) -> HubSessionSummary {
        HubSessionSummary {
            session_id: self.session_id.clone(),
            caller_identity: self.caller_identity.clone(),
            capability_ceiling: self.capability_ceiling.clone(),
            privacy_ceiling: self.privacy_ceiling,
            ceiling: self.ceiling,
            requests_submitted: self.requests_submitted,
            requests_admitted: self.requests_admitted,
            requests_rejected_pre_dispatch: self.requests_rejected_pre_dispatch,
            cumulative_input_bytes: self.cumulative_input_bytes,
            cumulative_output_bytes: self.cumulative_output_bytes,
            cumulative_wall_time_millis: self.cumulative_wall_time_millis,
            first_logical_sequence: self.first_logical_sequence,
            last_logical_sequence: self.last_logical_sequence,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::{HubCapability, HubCapabilitySet};
    use crate::descriptor::HubOperationDescriptor;
    use crate::execution::{HubDeterminismClass, HubExecutionMode, HubPrivacyClass, HubTrustClass};
    use crate::fault::HubToolError;
    use crate::ids::{HubApiVersion, HubCallerIdentity, HubOperationId, HubToolId, HubToolVersion};
    use crate::resource::HubResourceBudget;
    use crate::runtime::{HubTool, HubToolOutcome, RestrictedHubContext};

    /// A minimal counting tool: each call echoes back how many times it
    /// has been invoked, so tests can assert real ordering/sequencing
    /// without depending on TurboVec or any I/O.
    struct CountingTool {
        descriptor: crate::descriptor::HubToolDescriptor,
        calls: u32,
    }

    impl CountingTool {
        fn new() -> Self {
            let descriptor = crate::descriptor::HubToolDescriptor {
                tool_id: HubToolId::new("test.counter").unwrap(),
                name: "counter".into(),
                tool_version: HubToolVersion::new(1, 0, 0),
                hub_api_version: HubApiVersion::CURRENT,
                execution_mode: HubExecutionMode::InProcess,
                trust_class: HubTrustClass::InProcessUnisolated,
                operations: vec![HubOperationDescriptor::new(
                    HubOperationId::new("test.count").unwrap(),
                    [HubCapability::CpuCompute],
                    HubDeterminismClass::Deterministic,
                    true,
                )],
                resource_ceiling: HubResourceBudget::V0_CEILING,
                adapter_provenance: "test-only counting tool".into(),
            };
            Self {
                descriptor,
                calls: 0,
            }
        }
    }

    impl HubTool for CountingTool {
        fn descriptor(&self) -> &crate::descriptor::HubToolDescriptor {
            &self.descriptor
        }

        fn handle(
            &mut self,
            _operation_id: &HubOperationId,
            _payload: &[u8],
            _context: &RestrictedHubContext,
        ) -> Result<HubToolOutcome, HubToolError> {
            self.calls += 1;
            Ok(HubToolOutcome::payload_only(
                self.calls.to_string().into_bytes(),
            ))
        }
    }

    fn hub_with_counter() -> Hub {
        let mut hub = Hub::new();
        hub.register_tool(Box::new(CountingTool::new())).unwrap();
        hub
    }

    fn session(session_id: &str, ceiling: HubSessionCeiling) -> HubSession {
        HubSession::new(
            HubSessionId::new(session_id).unwrap(),
            HubCallerIdentity::new("cli:local").unwrap(),
            HubCapabilitySet::empty().grant(HubCapability::CpuCompute),
            HubPrivacyClass::ProjectLocal,
            ceiling,
        )
    }

    fn request(session_id: &str, request_id: &str) -> HubRequest {
        HubRequest {
            schema_version: crate::envelope::HUB_ENVELOPE_SCHEMA_VERSION,
            api_version: HubApiVersion::CURRENT,
            request_id: HubRequestId::new(request_id).unwrap(),
            session_id: HubSessionId::new(session_id).unwrap(),
            caller_identity: HubCallerIdentity::new("cli:local").unwrap(),
            tool_id: HubToolId::new("test.counter").unwrap(),
            operation_id: HubOperationId::new("test.count").unwrap(),
            capability_context: HubCapabilitySet::empty().grant(HubCapability::CpuCompute),
            privacy_class: HubPrivacyClass::ProjectLocal,
            resource_budget: HubResourceBudget::V0_CEILING,
            payload: b"{}".to_vec(),
        }
    }

    #[test]
    fn a_session_processes_multiple_requests_against_one_persistent_worker() {
        let mut hub = hub_with_counter();
        let mut session = session("sess-a", HubSessionCeiling::V0_DEFAULT);
        let r1 = session.submit(&mut hub, request("sess-a", "req-1"), None);
        let r2 = session.submit(&mut hub, request("sess-a", "req-2"), None);
        let r3 = session.submit(&mut hub, request("sess-a", "req-3"), None);

        // The SAME worker instance handled all three -- its private call
        // counter incremented across the session, proving persistence
        // rather than a fresh worker per request.
        assert_eq!(r1.payload, b"1");
        assert_eq!(r2.payload, b"2");
        assert_eq!(r3.payload, b"3");
    }

    #[test]
    fn logical_sequence_is_strictly_increasing_in_submission_order() {
        let mut hub = hub_with_counter();
        let mut session = session("sess-b", HubSessionCeiling::V0_DEFAULT);
        let replies: Vec<HubReply> = (0..5)
            .map(|i| session.submit(&mut hub, request("sess-b", &format!("req-{i}")), None))
            .collect();
        let sequences: Vec<u64> = replies.iter().map(|r| r.logical_sequence).collect();
        let mut sorted = sequences.clone();
        sorted.sort_unstable();
        assert_eq!(
            sequences, sorted,
            "sequence must already be sorted (strictly increasing)"
        );
        for w in sequences.windows(2) {
            assert!(
                w[1] > w[0],
                "sequence must be strictly increasing, got {sequences:?}"
            );
        }
    }

    #[test]
    fn session_request_id_overwrites_whatever_the_caller_set_on_the_request() {
        let mut hub = hub_with_counter();
        let mut session = session("sess-c", HubSessionCeiling::V0_DEFAULT);
        let mut req = request("wrong-session-id", "req-1");
        req.session_id = HubSessionId::new("wrong-session-id").unwrap();
        session.submit(&mut hub, req, None);
        // No direct observable on HubReply for session_id, but the audit
        // trail records what was actually admitted.
        assert_eq!(
            hub.audit().records()[0].session_id,
            HubSessionId::new("sess-c").unwrap()
        );
    }

    #[test]
    fn exceeding_the_session_request_count_ceiling_rejects_further_requests() {
        let mut hub = hub_with_counter();
        let ceiling = HubSessionCeiling {
            max_request_count: 2,
            ..HubSessionCeiling::V0_DEFAULT
        };
        let mut session = session("sess-d", ceiling);
        let r1 = session.submit(&mut hub, request("sess-d", "req-1"), None);
        let r2 = session.submit(&mut hub, request("sess-d", "req-2"), None);
        let r3 = session.submit(&mut hub, request("sess-d", "req-3"), None);
        assert!(r1.status.is_success());
        assert!(r2.status.is_success());
        assert!(matches!(r3.status, HubReplyStatus::Rejected(_)));
        assert_eq!(r3.status.fault().unwrap().code(), "SessionLimitExceeded");

        let summary = session.summary();
        assert_eq!(summary.requests_submitted, 3);
        assert_eq!(summary.requests_admitted, 2);
        assert_eq!(summary.requests_rejected_pre_dispatch, 1);
    }

    #[test]
    fn exceeding_the_session_cumulative_input_bytes_ceiling_rejects_further_requests() {
        let mut hub = hub_with_counter();
        let mut req1 = request("sess-e", "req-1");
        req1.payload = vec![0u8; 100];
        let ceiling = HubSessionCeiling {
            max_cumulative_input_bytes: 150,
            ..HubSessionCeiling::V0_DEFAULT
        };
        let mut session = session("sess-e", ceiling);
        let r1 = session.submit(&mut hub, req1, None);
        assert!(r1.status.is_success());

        let mut req2 = request("sess-e", "req-2");
        req2.payload = vec![0u8; 100]; // cumulative would be 200 > 150
        let r2 = session.submit(&mut hub, req2, None);
        assert!(matches!(r2.status, HubReplyStatus::Rejected(_)));
        assert_eq!(r2.status.fault().unwrap().code(), "SessionLimitExceeded");
    }

    #[test]
    fn session_identity_is_fixed_and_rejected_before_dispatch() {
        let mut hub = hub_with_counter();
        let mut session = session("sess-identity", HubSessionCeiling::V0_DEFAULT);
        let mut wrong_caller = request("sess-identity", "req-wrong");
        wrong_caller.caller_identity = HubCallerIdentity::new("cli:other").unwrap();

        let rejected = session.submit(&mut hub, wrong_caller, None);
        let admitted = session.submit(&mut hub, request("sess-identity", "req-admitted"), None);

        assert!(matches!(rejected.status, HubReplyStatus::Rejected(_)));
        assert_eq!(rejected.status.fault().unwrap().code(), "InputRejected");
        assert_eq!(admitted.payload, b"1", "rejection must not dispatch");
        let summary = session.summary();
        assert_eq!(
            summary.first_logical_sequence,
            Some(admitted.logical_sequence)
        );
        assert_eq!(
            summary.last_logical_sequence,
            Some(admitted.logical_sequence)
        );
    }

    #[test]
    fn session_capability_and_privacy_ceilings_cannot_be_widened() {
        let mut hub = hub_with_counter();
        let mut session = session("sess-policy", HubSessionCeiling::V0_DEFAULT);

        let mut widened_capability = request("sess-policy", "req-capability");
        widened_capability.capability_context = widened_capability
            .capability_context
            .grant(HubCapability::VectorSearch);
        let capability_reply = session.submit(&mut hub, widened_capability, None);
        assert!(matches!(
            capability_reply.status,
            HubReplyStatus::Rejected(_)
        ));
        assert_eq!(
            capability_reply.status.fault().unwrap().code(),
            "CapabilityDenied"
        );

        let mut widened_privacy = request("sess-policy", "req-privacy");
        widened_privacy.privacy_class = HubPrivacyClass::PrivateSource;
        let privacy_reply = session.submit(&mut hub, widened_privacy, None);
        assert!(matches!(privacy_reply.status, HubReplyStatus::Rejected(_)));
        assert_eq!(
            privacy_reply.status.fault().unwrap().code(),
            "InputRejected"
        );

        let admitted = session.submit(&mut hub, request("sess-policy", "req-ok"), None);
        assert_eq!(admitted.payload, b"1", "neither rejection may dispatch");
    }

    #[test]
    fn rejected_requests_still_consume_the_bounded_intake_count() {
        let mut hub = hub_with_counter();
        let ceiling = HubSessionCeiling {
            max_request_count: 1,
            ..HubSessionCeiling::V0_DEFAULT
        };
        let mut session = session("sess-intake", ceiling);
        let mut wrong_caller = request("sess-intake", "req-wrong");
        wrong_caller.caller_identity = HubCallerIdentity::new("cli:other").unwrap();
        let mut another_wrong_caller = request("sess-intake", "req-next");
        another_wrong_caller.caller_identity = HubCallerIdentity::new("cli:other").unwrap();

        let first = session.submit(&mut hub, wrong_caller, None);
        let second = session.submit(&mut hub, another_wrong_caller, None);

        assert_eq!(first.status.fault().unwrap().code(), "InputRejected");
        assert_eq!(
            second.status.fault().unwrap().code(),
            "SessionLimitExceeded"
        );
        let summary = session.summary();
        assert_eq!(summary.requests_submitted, 2);
        assert_eq!(summary.requests_admitted, 0);
        assert_eq!(summary.first_logical_sequence, None);
        assert_eq!(summary.last_logical_sequence, None);
    }

    #[test]
    fn session_output_budget_is_attenuated_to_the_remaining_ceiling() {
        let mut hub = hub_with_counter();
        let ceiling = HubSessionCeiling {
            max_cumulative_output_bytes: 1,
            ..HubSessionCeiling::V0_DEFAULT
        };
        let mut session = session("sess-output", ceiling);

        let first = session.submit(&mut hub, request("sess-output", "req-1"), None);
        let second = session.submit(&mut hub, request("sess-output", "req-2"), None);

        assert!(first.status.is_success());
        assert_eq!(first.payload.len(), 1);
        assert_eq!(second.status.fault().unwrap().code(), "OutputRejected");
        assert!(session.summary().cumulative_output_bytes <= ceiling.max_cumulative_output_bytes);
    }

    #[test]
    fn zero_session_queue_or_concurrency_ceiling_rejects_before_dispatch() {
        let mut hub = hub_with_counter();
        let mut queue_limited = session(
            "sess-queue",
            HubSessionCeiling {
                max_queue_depth: 0,
                ..HubSessionCeiling::V0_DEFAULT
            },
        );
        let queue_reply = queue_limited.submit(&mut hub, request("sess-queue", "req-queue"), None);
        assert_eq!(queue_reply.status.fault().unwrap().code(), "QueueFull");

        let mut concurrency_limited = session(
            "sess-concurrency",
            HubSessionCeiling {
                max_concurrent_requests: 0,
                ..HubSessionCeiling::V0_DEFAULT
            },
        );
        let concurrency_reply = concurrency_limited.submit(
            &mut hub,
            request("sess-concurrency", "req-concurrency"),
            None,
        );
        assert_eq!(
            concurrency_reply.status.fault().unwrap().code(),
            "QueueFull"
        );
    }

    #[test]
    fn a_pre_cancelled_request_id_is_rejected_with_cancelled_before_dispatch() {
        let mut hub = hub_with_counter();
        let mut session = session("sess-f", HubSessionCeiling::V0_DEFAULT);
        session.cancel(HubRequestId::new("req-2").unwrap());

        let r1 = session.submit(&mut hub, request("sess-f", "req-1"), None);
        let r2 = session.submit(&mut hub, request("sess-f", "req-2"), None);
        let r3 = session.submit(&mut hub, request("sess-f", "req-3"), None);

        assert!(r1.status.is_success());
        assert!(matches!(r2.status, HubReplyStatus::Rejected(_)));
        assert_eq!(r2.status.fault().unwrap().code(), "Cancelled");
        assert!(r3.status.is_success());
    }

    #[test]
    fn summary_reports_only_measured_usage_never_a_fabricated_zero() {
        let mut hub = hub_with_counter();
        let mut session = session("sess-g", HubSessionCeiling::V0_DEFAULT);
        session.submit(&mut hub, request("sess-g", "req-1"), None);
        let summary = session.summary();
        // input_bytes/output_bytes/wall_time_millis are always populated
        // by record_and_reply for every invocation in this crate's own
        // implementation, so this asserts the summary actually reflects
        // them rather than staying at zero by omission.
        assert!(summary.cumulative_input_bytes > 0);
        assert_eq!(
            summary.first_logical_sequence,
            summary.last_logical_sequence
        );
    }

    #[test]
    fn independent_sessions_against_the_same_hub_do_not_share_ceiling_state() {
        // Two independent sessions against the SAME Hub do not share
        // ceiling state -- each session's cumulative counters start fresh.
        let mut hub = hub_with_counter();
        let ceiling = HubSessionCeiling {
            max_request_count: 1,
            ..HubSessionCeiling::V0_DEFAULT
        };
        let mut session_a = session("sess-h1", ceiling);
        let mut session_b = session("sess-h2", ceiling);

        let a1 = session_a.submit(&mut hub, request("sess-h1", "req-a1"), None);
        let b1 = session_b.submit(&mut hub, request("sess-h2", "req-b1"), None);
        assert!(a1.status.is_success());
        assert!(
            b1.status.is_success(),
            "session B's own ceiling must not have been consumed by session A"
        );
    }
}
