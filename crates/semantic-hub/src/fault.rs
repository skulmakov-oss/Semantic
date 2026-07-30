//! Fault taxonomy. Distinguishes request rejection, tool-declared failure,
//! worker crash/protocol violation, and Hub-internal fault -- these must
//! never be conflated into one generic "error" bucket.

use std::fmt;

use crate::resource::HubBudgetExceeded;

/// Stable, exhaustive Hub-level fault taxonomy. Every admitted-or-rejected
/// invocation ends in exactly one status
/// ([`crate::envelope::HubReplyStatus`]) carrying at most one `HubFault`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HubFault {
    /// Rejected before admission completed.
    UnknownTool,
    UnknownOperation,
    ApiVersionUnsupported,
    SchemaVersionUnsupported,
    DescriptorIncompatible,
    InputRejected(String),
    CapabilityDenied(String),
    PrivacyDenied(String),
    ResourceBudgetInvalid(HubBudgetExceeded),
    QueueFull,
    ToolDisabled,
    ToolQuarantined,

    /// The request's capability context carries a sensitive capability
    /// (e.g. `NetworkAccess`) that no v0 tool may ever be granted -- even
    /// though `deny_sensitive()` strips it before the adapter ever sees
    /// it, silently ignoring the caller's request to grant it hides a
    /// caller mistake instead of surfacing it plainly. Distinct from
    /// `CapabilityDenied`, which is a *missing required* capability.
    SensitiveCapabilityDenied(String),

    /// A mutating operation was rejected because the worker is currently
    /// `Degraded` (elevated crash count, not yet restarted/quarantined).
    /// Read-only operations still proceed against a degraded worker --
    /// only further state mutation is held back, to limit how much
    /// damage a flaky worker can still do before supervision resolves it.
    WorkerDegraded,

    /// The session-level cumulative ceiling (request count, cumulative
    /// input/output bytes, cumulative wall time) was exceeded. Distinct
    /// from `ResourceBudgetInvalid`/`ResourceExhausted`, which are
    /// per-request, not per-session.
    SessionLimitExceeded(HubBudgetExceeded),

    /// Rejected/terminated during or after dispatch.
    DeadlineExceeded,
    Cancelled,
    ResourceExhausted(HubBudgetExceeded),

    /// A dispatch was attempted while the worker's own health state was
    /// already `Busy` -- an invariant violation under Hub v0's
    /// synchronous, single-owner (`&mut Hub`) execution model, where two
    /// overlapping dispatches to the same worker cannot occur through
    /// ordinary Rust borrowing. Kept as a real, checked rejection (not
    /// an assertion) so a future concurrent execution mode cannot silently
    /// re-enter a worker that is already mid-dispatch.
    WorkerBusy,

    /// The tool ran and declared its own failure -- distinct from a crash.
    ToolDeclaredFailure(String),

    /// The adapter could not durably persist an operation's state.
    PersistenceFailed(String),

    /// Persistent state cannot be used until its interrupted transaction is
    /// explicitly recovered.
    RecoveryRequired(String),

    /// The worker panicked or otherwise crashed; contained by the
    /// supervisor, not propagated as a process crash.
    WorkerPanicked(String),

    /// The worker returned something that did not conform to the expected
    /// reply schema -- distinct from a tool-declared failure.
    ProtocolViolation(String),

    /// Output failed post-dispatch structural/bounds validation.
    OutputRejected(String),

    /// Audit or provenance recording itself failed. Per policy this must
    /// never be silently swallowed into an apparent success.
    AuditProvenanceFailure(String),

    /// A fault in the Hub's own admission/dispatch/registry logic, not
    /// attributable to caller input or the tool.
    InternalHubFault(String),

    /// The monotonic audit sequence has no representable next value.
    SequenceExhausted,
}

impl HubFault {
    /// Stable machine-readable code, independent of `Display` wording, for
    /// CLI exit-code mapping and audit records.
    pub const fn code(&self) -> &'static str {
        match self {
            HubFault::UnknownTool => "UnknownTool",
            HubFault::UnknownOperation => "UnknownOperation",
            HubFault::ApiVersionUnsupported => "ApiVersionUnsupported",
            HubFault::SchemaVersionUnsupported => "SchemaVersionUnsupported",
            HubFault::DescriptorIncompatible => "DescriptorIncompatible",
            HubFault::InputRejected(_) => "InputRejected",
            HubFault::CapabilityDenied(_) => "CapabilityDenied",
            HubFault::PrivacyDenied(_) => "PrivacyDenied",
            HubFault::ResourceBudgetInvalid(_) => "ResourceBudgetInvalid",
            HubFault::QueueFull => "QueueFull",
            HubFault::ToolDisabled => "ToolDisabled",
            HubFault::ToolQuarantined => "ToolQuarantined",
            HubFault::SensitiveCapabilityDenied(_) => "SensitiveCapabilityDenied",
            HubFault::WorkerDegraded => "WorkerDegraded",
            HubFault::SessionLimitExceeded(_) => "SessionLimitExceeded",
            HubFault::DeadlineExceeded => "DeadlineExceeded",
            HubFault::Cancelled => "Cancelled",
            HubFault::ResourceExhausted(_) => "ResourceExhausted",
            HubFault::WorkerBusy => "WorkerBusy",
            HubFault::ToolDeclaredFailure(_) => "ToolDeclaredFailure",
            HubFault::PersistenceFailed(_) => "PersistenceFailed",
            HubFault::RecoveryRequired(_) => "RecoveryRequired",
            HubFault::WorkerPanicked(_) => "WorkerPanicked",
            HubFault::ProtocolViolation(_) => "ProtocolViolation",
            HubFault::OutputRejected(_) => "OutputRejected",
            HubFault::AuditProvenanceFailure(_) => "AuditProvenanceFailure",
            HubFault::InternalHubFault(_) => "InternalHubFault",
            HubFault::SequenceExhausted => "SequenceExhausted",
        }
    }

    /// Whether this fault means the request never reached tool dispatch.
    pub const fn is_pre_dispatch_rejection(&self) -> bool {
        matches!(
            self,
            HubFault::UnknownTool
                | HubFault::UnknownOperation
                | HubFault::ApiVersionUnsupported
                | HubFault::SchemaVersionUnsupported
                | HubFault::DescriptorIncompatible
                | HubFault::InputRejected(_)
                | HubFault::CapabilityDenied(_)
                | HubFault::PrivacyDenied(_)
                | HubFault::ResourceBudgetInvalid(_)
                | HubFault::QueueFull
                | HubFault::ToolDisabled
                | HubFault::ToolQuarantined
                | HubFault::SensitiveCapabilityDenied(_)
                | HubFault::WorkerDegraded
                | HubFault::SessionLimitExceeded(_)
                // `Cancelled` is listed in the enum among "rejected/
                // terminated during or after dispatch" faults because a
                // future execution mode may need to observe cancellation
                // mid-dispatch, but Hub v0's only actual producer of this
                // fault is `admission::admit`'s already-cancelled check
                // (step 1, before the registry is even consulted) -- so it
                // belongs here today. Found via
                // `session::tests::a_pre_cancelled_request_id_is_rejected_with_cancelled_before_dispatch`:
                // without this, a session-cancelled request was reported
                // as `ToolFailed(Cancelled)` even though admission never
                // let it reach dispatch at all.
                | HubFault::Cancelled
        )
    }
}

impl fmt::Display for HubFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HubFault::InputRejected(m)
            | HubFault::CapabilityDenied(m)
            | HubFault::PrivacyDenied(m)
            | HubFault::SensitiveCapabilityDenied(m)
            | HubFault::ToolDeclaredFailure(m)
            | HubFault::PersistenceFailed(m)
            | HubFault::RecoveryRequired(m)
            | HubFault::WorkerPanicked(m)
            | HubFault::ProtocolViolation(m)
            | HubFault::OutputRejected(m)
            | HubFault::AuditProvenanceFailure(m)
            | HubFault::InternalHubFault(m) => write!(f, "{}: {m}", self.code()),
            HubFault::ResourceBudgetInvalid(e)
            | HubFault::ResourceExhausted(e)
            | HubFault::SessionLimitExceeded(e) => {
                write!(f, "{}: {e}", self.code())
            }
            _ => f.write_str(self.code()),
        }
    }
}

impl std::error::Error for HubFault {}

/// An error a tool adapter declares about its own operation. Kept distinct
/// from `HubFault` so the Hub can wrap it as
/// `HubFault::ToolDeclaredFailure` without adapters needing to know the
/// full Hub fault taxonomy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HubToolError {
    pub code: String,
    pub message: String,
}

impl HubToolError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for HubToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_is_stable_and_independent_of_payload_wording() {
        assert_eq!(
            HubFault::InputRejected("anything".into()).code(),
            "InputRejected"
        );
        assert_eq!(
            HubFault::InputRejected("different wording".into()).code(),
            "InputRejected"
        );
    }

    #[test]
    fn pre_dispatch_rejections_are_classified_distinctly_from_post_dispatch() {
        assert!(HubFault::UnknownTool.is_pre_dispatch_rejection());
        assert!(HubFault::QueueFull.is_pre_dispatch_rejection());
        assert!(!HubFault::WorkerPanicked("boom".into()).is_pre_dispatch_rejection());
        assert!(!HubFault::DeadlineExceeded.is_pre_dispatch_rejection());
    }

    #[test]
    fn cancelled_is_a_pre_dispatch_rejection() {
        // `admission::admit` is the only Hub v0 code path that ever
        // constructs `HubFault::Cancelled`, and it does so at step 1,
        // before dispatch is ever reached -- `HubReplyStatus` must report
        // it as `Rejected`, matching every other admission-time fault,
        // not `ToolFailed`.
        assert!(HubFault::Cancelled.is_pre_dispatch_rejection());
    }

    #[test]
    fn tool_declared_failure_is_distinct_from_worker_panic() {
        let declared = HubFault::ToolDeclaredFailure("bad input".into());
        let panicked = HubFault::WorkerPanicked("index out of bounds".into());
        assert_ne!(declared.code(), panicked.code());
    }

    #[test]
    fn protocol_violation_is_distinct_from_operation_failure() {
        let protocol = HubFault::ProtocolViolation("reply missing request_id".into());
        let op_failure = HubFault::ToolDeclaredFailure("dimension mismatch".into());
        assert_ne!(protocol.code(), op_failure.code());
    }

    #[test]
    fn tool_error_display_includes_code_and_message() {
        let e = HubToolError::new("DimensionMismatch", "expected 128 got 64");
        assert_eq!(e.to_string(), "DimensionMismatch: expected 128 got 64");
    }

    #[test]
    fn new_v0_completion_fault_codes_are_stable_and_distinct() {
        assert_eq!(
            HubFault::SensitiveCapabilityDenied("NetworkAccess".into()).code(),
            "SensitiveCapabilityDenied"
        );
        assert_eq!(HubFault::WorkerDegraded.code(), "WorkerDegraded");
        assert_eq!(HubFault::WorkerBusy.code(), "WorkerBusy");
        assert_eq!(
            HubFault::PersistenceFailed("disk full".into()).code(),
            "PersistenceFailed"
        );
        assert_eq!(
            HubFault::RecoveryRequired("unfinished transaction".into()).code(),
            "RecoveryRequired"
        );
        assert_eq!(HubFault::SequenceExhausted.code(), "SequenceExhausted");
        let budget_err = HubBudgetExceeded {
            kind: crate::resource::HubResourceKind::InputBytes,
            limit: 10,
            attempted: 11,
        };
        assert_eq!(
            HubFault::SessionLimitExceeded(budget_err).code(),
            "SessionLimitExceeded"
        );
    }

    #[test]
    fn sensitive_capability_denied_and_worker_degraded_are_pre_dispatch_rejections() {
        assert!(HubFault::SensitiveCapabilityDenied("x".into()).is_pre_dispatch_rejection());
        assert!(HubFault::WorkerDegraded.is_pre_dispatch_rejection());
        let budget_err = HubBudgetExceeded {
            kind: crate::resource::HubResourceKind::InputBytes,
            limit: 10,
            attempted: 11,
        };
        assert!(HubFault::SessionLimitExceeded(budget_err).is_pre_dispatch_rejection());
    }

    #[test]
    fn worker_busy_is_not_a_pre_dispatch_rejection() {
        // WorkerBusy is only ever observed once dispatch itself has begun
        // (the worker's own health state was already Busy), so it must be
        // classified as a dispatch-time fault, not a pre-admission one.
        assert!(!HubFault::WorkerBusy.is_pre_dispatch_rejection());
    }
}
