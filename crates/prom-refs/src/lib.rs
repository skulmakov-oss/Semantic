//! This crate owns authority-free reference values.
//! References are untrusted claims; possession grants no authority.
//! Trusted domain owners issue and resolve canonical references.
//! Resolution, admission, capability checks, and execution are outside this crate.

#![no_std]
#![forbid(unsafe_code)]

/// An explicit four-part composite reference token.
/// It provides deterministic provenance coordinates, not authentication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReferenceToken {
    issuer: u64,
    namespace: u32,
    generation: u32,
    value: u64,
}

impl ReferenceToken {
    /// Construct a new `ReferenceToken` from explicit components.
    pub const fn new(issuer: u64, namespace: u32, generation: u32, value: u64) -> Self {
        Self {
            issuer,
            namespace,
            generation,
            value,
        }
    }

    /// Returns the issuer of the token.
    pub const fn issuer(&self) -> u64 {
        self.issuer
    }

    /// Returns the namespace of the token.
    pub const fn namespace(&self) -> u32 {
        self.namespace
    }

    /// Returns the generation of the token.
    pub const fn generation(&self) -> u32 {
        self.generation
    }

    /// Returns the value of the token.
    pub const fn value(&self) -> u64 {
        self.value
    }
}

macro_rules! define_ref_wrapper {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name {
            token: ReferenceToken,
        }

        impl $name {
            /// Construct from a ReferenceToken.
            pub const fn new(token: ReferenceToken) -> Self {
                Self { token }
            }

            /// Retrieve the underlying ReferenceToken.
            pub const fn token(&self) -> ReferenceToken {
                self.token
            }
        }
    };
}

define_ref_wrapper!(CapabilityRef, "Capability Candidate Reference");
define_ref_wrapper!(ActorRef, "Authority-Scoped Correlation Handle for Actor");
define_ref_wrapper!(SessionRef, "Session-Scoped Handle");
define_ref_wrapper!(ClientRef, "Session-Scoped Correlation Handle for Client");
define_ref_wrapper!(RevisionRef, "Versioned State Token for Revision");
define_ref_wrapper!(EpochRef, "Versioned Execution Token for Epoch");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reference_token_preserves_all_four_components() {
        let token = ReferenceToken::new(1, 2, 3, 4);
        assert_eq!(token.issuer(), 1);
        assert_eq!(token.namespace(), 2);
        assert_eq!(token.generation(), 3);
        assert_eq!(token.value(), 4);
    }

    #[test]
    fn reference_token_equality_uses_all_components() {
        let a = ReferenceToken::new(1, 2, 3, 4);
        let b = ReferenceToken::new(1, 2, 3, 4);
        let c = ReferenceToken::new(9, 2, 3, 4);
        let d = ReferenceToken::new(1, 9, 3, 4);
        let e = ReferenceToken::new(1, 2, 9, 4);
        let f = ReferenceToken::new(1, 2, 3, 9);
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
        assert_ne!(a, e);
        assert_ne!(a, f);
    }

    #[test]
    fn reference_token_ordering_is_deterministic() {
        let a = ReferenceToken::new(1, 2, 3, 4);
        let b = ReferenceToken::new(1, 2, 3, 5);
        let c = ReferenceToken::new(2, 0, 0, 0);
        assert!(a < b);
        assert!(a < c);
    }

    #[test]
    fn capability_ref_preserves_its_token() {
        let t = ReferenceToken::new(1, 2, 3, 4);
        let r = CapabilityRef::new(t);
        assert_eq!(r.token(), t);
    }

    #[test]
    fn actor_ref_preserves_its_token() {
        let t = ReferenceToken::new(1, 2, 3, 4);
        let r = ActorRef::new(t);
        assert_eq!(r.token(), t);
    }

    #[test]
    fn session_ref_preserves_its_token() {
        let t = ReferenceToken::new(1, 2, 3, 4);
        let r = SessionRef::new(t);
        assert_eq!(r.token(), t);
    }

    #[test]
    fn client_ref_preserves_its_token() {
        let t = ReferenceToken::new(1, 2, 3, 4);
        let r = ClientRef::new(t);
        assert_eq!(r.token(), t);
    }

    #[test]
    fn revision_ref_preserves_its_token() {
        let t = ReferenceToken::new(1, 2, 3, 4);
        let r = RevisionRef::new(t);
        assert_eq!(r.token(), t);
    }

    #[test]
    fn epoch_ref_preserves_its_token() {
        let t = ReferenceToken::new(1, 2, 3, 4);
        let r = EpochRef::new(t);
        assert_eq!(r.token(), t);
    }
}
