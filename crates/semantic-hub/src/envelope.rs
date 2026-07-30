//! Request/reply envelopes exchanged between a caller and the Hub.

use crate::capability::HubCapabilitySet;
use crate::execution::HubPrivacyClass;
use crate::fault::HubFault;
use crate::ids::{
    HubApiVersion, HubCallerIdentity, HubOperationId, HubRequestId, HubSessionId, HubToolId,
};
use crate::provenance::HubProvenance;
use crate::resource::{HubResourceBudget, HubResourceUsage};

/// Schema version of the `HubRequest`/`HubReply` wire shape, independent of
/// [`HubApiVersion`] (which covers the whole registry/admission/dispatch
/// contract). Bumped only when the envelope's own field set changes.
///
/// v1 -> v2 (Semantic Hub v0 completion pass): `HubReply` gained
/// `logical_sequence`, `provenance`, and `warnings`. `HubRequest`'s own
/// field set is unchanged -- a logical sequence number is Hub-*resolved*,
/// not caller-supplied (see `docs/spec/hub/hub_session_v0.md`), so no
/// request-side field was added. The one caller of this crate
/// (`smc-cli::hub`) always constructs `HubRequest.schema_version` from
/// this live constant rather than a hardcoded literal, so this bump has no
/// backward-compatibility burden today; it exists so a *future* caller
/// pinned to v1 gets a typed `SchemaVersionUnsupported` rejection instead
/// of silently misinterpreting a `HubReply` missing fields it expects.
pub const HUB_ENVELOPE_SCHEMA_VERSION: u32 = 2;

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
    /// The Hub-assigned position of this invocation within its audit
    /// trail, matching `HubAuditRecord::sequence` for the record this
    /// invocation produced. Hub-*resolved*, never caller-supplied: within
    /// one session, `logical_sequence` is strictly increasing in the same
    /// order requests were submitted (see
    /// `docs/spec/hub/hub_session_v0.md` Section on ordering).
    pub logical_sequence: u64,
    pub tool_id: HubToolId,
    pub tool_version: crate::ids::HubToolVersion,
    pub operation_id: HubOperationId,
    pub status: HubReplyStatus,
    pub payload: Vec<u8>,
    pub resource_usage: HubResourceUsage,
    /// Evidence of how this reply was produced -- never a claim that its
    /// payload is true, relevant, or safe to commit. See
    /// `HubProvenance`'s own doc comment and
    /// `docs/architecture/semantic_hub_v0.md` Section 4.
    pub provenance: HubProvenance,
    /// Non-fatal observations the Hub or adapter wants to surface
    /// alongside a reply that otherwise completed normally (or was
    /// rejected/failed on its own already-reported terms). Empty by
    /// default in v0: no warning-producing path exists yet, and this
    /// field is never backfilled with a fabricated entry just to be
    /// non-empty.
    pub warnings: Vec<String>,
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

    fn sample_provenance() -> HubProvenance {
        HubProvenance {
            schema_version: crate::provenance::HUB_PROVENANCE_SCHEMA_VERSION,
            request_id: HubRequestId::new("req-1").unwrap(),
            session_id: HubSessionId::new("sess-1").unwrap(),
            logical_sequence: 0,
            caller_identity: HubCallerIdentity::new("cli:local").unwrap(),
            tool_id: HubToolId::new("vector.turbovec").unwrap(),
            tool_version: HubToolVersion::new(0, 1, 0),
            adapter_provenance: "test".into(),
            hub_api_version: HubApiVersion::CURRENT,
            execution_mode: crate::execution::HubExecutionMode::InProcess,
            operation_id: HubOperationId::new("vector.search").unwrap(),
            determinism: HubDeterminismClass::Unknown,
            trust_class: crate::execution::HubTrustClass::InProcessUnisolated,
            privacy_class: HubPrivacyClass::ProjectLocal,
            result_kind: "hub.reply.payload".into(),
            result_id: "reply:req-1:0".into(),
            result_digest: crate::provenance::HubDigest::of(b"[]"),
            input_digest: crate::provenance::HubDigest::of(b"{}"),
            output_digest: crate::provenance::HubDigest::of(b"[]"),
            capability_context_digest: crate::provenance::HubDigest::of(b""),
            resource_budget_digest: crate::provenance::HubDigest::of(b"budget"),
            resource_usage: HubResourceUsage::default(),
            audit_record_id: Some("audit:sess-1:0".into()),
            worker_state_after: crate::worker::HubWorkerState::Ready,
            artifact: None,
            warnings: Vec::new(),
        }
    }

    #[test]
    fn reply_holds_declared_shape() {
        let reply = HubReply {
            schema_version: HUB_ENVELOPE_SCHEMA_VERSION,
            request_id: HubRequestId::new("req-1").unwrap(),
            logical_sequence: 0,
            tool_id: HubToolId::new("vector.turbovec").unwrap(),
            tool_version: HubToolVersion::new(0, 1, 0),
            operation_id: HubOperationId::new("vector.search").unwrap(),
            status: HubReplyStatus::Success,
            payload: b"[]".to_vec(),
            resource_usage: HubResourceUsage::default(),
            provenance: sample_provenance(),
            warnings: Vec::new(),
        };
        assert!(reply.status.is_success());
        assert_eq!(reply.payload, b"[]");
        assert!(reply.warnings.is_empty());
        let _ = HubDeterminismClass::Unknown;
    }
}
