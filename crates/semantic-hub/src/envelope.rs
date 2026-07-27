//! Request/reply envelopes exchanged between a caller and the Hub.

use crate::capability::HubCapabilitySet;
use crate::execution::HubPrivacyClass;
use crate::fault::HubFault;
use crate::ids::{
    HubApiVersion, HubCallerIdentity, HubOperationId, HubRequestId, HubSessionId, HubToolId,
};
use crate::resource::{HubResourceBudget, HubResourceUsage};

/// Schema version of the `HubRequest`/`HubReply` wire shape, independent of
/// [`HubApiVersion`] (which covers the whole registry/admission/dispatch
/// contract). Bumped only when the envelope's own field set changes.
pub const HUB_ENVELOPE_SCHEMA_VERSION: u32 = 1;

/// Maximum accepted `payload` byte length before admission even attempts to
/// interpret it. Chosen to bound allocation for hostile oversized input.
pub const MAX_PAYLOAD_BYTES: usize = 32 * 1024 * 1024;

/// A typed request delivered to the Hub for admission and dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HubRequest {
    pub schema_version: u32,
    pub api_version: HubApiVersion,
    pub request_id: HubRequestId,
    pub session_id: HubSessionId,
    pub caller_identity: HubCallerIdentity,
    pub tool_id: HubToolId,
    pub operation_id: HubOperationId,
    pub capability_context: HubCapabilitySet,
    pub privacy_class: HubPrivacyClass,
    pub resource_budget: HubResourceBudget,
    pub payload: Vec<u8>,
}

impl HubRequest {
    pub fn payload_len(&self) -> usize {
        self.payload.len()
    }
}

/// Structural status of one completed (or rejected) invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HubReplyStatus {
    Success,
    Rejected(HubFault),
    ToolFailed(HubFault),
    Crashed(HubFault),
    HubFault(HubFault),
}

impl HubReplyStatus {
    pub const fn fault(&self) -> Option<&HubFault> {
        match self {
            HubReplyStatus::Success => None,
            HubReplyStatus::Rejected(f)
            | HubReplyStatus::ToolFailed(f)
            | HubReplyStatus::Crashed(f)
            | HubReplyStatus::HubFault(f) => Some(f),
        }
    }

    pub const fn is_success(&self) -> bool {
        matches!(self, HubReplyStatus::Success)
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            HubReplyStatus::Success => "Success",
            HubReplyStatus::Rejected(_) => "Rejected",
            HubReplyStatus::ToolFailed(_) => "ToolFailed",
            HubReplyStatus::Crashed(_) => "Crashed",
            HubReplyStatus::HubFault(_) => "HubFault",
        }
    }
}

/// A typed, structurally-validated result returned by the Hub. Its
/// `payload` is untrusted computational evidence, never Semantic truth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HubReply {
    pub schema_version: u32,
    pub request_id: HubRequestId,
    pub tool_id: HubToolId,
    pub tool_version: crate::ids::HubToolVersion,
    pub operation_id: HubOperationId,
    pub status: HubReplyStatus,
    pub payload: Vec<u8>,
    pub resource_usage: HubResourceUsage,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::HubDeterminismClass;
    use crate::ids::HubToolVersion;
    use crate::resource::HubBudgetExceeded;
    use crate::resource::HubResourceKind;

    fn sample_request() -> HubRequest {
        HubRequest {
            schema_version: HUB_ENVELOPE_SCHEMA_VERSION,
            api_version: HubApiVersion::CURRENT,
            request_id: HubRequestId::new("req-1").unwrap(),
            session_id: HubSessionId::new("sess-1").unwrap(),
            caller_identity: HubCallerIdentity::new("cli:local").unwrap(),
            tool_id: HubToolId::new("vector.turbovec").unwrap(),
            operation_id: HubOperationId::new("vector.search").unwrap(),
            capability_context: HubCapabilitySet::empty(),
            privacy_class: HubPrivacyClass::ProjectLocal,
            resource_budget: HubResourceBudget::V0_CEILING,
            payload: b"{}".to_vec(),
        }
    }

    #[test]
    fn request_payload_len_matches_payload() {
        let req = sample_request();
        assert_eq!(req.payload_len(), 2);
    }

    #[test]
    fn reply_status_success_has_no_fault() {
        assert!(HubReplyStatus::Success.fault().is_none());
        assert!(HubReplyStatus::Success.is_success());
    }

    #[test]
    fn reply_status_variants_carry_and_expose_their_fault() {
        let budget_err = HubBudgetExceeded {
            kind: HubResourceKind::OutputBytes,
            limit: 10,
            attempted: 11,
        };
        let status = HubReplyStatus::Rejected(HubFault::ResourceExhausted(budget_err));
        assert!(!status.is_success());
        assert_eq!(status.fault().unwrap().code(), "ResourceExhausted");
        assert_eq!(status.as_str(), "Rejected");
    }

    #[test]
    fn reply_holds_declared_shape() {
        let reply = HubReply {
            schema_version: HUB_ENVELOPE_SCHEMA_VERSION,
            request_id: HubRequestId::new("req-1").unwrap(),
            tool_id: HubToolId::new("vector.turbovec").unwrap(),
            tool_version: HubToolVersion::new(0, 1, 0),
            operation_id: HubOperationId::new("vector.search").unwrap(),
            status: HubReplyStatus::Success,
            payload: b"[]".to_vec(),
            resource_usage: HubResourceUsage::default(),
        };
        assert!(reply.status.is_success());
        assert_eq!(reply.payload, b"[]");
        let _ = HubDeterminismClass::Unknown;
    }
}
