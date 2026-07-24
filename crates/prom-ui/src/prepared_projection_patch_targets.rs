//! `PreparedProjectionPatchTargets` evidence, transition-scoped.
//!
//! This module implements the crate-private producer frozen in
//! `docs/spec/ui/shell_player_session_state_v0.md` section 7.1.9.2. It
//! derives deterministic, fabrication-resistant stable-target evidence
//! (`NodeAnchor` / `BindingAnchor` / `CollectionAnchor`) from one admitted
//! private [`ProjectionPatchSet`], reusing its existing replay ordering.
#![allow(dead_code)]

use alloc::vec::Vec;

use crate::contract_primitives::{Epoch, StaticDocumentId};
use crate::projection_patch::{
    ProjectionPatchOperation, ProjectionPatchReplayCoordinate, ProjectionPatchReplayError,
    ProjectionPatchSet, ProjectionPatchStreamId,
};
use crate::target_anchor::{BindingAnchor, CollectionAnchor, NodeAnchor, TargetAnchor};

/// One ordered stable-target manifest entry: the coordinate reused from the
/// admitted `ProjectionPatch` replay trace plus the classified target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct PreparedTargetEntry {
    coordinate: ProjectionPatchReplayCoordinate,
    anchor: TargetAnchor,
}

impl PreparedTargetEntry {
    pub(crate) const fn coordinate(self) -> ProjectionPatchReplayCoordinate {
        self.coordinate
    }

    pub(crate) const fn anchor(self) -> TargetAnchor {
        self.anchor
    }
}

/// Immutable batch-binding metadata identifying the originating transition,
/// sufficient to prevent mixing evidence from different prepared
/// transitions. See contract section 7.1.9.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct PreparedBatchBinding {
    stream_id: ProjectionPatchStreamId,
    document_id: StaticDocumentId,
    epoch: Epoch,
}

impl PreparedBatchBinding {
    pub(crate) const fn stream_id(self) -> ProjectionPatchStreamId {
        self.stream_id
    }

    pub(crate) const fn document_id(self) -> StaticDocumentId {
        self.document_id
    }

    pub(crate) const fn epoch(self) -> Epoch {
        self.epoch
    }
}

/// Prepared transition-target evidence for exactly one admitted
/// `ProjectionPatch` batch. See contract section 7.1.9.2.
///
/// Same-crate private construction is allowed only through
/// [`derive_prepared_projection_patch_targets`]. It is opaque outside
/// `prom-ui`, immutable after derivation, and it is not
/// `PreparedActiveProjectionTargets`, the runtime catalog, patch-application
/// evidence, or action authorization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreparedProjectionPatchTargets {
    batch_binding: Option<PreparedBatchBinding>,
    entries: Vec<PreparedTargetEntry>,
    target_reference_count: usize,
}

impl PreparedProjectionPatchTargets {
    fn new(batch_binding: Option<PreparedBatchBinding>, entries: Vec<PreparedTargetEntry>) -> Self {
        let target_reference_count = entries.len();
        Self {
            batch_binding,
            entries,
            target_reference_count,
        }
    }

    /// Identifies the originating transition. `None` only for an empty
    /// patch set, which carries no batch to bind to.
    pub(crate) fn batch_binding(&self) -> Option<PreparedBatchBinding> {
        self.batch_binding
    }

    /// The ordered stable-target manifest, in stable
    /// `(patch_ordinal, operation_ordinal)` order.
    pub(crate) fn entries(&self) -> &[PreparedTargetEntry] {
        &self.entries
    }

    /// The actual manifest target-reference count, derived from the
    /// manifest shape itself. See contract section 7.1.9.8.
    pub(crate) const fn target_reference_count(&self) -> usize {
        self.target_reference_count
    }
}

/// Deterministically derives [`PreparedProjectionPatchTargets`] from one
/// admitted private `ProjectionPatch` batch.
///
/// Every existing patch operation targets exactly one stable-target anchor:
/// `SetBindingValue` targets a `BindingAnchor`, `SetNodeAvailability` targets
/// a `NodeAnchor`, and every collection operation targets a
/// `CollectionAnchor` identified by its collection node. Manifest order
/// reuses the admitted batch's own deterministic replay order; this function
/// does not reorder, sort, or deduplicate entries.
pub(crate) fn derive_prepared_projection_patch_targets(
    patch_set: &ProjectionPatchSet,
) -> Result<PreparedProjectionPatchTargets, ProjectionPatchReplayError> {
    let trace = patch_set.replay_trace()?;

    let mut entries = Vec::new();
    for step in trace.steps() {
        entries.push(PreparedTargetEntry {
            coordinate: step.coordinate(),
            anchor: target_anchor_for_operation(step.operation()),
        });
    }

    let batch_binding = patch_set.patches().first().map(|patch| {
        let envelope = patch.envelope();
        PreparedBatchBinding {
            stream_id: envelope.stream_id(),
            document_id: envelope.document_id(),
            epoch: envelope.epoch(),
        }
    });

    Ok(PreparedProjectionPatchTargets::new(batch_binding, entries))
}

fn target_anchor_for_operation(operation: &ProjectionPatchOperation) -> TargetAnchor {
    match operation {
        ProjectionPatchOperation::SetBindingValue { target, .. } => {
            TargetAnchor::Binding(BindingAnchor::new(target.node, target.slot))
        }
        ProjectionPatchOperation::SetNodeAvailability { node, .. } => {
            TargetAnchor::Node(NodeAnchor::new(*node))
        }
        ProjectionPatchOperation::CollectionInsert { collection, .. }
        | ProjectionPatchOperation::CollectionUpdate { collection, .. }
        | ProjectionPatchOperation::CollectionRemove { collection, .. }
        | ProjectionPatchOperation::CollectionMove { collection, .. } => {
            TargetAnchor::Collection(CollectionAnchor::new(*collection))
        }
    }
}
