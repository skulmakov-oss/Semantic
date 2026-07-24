//! `PreparedActiveProjectionTargets` evidence, activation-scoped.
//!
//! This module implements the crate-private producer frozen in
//! `docs/spec/ui/shell_player_session_state_v0.md` section 7.1.9.3. It
//! derives deterministic, fabrication-resistant declared-target evidence
//! from one coherent validated projection-owned structural input: the
//! canonical `StaticUiDocument` (node anchors), the qualified
//! `BindingGraphDocument` (binding anchors), and the explicit
//! `QualifiedCollectionAnchorDeclarations` (collection anchors — the sole
//! source of collection-anchor membership; never inferred).
#![allow(dead_code)]

use alloc::vec::Vec;

use crate::binding_graph::BindingGraphDocument;
use crate::collection_anchor_declarations::QualifiedCollectionAnchorDeclarations;
use crate::static_ir::StaticUiDocument;
use crate::target_anchor::{BindingAnchor, CollectionAnchor, NodeAnchor};

/// Prepared activation-target evidence for exactly one activated projection.
/// See contract section 7.1.9.3.
///
/// Same-crate private construction is allowed only through
/// [`derive_prepared_active_projection_targets`]. It is opaque outside
/// `prom-ui`, immutable after derivation, read-only, and it is not the
/// runtime catalog itself, `ProjectionPatch` operations, or action
/// authorization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparedActiveProjectionTargets {
    node_anchors: Vec<NodeAnchor>,
    binding_anchors: Vec<BindingAnchor>,
    collection_anchors: Vec<CollectionAnchor>,
}

impl PreparedActiveProjectionTargets {
    fn new(
        mut node_anchors: Vec<NodeAnchor>,
        mut binding_anchors: Vec<BindingAnchor>,
        mut collection_anchors: Vec<CollectionAnchor>,
    ) -> Self {
        node_anchors.sort();
        binding_anchors.sort();
        collection_anchors.sort();
        Self {
            node_anchors,
            binding_anchors,
            collection_anchors,
        }
    }

    /// Declared `NodeAnchor` coordinates, in deterministic ascending order.
    pub(crate) fn node_anchors(&self) -> &[NodeAnchor] {
        &self.node_anchors
    }

    /// Declared `BindingAnchor` coordinates, in deterministic ascending
    /// order.
    pub(crate) fn binding_anchors(&self) -> &[BindingAnchor] {
        &self.binding_anchors
    }

    /// Declared `CollectionAnchor` coordinates, in deterministic ascending
    /// order. Sourced only from explicit `CollectionAnchor` declarations.
    pub(crate) fn collection_anchors(&self) -> &[CollectionAnchor] {
        &self.collection_anchors
    }
}

/// Rejects derivation when the supplied structural input is not coherent:
/// the qualified `CollectionAnchor` declarations must have been qualified
/// against the same document identity, revision, and epoch as the supplied
/// `StaticUiDocument`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PreparedActiveProjectionTargetsError {
    IncoherentCollectionAnchorSource,
}

/// Deterministically derives [`PreparedActiveProjectionTargets`] from one
/// coherent validated projection-owned structural input.
///
/// `static_document` is the sole source of `NodeAnchor` membership: every
/// node in the canonical document is an activatable node-availability
/// target. `binding_graph` is the sole source of `BindingAnchor` membership:
/// every qualified declared binding is an activatable binding-value target.
/// `collection_anchors` is the sole source of `CollectionAnchor` membership
/// — never inferred from node existence, role text, `CollectionKey`, a
/// collection-domain binding, or a collection patch operation.
pub(crate) fn derive_prepared_active_projection_targets(
    static_document: &StaticUiDocument,
    binding_graph: &BindingGraphDocument,
    collection_anchors: &QualifiedCollectionAnchorDeclarations,
) -> Result<PreparedActiveProjectionTargets, PreparedActiveProjectionTargetsError> {
    if collection_anchors.document_id() != static_document.id()
        || collection_anchors.revision() != static_document.revision()
        || collection_anchors.epoch() != static_document.epoch()
    {
        return Err(PreparedActiveProjectionTargetsError::IncoherentCollectionAnchorSource);
    }

    let node_anchors = static_document
        .nodes()
        .iter()
        .map(|node| NodeAnchor::new(node.id()))
        .collect();

    let binding_anchors = binding_graph
        .bindings
        .iter()
        .map(|declaration| BindingAnchor::new(declaration.target.node, declaration.target.slot))
        .collect();

    let collection_anchor_list = collection_anchors
        .anchors()
        .iter()
        .map(|id| CollectionAnchor::new(*id))
        .collect();

    Ok(PreparedActiveProjectionTargets::new(
        node_anchors,
        binding_anchors,
        collection_anchor_list,
    ))
}
