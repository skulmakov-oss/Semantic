//! Action IR contract foundation.
#![allow(dead_code)]

use alloc::vec::Vec;
use core::cmp::Ordering;

use crate::contract_primitives::{DiagnosticCode, DiagnosticCoordinate, SourceRef, StaticNodeId};
use crate::semantic_refs::{CapabilityRef, EpochRef, RevisionRef, SemanticActionRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ActionRouteId(pub(crate) u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ActionRouteOrder(pub(crate) u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ActionTargetSlot(pub(crate) u32);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ActionTarget {
    pub(crate) node: StaticNodeId,
    pub(crate) slot: ActionTargetSlot,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ActionContextRequirements {
    pub(crate) requires_actor: bool,
    pub(crate) requires_session: bool,
    pub(crate) requires_client: bool,
    pub(crate) required_revision: Option<RevisionRef>,
    pub(crate) required_epoch: Option<EpochRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ActionRoute {
    pub(crate) target: ActionTarget,
    pub(crate) order: ActionRouteOrder,
    pub(crate) id: ActionRouteId,
    pub(crate) semantic_action: SemanticActionRef,
    pub(crate) source_ref: Option<SourceRef>,
    pub(crate) context_reqs: ActionContextRequirements,
    pub(crate) capability: Option<CapabilityRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ActionIrDiagnosticKind {
    DuplicateActionRouteId,
    MissingTargetNode,
    DuplicateRouteSlot,
    DuplicateRouteOrder,
    ZeroSemanticActionRef,
    InvalidContextRequirements,
}

impl ActionIrDiagnosticKind {
    const fn code(self) -> DiagnosticCode {
        match self {
            Self::DuplicateActionRouteId => DiagnosticCode::new("AIR_DUP_ROUTE_ID"),
            Self::MissingTargetNode => DiagnosticCode::new("AIR_MISSING_TARGET"),
            Self::DuplicateRouteSlot => DiagnosticCode::new("AIR_DUP_ROUTE_SLOT"),
            Self::DuplicateRouteOrder => DiagnosticCode::new("AIR_DUP_ROUTE_ORDER"),
            Self::ZeroSemanticActionRef => DiagnosticCode::new("AIR_ZERO_ACTION_REF"),
            Self::InvalidContextRequirements => DiagnosticCode::new("AIR_INVALID_CTX_REQ"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ActionIrDiagnostic {
    pub(crate) coordinate: DiagnosticCoordinate,
}

impl ActionIrDiagnostic {
    pub(crate) fn new(
        source: Option<SourceRef>,
        kind: ActionIrDiagnosticKind,
        primary_id: u64,
        secondary_id: u64,
    ) -> Self {
        Self {
            coordinate: DiagnosticCoordinate::new(source, kind.code(), primary_id, secondary_id),
        }
    }
}

impl Ord for ActionIrDiagnostic {
    fn cmp(&self, other: &Self) -> Ordering {
        self.coordinate.cmp(&other.coordinate)
    }
}

impl PartialOrd for ActionIrDiagnostic {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ActionIrDocument {
    pub(crate) routes: Vec<ActionRoute>,
}

impl ActionIrDocument {
    pub(crate) fn build(
        mut routes: Vec<ActionRoute>,
        target_document: &crate::static_ir::StaticUiDocument,
    ) -> Result<Self, Vec<ActionIrDiagnostic>> {
        routes.sort();
        let mut diagnostics = Vec::new();

        for (i, r) in routes.iter().enumerate() {
            // Check missing target
            if !target_document
                .nodes()
                .iter()
                .any(|n| n.id() == r.target.node)
            {
                diagnostics.push(ActionIrDiagnostic::new(
                    r.source_ref,
                    ActionIrDiagnosticKind::MissingTargetNode,
                    r.id.0,
                    r.target.node.raw(),
                ));
            }

            // Check duplicate ID
            if routes[..i].iter().any(|prev| prev.id == r.id) {
                diagnostics.push(ActionIrDiagnostic::new(
                    r.source_ref,
                    ActionIrDiagnosticKind::DuplicateActionRouteId,
                    r.id.0,
                    0,
                ));
            }

            // Canonical duplicate key: target + slot + order
            if routes[..i]
                .iter()
                .any(|prev| prev.target == r.target && prev.order == r.order)
            {
                diagnostics.push(ActionIrDiagnostic::new(
                    r.source_ref,
                    ActionIrDiagnosticKind::DuplicateRouteOrder,
                    r.id.0,
                    0,
                ));
            }

            // Check Zero Semantic Action Ref
            if r.semantic_action.raw() == 0 {
                diagnostics.push(ActionIrDiagnostic::new(
                    r.source_ref,
                    ActionIrDiagnosticKind::ZeroSemanticActionRef,
                    r.id.0,
                    0,
                ));
            }
        }

        if !diagnostics.is_empty() {
            diagnostics.sort();
            diagnostics.dedup();
            return Err(diagnostics);
        }

        // Normalization must be idempotent, stable, storage-permutation invariant,
        // and differentiate on ActionRouteOrder. We sort routes by the target, slot, order, and id.
        routes.sort_by(|a, b| {
            a.target
                .cmp(&b.target)
                .then(a.order.cmp(&b.order))
                .then(a.id.cmp(&b.id))
        });

        Ok(Self { routes })
    }
}
