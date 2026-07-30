//! Provenance evidence: proves how a result was produced, not that it is
//! true. No cryptographic signing chain is implemented here -- that is
//! explicitly future work tracked by issue #1374 (artifact provenance and
//! signing chain). `content_digest` is a bounded, deterministic,
//! non-cryptographic fingerprint (FNV-1a/64) used only to correlate exact
//! bytes across request/reply/audit evidence; it is not a security
//! commitment and must never be documented as one.

use std::fmt;

/// FNV-1a 64-bit hash: simple, dependency-free, and fully deterministic
/// across platforms and processes. Chosen over `DefaultHasher` to avoid any
/// reliance on unspecified standard-library hasher internals, and over
/// adding a cryptographic-hash dependency for a v0 correlation fingerprint
/// that explicitly does not claim tamper-evidence.
pub fn content_digest(bytes: &[u8]) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut hash = FNV_OFFSET_BASIS;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// Digest of a payload plus its declared byte length, so a digest collision
/// across differently-sized payloads is distinguishable in evidence review.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HubDigest {
    pub fnv1a64: u64,
    pub byte_len: u64,
}

impl HubDigest {
    pub fn of(bytes: &[u8]) -> Self {
        Self {
            fnv1a64: content_digest(bytes),
            byte_len: bytes.len() as u64,
        }
    }
}

impl fmt::Display for HubDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fnv1a64:{:016x}:{}", self.fnv1a64, self.byte_len)
    }
}

/// Identifies the durable artifact a mutating operation produced (e.g. a
/// TurboVec `.tvim` index write), so provenance can bind a reply back to
/// the exact persisted bytes it committed -- not only to the reply payload
/// itself. `None` on [`HubProvenance::artifact`] for a non-mutating
/// operation (search, describe): there is no artifact to bind to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HubArtifactProvenance {
    /// Stable, tool-defined kind string, e.g. `"turbovec.index"`.
    pub kind: String,
    /// Tool-defined artifact identity, e.g. the index name.
    pub id: String,
    /// Digest of the exact bytes committed to durable storage.
    pub digest: HubDigest,
    /// Adapter transaction identity when the artifact was produced by a
    /// recoverable mutation protocol.
    pub transaction_id: Option<String>,
}

/// Ties one reply back to the exact tool/adapter/dependency identity,
/// input/output bytes, capability/budget context, and audit correlation
/// that produced it. Proves production path, not truth -- see
/// `docs/architecture/semantic_hub_v0.md` Section 4 for the full
/// authority/non-authority statement this type is evidence for, never a
/// substitute for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HubProvenance {
    pub schema_version: u32,
    pub request_id: crate::ids::HubRequestId,
    pub session_id: crate::ids::HubSessionId,
    pub logical_sequence: u64,
    pub caller_identity: crate::ids::HubCallerIdentity,
    pub tool_id: crate::ids::HubToolId,
    pub tool_version: crate::ids::HubToolVersion,
    pub adapter_provenance: String,
    pub hub_api_version: crate::ids::HubApiVersion,
    pub execution_mode: crate::execution::HubExecutionMode,
    pub operation_id: crate::ids::HubOperationId,
    pub determinism: crate::execution::HubDeterminismClass,
    pub trust_class: crate::execution::HubTrustClass,
    pub privacy_class: crate::execution::HubPrivacyClass,
    /// Stable result correlation independent of an optional durable
    /// artifact. For a non-mutating reply this binds the reply payload; for
    /// a committed mutation it binds the durable artifact.
    pub result_kind: String,
    pub result_id: String,
    pub result_digest: HubDigest,
    pub input_digest: HubDigest,
    pub output_digest: HubDigest,
    pub capability_context_digest: HubDigest,
    pub resource_budget_digest: HubDigest,
    pub resource_usage: crate::resource::HubResourceUsage,
    /// Deterministic identity of the matching audit record. `None` is
    /// reserved for the sequence-exhaustion sentinel reply, which cannot be
    /// appended without duplicating a sequence.
    pub audit_record_id: Option<String>,
    pub worker_state_after: crate::worker::HubWorkerState,
    pub artifact: Option<HubArtifactProvenance>,
    pub warnings: Vec<String>,
}

/// Schema version of [`HubProvenance`]'s own field set, independent of
/// [`crate::envelope::HUB_ENVELOPE_SCHEMA_VERSION`] (which covers the
/// request/reply envelope shape as a whole).
pub const HUB_PROVENANCE_SCHEMA_VERSION: u32 = 2;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_is_deterministic_across_calls() {
        let a = content_digest(b"hello world");
        let b = content_digest(b"hello world");
        assert_eq!(a, b);
    }

    #[test]
    fn digest_differs_for_different_content() {
        assert_ne!(content_digest(b"hello"), content_digest(b"world"));
    }

    #[test]
    fn digest_of_empty_input_is_the_fnv_offset_basis() {
        assert_eq!(content_digest(b""), 0xcbf29ce484222325);
    }

    #[test]
    fn hub_digest_captures_length_alongside_hash() {
        let d = HubDigest::of(b"abc");
        assert_eq!(d.byte_len, 3);
        assert_eq!(d.fnv1a64, content_digest(b"abc"));
    }

    fn sample_provenance() -> HubProvenance {
        HubProvenance {
            schema_version: HUB_PROVENANCE_SCHEMA_VERSION,
            request_id: crate::ids::HubRequestId::new("req-1").unwrap(),
            session_id: crate::ids::HubSessionId::new("sess-1").unwrap(),
            logical_sequence: 0,
            caller_identity: crate::ids::HubCallerIdentity::new("cli:local").unwrap(),
            tool_id: crate::ids::HubToolId::new("vector.turbovec").unwrap(),
            tool_version: crate::ids::HubToolVersion::new(0, 1, 0),
            adapter_provenance: "semantic-hub-turbovec 0.1.0".into(),
            hub_api_version: crate::ids::HubApiVersion::CURRENT,
            execution_mode: crate::execution::HubExecutionMode::InProcess,
            operation_id: crate::ids::HubOperationId::new("vector.search").unwrap(),
            determinism: crate::execution::HubDeterminismClass::Unknown,
            trust_class: crate::execution::HubTrustClass::InProcessUnisolated,
            privacy_class: crate::execution::HubPrivacyClass::ProjectLocal,
            result_kind: "hub.reply.payload".into(),
            result_id: "reply:req-1:0".into(),
            result_digest: HubDigest::of(b"output"),
            input_digest: HubDigest::of(b"input"),
            output_digest: HubDigest::of(b"output"),
            capability_context_digest: HubDigest::of(b"VectorSearch"),
            resource_budget_digest: HubDigest::of(b"budget"),
            resource_usage: crate::resource::HubResourceUsage::default(),
            audit_record_id: Some("audit:sess-1:0".into()),
            worker_state_after: crate::worker::HubWorkerState::Ready,
            artifact: None,
            warnings: Vec::new(),
        }
    }

    #[test]
    fn provenance_never_claims_truth_only_production_path() {
        // Documentation-as-code: this type carries no "is this true" field
        // by construction -- the closest thing to asserting a design
        // invariant in a test is confirming the shape only records
        // identity/digest/context evidence.
        let p = sample_provenance();
        assert_eq!(p.artifact, None);
        assert!(p.warnings.is_empty());
    }

    #[test]
    fn artifact_provenance_binds_a_mutating_operations_committed_bytes() {
        let mut p = sample_provenance();
        p.artifact = Some(HubArtifactProvenance {
            kind: "turbovec.index".into(),
            id: "docs".into(),
            digest: HubDigest::of(b"committed index bytes"),
            transaction_id: Some("txn-1".into()),
        });
        let artifact = p.artifact.unwrap();
        assert_eq!(artifact.kind, "turbovec.index");
        assert_eq!(artifact.digest, HubDigest::of(b"committed index bytes"));
        assert_eq!(artifact.transaction_id.as_deref(), Some("txn-1"));
    }

    #[test]
    fn correlation_identifiers_are_deterministic_and_do_not_use_wall_clock_time() {
        let a = sample_provenance();
        let b = sample_provenance();
        assert_eq!(a.result_id, b.result_id);
        assert_eq!(a.audit_record_id, b.audit_record_id);
        assert_eq!(a.result_digest, a.output_digest);
    }

    #[test]
    fn hub_digest_display_is_stable_and_bounded() {
        let d = HubDigest::of(b"abc");
        let s = d.to_string();
        assert!(s.starts_with("fnv1a64:"));
        assert!(s.ends_with(":3"));
    }
}
