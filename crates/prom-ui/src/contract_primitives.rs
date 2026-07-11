//! UI DNA v2 contract primitives.
//!
//! This module is a neutral leaf provider. It must not import projection,
//! static IR, runtime, renderer, backend, or admission consumers.
#![allow(dead_code)]

use core::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SchemaVersion(u32);

impl SchemaVersion {
    pub(crate) const CURRENT: Self = Self(1);

    pub(crate) const fn new(raw: u32) -> Result<Self, ContractPrimitiveError> {
        if raw == 0 {
            Err(ContractPrimitiveError::ZeroVersion)
        } else {
            Ok(Self(raw))
        }
    }

    pub(crate) const fn raw(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ContractVersion(u32);

impl ContractVersion {
    pub(crate) const CURRENT: Self = Self(1);

    pub(crate) const fn new(raw: u32) -> Result<Self, ContractPrimitiveError> {
        if raw == 0 {
            Err(ContractPrimitiveError::ZeroVersion)
        } else {
            Ok(Self(raw))
        }
    }

    pub(crate) const fn raw(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SourceId(u64);

impl SourceId {
    pub(crate) const fn new(raw: u64) -> Result<Self, ContractPrimitiveError> {
        if raw == 0 {
            Err(ContractPrimitiveError::ZeroId)
        } else {
            Ok(Self(raw))
        }
    }

    pub(crate) const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SourceSpan {
    start: u32,
    end: u32,
}

impl SourceSpan {
    pub(crate) const fn new(start: u32, end: u32) -> Result<Self, ContractPrimitiveError> {
        if start > end {
            Err(ContractPrimitiveError::InvalidSourceSpan)
        } else {
            Ok(Self { start, end })
        }
    }

    pub(crate) const fn start(self) -> u32 {
        self.start
    }

    pub(crate) const fn end(self) -> u32 {
        self.end
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SourceRef {
    source: SourceId,
    span: SourceSpan,
}

impl SourceRef {
    pub(crate) const fn new(source: SourceId, span: SourceSpan) -> Self {
        Self { source, span }
    }

    pub(crate) const fn source(self) -> SourceId {
        self.source
    }

    pub(crate) const fn span(self) -> SourceSpan {
        self.span
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Revision(u64);

impl Revision {
    pub(crate) const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub(crate) const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Epoch(u64);

impl Epoch {
    pub(crate) const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub(crate) const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ProjectionSourceSurfaceId(u64);

impl ProjectionSourceSurfaceId {
    pub(crate) const fn new(raw: u64) -> Result<Self, ContractPrimitiveError> {
        if raw == 0 {
            Err(ContractPrimitiveError::ZeroId)
        } else {
            Ok(Self(raw))
        }
    }

    pub(crate) const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ProjectionSourceNodeId(u64);

impl ProjectionSourceNodeId {
    pub(crate) const fn new(raw: u64) -> Result<Self, ContractPrimitiveError> {
        if raw == 0 {
            Err(ContractPrimitiveError::ZeroId)
        } else {
            Ok(Self(raw))
        }
    }

    pub(crate) const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct StaticDocumentId(u64);

impl StaticDocumentId {
    pub(crate) const fn new(raw: u64) -> Result<Self, ContractPrimitiveError> {
        if raw == 0 {
            Err(ContractPrimitiveError::ZeroId)
        } else {
            Ok(Self(raw))
        }
    }

    pub(crate) const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct StaticSurfaceId(u64);

impl StaticSurfaceId {
    pub(crate) const fn new(raw: u64) -> Result<Self, ContractPrimitiveError> {
        if raw == 0 {
            Err(ContractPrimitiveError::ZeroId)
        } else {
            Ok(Self(raw))
        }
    }

    pub(crate) const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct StaticNodeId(u64);

impl StaticNodeId {
    pub(crate) const fn new(raw: u64) -> Result<Self, ContractPrimitiveError> {
        if raw == 0 {
            Err(ContractPrimitiveError::ZeroId)
        } else {
            Ok(Self(raw))
        }
    }

    pub(crate) const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct CollectionKey(u64);

impl CollectionKey {
    pub(crate) const fn new(raw: u64) -> Result<Self, ContractPrimitiveError> {
        if raw == 0 {
            Err(ContractPrimitiveError::ZeroId)
        } else {
            Ok(Self(raw))
        }
    }

    pub(crate) const fn raw(self) -> u64 {
        self.0
    }
}

/// Explicit semantic position of a child within one parent's ordered edge set.
/// It is deliberately separate from node identity and storage position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ChildOrder(u32);

impl ChildOrder {
    pub(crate) const fn new(raw: u32) -> Self {
        Self(raw)
    }

    pub(crate) const fn raw(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct DiagnosticCode(&'static str);

impl DiagnosticCode {
    pub(crate) const fn new(code: &'static str) -> Self {
        Self(code)
    }

    pub(crate) const fn as_str(self) -> &'static str {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct DiagnosticCoordinate {
    source: Option<SourceRef>,
    code: DiagnosticCode,
    domain: u64,
    secondary: u64,
}

impl DiagnosticCoordinate {
    pub(crate) const fn new(
        source: Option<SourceRef>,
        code: DiagnosticCode,
        domain: u64,
        secondary: u64,
    ) -> Self {
        Self {
            source,
            code,
            domain,
            secondary,
        }
    }

    pub(crate) const fn source(self) -> Option<SourceRef> {
        self.source
    }

    pub(crate) const fn code(self) -> DiagnosticCode {
        self.code
    }

    pub(crate) const fn domain(self) -> u64 {
        self.domain
    }

    pub(crate) const fn secondary(self) -> u64 {
        self.secondary
    }
}

impl Ord for DiagnosticCoordinate {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.source, self.code, self.domain, self.secondary).cmp(&(
            other.source,
            other.code,
            other.domain,
            other.secondary,
        ))
    }
}

impl PartialOrd for DiagnosticCoordinate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ContractPrimitiveError {
    ZeroVersion,
    ZeroId,
    InvalidSourceSpan,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_primitives_source_span_accepts_ordered_bounds() {
        let span = SourceSpan::new(2, 5).expect("valid span");
        assert_eq!(span.start(), 2);
        assert_eq!(span.end(), 5);
    }

    #[test]
    fn contract_primitives_source_span_rejects_inverted_bounds() {
        assert_eq!(
            SourceSpan::new(5, 2),
            Err(ContractPrimitiveError::InvalidSourceSpan)
        );
    }

    #[test]
    fn contract_primitives_versions_are_non_zero() {
        assert_eq!(
            SchemaVersion::new(0),
            Err(ContractPrimitiveError::ZeroVersion)
        );
        assert_eq!(
            ContractVersion::new(0),
            Err(ContractPrimitiveError::ZeroVersion)
        );
        assert_eq!(SchemaVersion::CURRENT.raw(), 1);
        assert_eq!(ContractVersion::CURRENT.raw(), 1);
    }

    #[test]
    fn contract_primitives_domain_ids_do_not_cross_compare() {
        let source = ProjectionSourceNodeId::new(7).expect("source node id");
        let static_node = StaticNodeId::new(7).expect("static node id");

        assert_eq!(source.raw(), static_node.raw());
    }

    #[test]
    fn contract_primitives_order_diagnostic_coordinates_deterministically() {
        let source = SourceId::new(1).expect("source id");
        let first = DiagnosticCoordinate::new(
            Some(SourceRef::new(source, SourceSpan::new(0, 1).expect("span"))),
            DiagnosticCode::new("B"),
            1,
            0,
        );
        let second = DiagnosticCoordinate::new(
            Some(SourceRef::new(source, SourceSpan::new(0, 1).expect("span"))),
            DiagnosticCode::new("C"),
            1,
            0,
        );

        assert!(first < second);
        assert_eq!(first.code.as_str(), "B");
    }
}
