//! Binding Graph contract foundation.
#![allow(dead_code)]

use alloc::vec::Vec;
use core::cmp::Ordering;

use crate::contract_primitives::{DiagnosticCode, DiagnosticCoordinate, SourceRef, StaticNodeId};
use crate::semantic_refs::SemanticValueRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct BindingId(pub(crate) u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum BindingValueDomain {
    Quad,
    Scalar,
    Text,
    Evidence,
    Collection,
    Task,
    Connectivity,
    ActionAvailability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct BindingSlot(pub(crate) u32);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct BindingTarget {
    pub(crate) node: StaticNodeId,
    pub(crate) slot: BindingSlot,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct BindingDependency {
    pub(crate) target_binding: BindingId,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct BindingDeclaration {
    pub(crate) id: BindingId,
    pub(crate) target: BindingTarget,
    pub(crate) domain: BindingValueDomain,
    pub(crate) dependencies: Vec<BindingDependency>,
    pub(crate) collection_key: Option<u64>,
    pub(crate) semantic_value: Option<SemanticValueRef>,
    pub(crate) source_ref: Option<SourceRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum BindingDiagnosticKind {
    DuplicateBindingId,
    MissingTargetNode,
    DuplicateTargetSlot,
    DanglingDependency,
    DuplicateDependency,
    SelfDependency,
    Cycle,
    MissingCollectionKey,
    InvalidCollectionKey,
    ZeroSemanticRef,
    InvalidDomainSlot,
}

impl BindingDiagnosticKind {
    const fn code(self) -> DiagnosticCode {
        match self {
            Self::DuplicateBindingId => DiagnosticCode::new("BG_DUP_BINDING_ID"),
            Self::MissingTargetNode => DiagnosticCode::new("BG_MISSING_TARGET"),
            Self::DuplicateTargetSlot => DiagnosticCode::new("BG_DUP_TARGET_SLOT"),
            Self::DanglingDependency => DiagnosticCode::new("BG_DANGLING_DEP"),
            Self::DuplicateDependency => DiagnosticCode::new("BG_DUP_DEP"),
            Self::SelfDependency => DiagnosticCode::new("BG_SELF_DEP"),
            Self::Cycle => DiagnosticCode::new("BG_CYCLE"),
            Self::MissingCollectionKey => DiagnosticCode::new("BG_MISSING_COLL_KEY"),
            Self::InvalidCollectionKey => DiagnosticCode::new("BG_INVALID_COLL_KEY"),
            Self::ZeroSemanticRef => DiagnosticCode::new("BG_ZERO_SEM_REF"),
            Self::InvalidDomainSlot => DiagnosticCode::new("BG_INVALID_DOMAIN_SLOT"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct BindingDiagnostic {
    pub(crate) coordinate: DiagnosticCoordinate,
}

impl BindingDiagnostic {
    pub(crate) fn new(
        source: Option<SourceRef>,
        kind: BindingDiagnosticKind,
        primary_id: u64,
        secondary_id: u64,
    ) -> Self {
        Self {
            coordinate: DiagnosticCoordinate::new(source, kind.code(), primary_id, secondary_id),
        }
    }
}

impl Ord for BindingDiagnostic {
    fn cmp(&self, other: &Self) -> Ordering {
        self.coordinate.cmp(&other.coordinate)
    }
}

impl PartialOrd for BindingDiagnostic {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct BindingGraphDocument {
    pub(crate) bindings: Vec<BindingDeclaration>,
}

impl BindingGraphDocument {
    pub(crate) fn build(
        mut bindings: Vec<BindingDeclaration>,
        target_document: &crate::static_ir::StaticUiDocument,
    ) -> Result<Self, Vec<BindingDiagnostic>> {
        bindings.sort();
        let mut diagnostics = Vec::new();

        // 1. Initial independent structural validation
        for (i, b) in bindings.iter().enumerate() {
            // Check MissingTargetNode
            if !target_document
                .nodes()
                .iter()
                .any(|n| n.id() == b.target.node)
            {
                diagnostics.push(BindingDiagnostic::new(
                    b.source_ref,
                    BindingDiagnosticKind::MissingTargetNode,
                    b.id.0,
                    b.target.node.raw(),
                ));
            }

            // Check duplicate BindingId
            if bindings[..i].iter().any(|prev| prev.id == b.id) {
                diagnostics.push(BindingDiagnostic::new(
                    b.source_ref,
                    BindingDiagnosticKind::DuplicateBindingId,
                    b.id.0,
                    0,
                ));
            }

            // Check duplicate Target Slot
            if bindings[..i].iter().any(|prev| prev.target == b.target) {
                diagnostics.push(BindingDiagnostic::new(
                    b.source_ref,
                    BindingDiagnosticKind::DuplicateTargetSlot,
                    b.id.0,
                    0,
                ));
            }

            // Semantic ref checks
            if let Some(s) = b.semantic_value {
                if s.raw() == 0 {
                    diagnostics.push(BindingDiagnostic::new(
                        b.source_ref,
                        BindingDiagnosticKind::ZeroSemanticRef,
                        b.id.0,
                        0,
                    ));
                }
            }

            // Collection key checks
            if b.domain == BindingValueDomain::Collection && b.collection_key.is_none() {
                diagnostics.push(BindingDiagnostic::new(
                    b.source_ref,
                    BindingDiagnosticKind::MissingCollectionKey,
                    b.id.0,
                    0,
                ));
            } else if b.domain != BindingValueDomain::Collection && b.collection_key.is_some() {
                diagnostics.push(BindingDiagnostic::new(
                    b.source_ref,
                    BindingDiagnosticKind::InvalidCollectionKey,
                    b.id.0,
                    0,
                ));
            }

            // Dependency checks
            for (j, dep) in b.dependencies.iter().enumerate() {
                if b.dependencies[..j].contains(dep) {
                    diagnostics.push(BindingDiagnostic::new(
                        b.source_ref,
                        BindingDiagnosticKind::DuplicateDependency,
                        b.id.0,
                        dep.target_binding.0,
                    ));
                }
                if dep.target_binding == b.id {
                    diagnostics.push(BindingDiagnostic::new(
                        b.source_ref,
                        BindingDiagnosticKind::SelfDependency,
                        b.id.0,
                        dep.target_binding.0,
                    ));
                }
                if !bindings.iter().any(|cand| cand.id == dep.target_binding) {
                    diagnostics.push(BindingDiagnostic::new(
                        b.source_ref,
                        BindingDiagnosticKind::DanglingDependency,
                        b.id.0,
                        dep.target_binding.0,
                    ));
                }
            }
        }

        // 2. Cycle detection (iterative)
        for b in &bindings {
            let mut visited = alloc::vec::Vec::new();
            let mut stack = alloc::vec::Vec::new();
            stack.push(b.id);
            visited.push(b.id);

            while let Some(current_id) = stack.pop() {
                if let Some(current) = bindings.iter().find(|cand| cand.id == current_id) {
                    for dep in &current.dependencies {
                        if dep.target_binding == b.id {
                            diagnostics.push(BindingDiagnostic::new(
                                b.source_ref,
                                BindingDiagnosticKind::Cycle,
                                b.id.0,
                                dep.target_binding.0,
                            ));
                            break; // Avoid infinite loops on this path, log once for root
                        } else if !visited.contains(&dep.target_binding) {
                            visited.push(dep.target_binding);
                            stack.push(dep.target_binding);
                        }
                    }
                }
            }
        }

        if !diagnostics.is_empty() {
            diagnostics.sort();
            diagnostics.dedup();
            return Err(diagnostics);
        }

        // Canonical normalization
        for binding in &mut bindings {
            binding.dependencies.sort();
            binding.dependencies.dedup();
        }
        bindings.sort();

        Ok(Self { bindings })
    }
}
