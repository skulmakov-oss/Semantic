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

    /// Rejected/terminated during or after dispatch.
    DeadlineExceeded,
    Cancelled,
    ResourceExhausted(HubBudgetExceeded),

    /// The tool ran and declared its own failure -- distinct from a crash.
    ToolDeclaredFailure(String),

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
            HubFault::DeadlineExceeded => "DeadlineExceeded",
            HubFault::Cancelled => "Cancelled",
            HubFault::ResourceExhausted(_) => "ResourceExhausted",
            HubFault::ToolDeclaredFailure(_) => "ToolDeclaredFailure",
            HubFault::WorkerPanicked(_) => "WorkerPanicked",
            HubFault::ProtocolViolation(_) => "ProtocolViolation",
            HubFault::OutputRejected(_) => "OutputRejected",
            HubFault::AuditProvenanceFailure(_) => "AuditProvenanceFailure",
            HubFault::InternalHubFault(_) => "InternalHubFault",
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
        )
    }
}

impl fmt::Display for HubFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HubFault::InputRejected(m)
            | HubFault::CapabilityDenied(m)
            | HubFault::PrivacyDenied(m)
            | HubFault::ToolDeclaredFailure(m)
            | HubFault::WorkerPanicked(m)
            | HubFault::ProtocolViolation(m)
            | HubFault::OutputRejected(m)
            | HubFault::AuditProvenanceFailure(m)
            | HubFault::InternalHubFault(m) => write!(f, "{}: {m}", self.code()),
            HubFault::ResourceBudgetInvalid(e) | HubFault::ResourceExhausted(e) => {
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
}
