//! Deterministic local projection state and patch application.
//!
//! `LocalProjectionState` is the committed/candidate local projected value
//! set that a Shell Player session accumulates across successful patch-batch
//! transitions: binding values, node availability, and ordered collection
//! contents. It is local, reconstructible, session-scoped, and not Semantic
//! truth (see `docs/spec/ui/shell_player_session_state_v0.md` section 4).
//!
//! [`apply_projection_patch_set`] computes a candidate state from a previous
//! state and an admitted `ProjectionPatchSet`, atomically: it never mutates
//! `previous`, and it returns the complete candidate only if every operation
//! in the batch applies successfully. `CollectionKey` existence, absence,
//! insertion position, and move legality are operation/candidate-state
//! concerns owned by this module (see contract section 7.1.3), not stage-5
//! stable-target concerns.
#![allow(dead_code)]

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::binding_graph::BindingSlot;
use crate::contract_primitives::{CollectionKey, StaticNodeId};
use crate::projection_patch::{
    ProjectionNodeAvailability, ProjectionPatchOperation, ProjectionPatchSet, ProjectionPatchValue,
};

/// Deterministic local projected-value state for one Shell Player session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LocalProjectionState {
    bindings: BTreeMap<(StaticNodeId, BindingSlot), ProjectionPatchValue>,
    availability: BTreeMap<StaticNodeId, ProjectionNodeAvailability>,
    collections: BTreeMap<StaticNodeId, Vec<(CollectionKey, ProjectionPatchValue)>>,
}

impl LocalProjectionState {
    /// The canonical zero value: no bindings, no availability overrides, no
    /// collection contents. This is the correct initial state for a
    /// freshly activated session that has not yet committed any patch.
    pub(crate) fn empty() -> Self {
        Self {
            bindings: BTreeMap::new(),
            availability: BTreeMap::new(),
            collections: BTreeMap::new(),
        }
    }

    pub(crate) fn binding_value(
        &self,
        node: StaticNodeId,
        slot: BindingSlot,
    ) -> Option<&ProjectionPatchValue> {
        self.bindings.get(&(node, slot))
    }

    pub(crate) fn node_availability(
        &self,
        node: StaticNodeId,
    ) -> Option<ProjectionNodeAvailability> {
        self.availability.get(&node).copied()
    }

    /// Ordered `(key, value)` entries for `collection`, in current collection
    /// order. Empty (not absent) if the collection has never been touched.
    pub(crate) fn collection_entries(
        &self,
        collection: StaticNodeId,
    ) -> &[(CollectionKey, ProjectionPatchValue)] {
        self.collections
            .get(&collection)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

/// A structural application failure. `CollectionKey` existence, absence, and
/// insertion-position legality are candidate-state concerns (contract
/// section 7.1.3), evaluated here rather than at stage 5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LocalProjectionApplicationError {
    /// `CollectionInsert` targeted a key already present in the collection.
    KeyAlreadyPresent {
        collection: StaticNodeId,
        key: CollectionKey,
    },
    /// `CollectionUpdate`, `CollectionRemove`, or `CollectionMove` targeted a
    /// key absent from the collection.
    KeyMissing {
        collection: StaticNodeId,
        key: CollectionKey,
    },
    /// `CollectionInsert` or `CollectionMove` named a `before` key absent
    /// from the collection.
    BeforeKeyMissing {
        collection: StaticNodeId,
        before: CollectionKey,
    },
}

/// Deterministically computes the candidate [`LocalProjectionState`] that
/// results from applying every operation in `patch_set`, in order, to
/// `previous`.
///
/// Atomic: `previous` is never mutated, and this returns `Ok` only if every
/// operation in the complete batch applies successfully. On the first
/// structural failure, the partially built candidate is discarded and `Err`
/// is returned; the caller's `previous` value remains the complete,
/// unmodified prior state.
pub(crate) fn apply_projection_patch_set(
    previous: &LocalProjectionState,
    patch_set: &ProjectionPatchSet,
) -> Result<LocalProjectionState, LocalProjectionApplicationError> {
    let mut candidate = previous.clone();
    for patch in patch_set.patches() {
        for operation in patch.operations() {
            apply_operation(&mut candidate, operation)?;
        }
    }
    Ok(candidate)
}

fn apply_operation(
    state: &mut LocalProjectionState,
    operation: &ProjectionPatchOperation,
) -> Result<(), LocalProjectionApplicationError> {
    match operation {
        ProjectionPatchOperation::SetBindingValue { target, value } => {
            state
                .bindings
                .insert((target.node, target.slot), value.clone());
            Ok(())
        }
        ProjectionPatchOperation::SetNodeAvailability { node, availability } => {
            state.availability.insert(*node, *availability);
            Ok(())
        }
        ProjectionPatchOperation::CollectionInsert {
            collection,
            key,
            before,
            value,
        } => {
            let entries = state.collections.entry(*collection).or_default();
            if entries.iter().any(|(existing, _)| existing == key) {
                return Err(LocalProjectionApplicationError::KeyAlreadyPresent {
                    collection: *collection,
                    key: *key,
                });
            }
            let insert_at = resolve_insert_position(entries, *before, *collection)?;
            entries.insert(insert_at, (*key, value.clone()));
            Ok(())
        }
        ProjectionPatchOperation::CollectionUpdate {
            collection,
            key,
            value,
        } => {
            let entries = collection_entries_mut(state, *collection, *key)?;
            let entry = entries
                .iter_mut()
                .find(|(existing, _)| existing == key)
                .ok_or(LocalProjectionApplicationError::KeyMissing {
                    collection: *collection,
                    key: *key,
                })?;
            entry.1 = value.clone();
            Ok(())
        }
        ProjectionPatchOperation::CollectionRemove { collection, key } => {
            let entries = collection_entries_mut(state, *collection, *key)?;
            let position = entries
                .iter()
                .position(|(existing, _)| existing == key)
                .ok_or(LocalProjectionApplicationError::KeyMissing {
                    collection: *collection,
                    key: *key,
                })?;
            entries.remove(position);
            Ok(())
        }
        ProjectionPatchOperation::CollectionMove {
            collection,
            key,
            before,
        } => {
            let entries = collection_entries_mut(state, *collection, *key)?;
            let position = entries
                .iter()
                .position(|(existing, _)| existing == key)
                .ok_or(LocalProjectionApplicationError::KeyMissing {
                    collection: *collection,
                    key: *key,
                })?;
            let item = entries.remove(position);
            let insert_at = resolve_insert_position(entries, *before, *collection)?;
            entries.insert(insert_at, item);
            Ok(())
        }
    }
}

fn collection_entries_mut(
    state: &mut LocalProjectionState,
    collection: StaticNodeId,
    key: CollectionKey,
) -> Result<&mut Vec<(CollectionKey, ProjectionPatchValue)>, LocalProjectionApplicationError> {
    state
        .collections
        .get_mut(&collection)
        .ok_or(LocalProjectionApplicationError::KeyMissing { collection, key })
}

fn resolve_insert_position(
    entries: &[(CollectionKey, ProjectionPatchValue)],
    before: Option<CollectionKey>,
    collection: StaticNodeId,
) -> Result<usize, LocalProjectionApplicationError> {
    match before {
        None => Ok(entries.len()),
        Some(before_key) => entries
            .iter()
            .position(|(existing, _)| *existing == before_key)
            .ok_or(LocalProjectionApplicationError::BeforeKeyMissing {
                collection,
                before: before_key,
            }),
    }
}
