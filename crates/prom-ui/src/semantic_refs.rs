//! Opaque, non-authoritative references to external Semantic-owned entities.
#![allow(dead_code, unused_imports)]

pub(crate) use prom_refs::{
    ActorRef, CapabilityRef, ClientRef, EpochRef, ReferenceToken, RevisionRef, SessionRef,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SemanticValueRef(u64);

impl SemanticValueRef {
    pub(crate) const fn new(raw: u64) -> Self {
        Self(raw)
    }
    pub(crate) const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SemanticActionRef(u64);

impl SemanticActionRef {
    pub(crate) const fn new(raw: u64) -> Self {
        Self(raw)
    }
    pub(crate) const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SemanticEvidenceRef(u64);

impl SemanticEvidenceRef {
    pub(crate) const fn new(raw: u64) -> Self {
        Self(raw)
    }
    pub(crate) const fn raw(self) -> u64 {
        self.0
    }
}
