//! Opaque, non-authoritative references to external Semantic-owned entities.
#![allow(dead_code)]

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct RevisionRef(u64);

impl RevisionRef {
    pub(crate) const fn new(raw: u64) -> Self {
        Self(raw)
    }
    pub(crate) const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct EpochRef(u64);

impl EpochRef {
    pub(crate) const fn new(raw: u64) -> Self {
        Self(raw)
    }
    pub(crate) const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ActorRef(u64);

impl ActorRef {
    pub(crate) const fn new(raw: u64) -> Self {
        Self(raw)
    }
    pub(crate) const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SessionRef(u64);

impl SessionRef {
    pub(crate) const fn new(raw: u64) -> Self {
        Self(raw)
    }
    pub(crate) const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ClientRef(u64);

impl ClientRef {
    pub(crate) const fn new(raw: u64) -> Self {
        Self(raw)
    }
    pub(crate) const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct CapabilityRef(u64);

impl CapabilityRef {
    pub(crate) const fn new(raw: u64) -> Self {
        Self(raw)
    }
    pub(crate) const fn raw(self) -> u64 {
        self.0
    }
}
