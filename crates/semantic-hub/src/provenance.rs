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

/// Ties one reply back to the exact tool/adapter/dependency identity and
/// input/output bytes that produced it. Proves production path, not truth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HubProvenance {
    pub tool_id: crate::ids::HubToolId,
    pub tool_version: crate::ids::HubToolVersion,
    pub adapter_provenance: String,
    pub execution_mode: crate::execution::HubExecutionMode,
    pub determinism: crate::execution::HubDeterminismClass,
    pub trust_class: crate::execution::HubTrustClass,
    pub input_digest: HubDigest,
    pub output_digest: HubDigest,
}

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

    #[test]
    fn hub_digest_display_is_stable_and_bounded() {
        let d = HubDigest::of(b"abc");
        let s = d.to_string();
        assert!(s.starts_with("fnv1a64:"));
        assert!(s.ends_with(":3"));
    }
}
