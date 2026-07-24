//! UI-DNA2 `LocalProjectionState` / patch-application qualification tests.
//!
//! These tests qualify the crate-private, pure in-memory
//! `LocalProjectionState` + `apply_projection_patch_set` path only. They do
//! not exercise stage-4/5/6, the runtime catalog, replay-cursor
//! advancement, or public-API surfaces.

use alloc::vec;
use alloc::vec::Vec;

use crate::binding_graph::{BindingSlot, BindingTarget};
use crate::contract_primitives::{CollectionKey, Epoch, Revision, StaticDocumentId, StaticNodeId};
use crate::local_projection_state::{
    apply_projection_patch_set, LocalProjectionApplicationError, LocalProjectionState,
};
use crate::projection_patch::{
    ProjectionNodeAvailability, ProjectionPatch, ProjectionPatchEnvelope, ProjectionPatchId,
    ProjectionPatchOperation, ProjectionPatchSequence, ProjectionPatchSet, ProjectionPatchStreamId,
    ProjectionPatchValue, ProjectionQuadState,
};

fn node(raw: u64) -> StaticNodeId {
    StaticNodeId::new(raw).expect("static node id")
}

fn key(raw: u64) -> CollectionKey {
    CollectionKey::new(raw).expect("collection key")
}

fn set_of(operations: Vec<ProjectionPatchOperation>) -> ProjectionPatchSet {
    let envelope = ProjectionPatchEnvelope::new(
        ProjectionPatchId::new(1).expect("patch id"),
        ProjectionPatchStreamId::new(1).expect("stream id"),
        StaticDocumentId::new(1).expect("doc id"),
        None,
        Revision::new(0),
        Revision::new(1),
        Epoch::new(1),
        ProjectionPatchSequence::new(1),
    );
    let patch = ProjectionPatch::new(envelope, operations).expect("valid patch");
    ProjectionPatchSet::new(vec![patch]).expect("valid set")
}

fn set_binding(node_raw: u64, slot_raw: u32, text: &str) -> ProjectionPatchOperation {
    ProjectionPatchOperation::SetBindingValue {
        target: BindingTarget {
            node: node(node_raw),
            slot: BindingSlot(slot_raw),
        },
        value: ProjectionPatchValue::Text(text.into()),
    }
}

fn set_availability(
    node_raw: u64,
    availability: ProjectionNodeAvailability,
) -> ProjectionPatchOperation {
    ProjectionPatchOperation::SetNodeAvailability {
        node: node(node_raw),
        availability,
    }
}

fn insert(collection: u64, k: u64, before: Option<u64>, value: i64) -> ProjectionPatchOperation {
    ProjectionPatchOperation::CollectionInsert {
        collection: node(collection),
        key: key(k),
        before: before.map(key),
        value: ProjectionPatchValue::SignedScalar(value),
    }
}

fn update(collection: u64, k: u64, value: i64) -> ProjectionPatchOperation {
    ProjectionPatchOperation::CollectionUpdate {
        collection: node(collection),
        key: key(k),
        value: ProjectionPatchValue::SignedScalar(value),
    }
}

fn remove(collection: u64, k: u64) -> ProjectionPatchOperation {
    ProjectionPatchOperation::CollectionRemove {
        collection: node(collection),
        key: key(k),
    }
}

fn move_op(collection: u64, k: u64, before: Option<u64>) -> ProjectionPatchOperation {
    ProjectionPatchOperation::CollectionMove {
        collection: node(collection),
        key: key(k),
        before: before.map(key),
    }
}

#[test]
fn test_empty_state_has_no_values() {
    let state = LocalProjectionState::empty();
    assert_eq!(state.binding_value(node(1), BindingSlot(0)), None);
    assert_eq!(state.node_availability(node(1)), None);
    assert!(state.collection_entries(node(1)).is_empty());
}

#[test]
fn test_set_binding_value_upserts() {
    let previous = LocalProjectionState::empty();
    let set = set_of(vec![set_binding(1, 0, "hello")]);
    let candidate = apply_projection_patch_set(&previous, &set).expect("applies");
    assert_eq!(
        candidate.binding_value(node(1), BindingSlot(0)),
        Some(&ProjectionPatchValue::Text("hello".into()))
    );

    let set2 = set_of(vec![set_binding(1, 0, "world")]);
    let candidate2 = apply_projection_patch_set(&candidate, &set2).expect("applies");
    assert_eq!(
        candidate2.binding_value(node(1), BindingSlot(0)),
        Some(&ProjectionPatchValue::Text("world".into()))
    );
}

#[test]
fn test_set_node_availability_upserts() {
    let previous = LocalProjectionState::empty();
    let set = set_of(vec![set_availability(
        1,
        ProjectionNodeAvailability::Unavailable,
    )]);
    let candidate = apply_projection_patch_set(&previous, &set).expect("applies");
    assert_eq!(
        candidate.node_availability(node(1)),
        Some(ProjectionNodeAvailability::Unavailable)
    );

    let set2 = set_of(vec![set_availability(
        1,
        ProjectionNodeAvailability::Available,
    )]);
    let candidate2 = apply_projection_patch_set(&candidate, &set2).expect("applies");
    assert_eq!(
        candidate2.node_availability(node(1)),
        Some(ProjectionNodeAvailability::Available)
    );
}

#[test]
fn test_collection_insert_appends_in_order() {
    let previous = LocalProjectionState::empty();
    let set = set_of(vec![
        insert(9, 1, None, 10),
        insert(9, 2, None, 20),
        insert(9, 3, None, 30),
    ]);
    let candidate = apply_projection_patch_set(&previous, &set).expect("applies");
    let entries = candidate.collection_entries(node(9));
    assert_eq!(
        entries,
        &[
            (key(1), ProjectionPatchValue::SignedScalar(10)),
            (key(2), ProjectionPatchValue::SignedScalar(20)),
            (key(3), ProjectionPatchValue::SignedScalar(30)),
        ]
    );
}

#[test]
fn test_collection_insert_before_places_at_correct_position() {
    let previous = LocalProjectionState::empty();
    let set = set_of(vec![
        insert(9, 1, None, 10),
        insert(9, 3, None, 30),
        insert(9, 2, Some(3), 20),
    ]);
    let candidate = apply_projection_patch_set(&previous, &set).expect("applies");
    let keys: Vec<_> = candidate
        .collection_entries(node(9))
        .iter()
        .map(|(k, _)| *k)
        .collect();
    assert_eq!(keys, vec![key(1), key(2), key(3)]);
}

#[test]
fn test_collection_insert_duplicate_key_across_transitions_is_rejected() {
    // A single patch cannot target the same collection key twice (rejected
    // earlier by `ProjectionPatch` construction itself as
    // `PP_DUP_MUTATION_TARGET`). This exercises the cross-transition case:
    // a key already committed from an earlier transition, re-inserted by a
    // later one.
    let previous = LocalProjectionState::empty();
    let first = set_of(vec![insert(9, 1, None, 10)]);
    let committed = apply_projection_patch_set(&previous, &first).expect("first applies");

    let second = set_of(vec![insert(9, 1, None, 99)]);
    let err = apply_projection_patch_set(&committed, &second).expect_err("duplicate key rejected");
    assert_eq!(
        err,
        LocalProjectionApplicationError::KeyAlreadyPresent {
            collection: node(9),
            key: key(1),
        }
    );
}

#[test]
fn test_collection_update_missing_key_is_rejected() {
    let previous = LocalProjectionState::empty();
    let set = set_of(vec![update(9, 1, 99)]);
    let err = apply_projection_patch_set(&previous, &set).expect_err("missing key rejected");
    assert_eq!(
        err,
        LocalProjectionApplicationError::KeyMissing {
            collection: node(9),
            key: key(1),
        }
    );
}

#[test]
fn test_collection_update_replaces_value() {
    let previous = LocalProjectionState::empty();
    let set = set_of(vec![insert(9, 1, None, 10), update(9, 1, 99)]);
    let candidate = apply_projection_patch_set(&previous, &set).expect("applies");
    assert_eq!(
        candidate.collection_entries(node(9)),
        &[(key(1), ProjectionPatchValue::SignedScalar(99))]
    );
}

#[test]
fn test_collection_remove_missing_key_is_rejected() {
    let previous = LocalProjectionState::empty();
    let set = set_of(vec![remove(9, 1)]);
    let err = apply_projection_patch_set(&previous, &set).expect_err("missing key rejected");
    assert_eq!(
        err,
        LocalProjectionApplicationError::KeyMissing {
            collection: node(9),
            key: key(1),
        }
    );
}

#[test]
fn test_collection_remove_deletes_entry() {
    let previous = LocalProjectionState::empty();
    let set = set_of(vec![
        insert(9, 1, None, 10),
        insert(9, 2, None, 20),
        remove(9, 1),
    ]);
    let candidate = apply_projection_patch_set(&previous, &set).expect("applies");
    assert_eq!(
        candidate.collection_entries(node(9)),
        &[(key(2), ProjectionPatchValue::SignedScalar(20))]
    );
}

#[test]
fn test_collection_move_reorders_entry() {
    let previous = LocalProjectionState::empty();
    let set = set_of(vec![
        insert(9, 1, None, 10),
        insert(9, 2, None, 20),
        insert(9, 3, None, 30),
        move_op(9, 3, Some(1)),
    ]);
    let candidate = apply_projection_patch_set(&previous, &set).expect("applies");
    let keys: Vec<_> = candidate
        .collection_entries(node(9))
        .iter()
        .map(|(k, _)| *k)
        .collect();
    assert_eq!(keys, vec![key(3), key(1), key(2)]);
}

#[test]
fn test_collection_move_to_end_when_before_is_none() {
    let previous = LocalProjectionState::empty();
    let set = set_of(vec![
        insert(9, 1, None, 10),
        insert(9, 2, None, 20),
        move_op(9, 1, None),
    ]);
    let candidate = apply_projection_patch_set(&previous, &set).expect("applies");
    let keys: Vec<_> = candidate
        .collection_entries(node(9))
        .iter()
        .map(|(k, _)| *k)
        .collect();
    assert_eq!(keys, vec![key(2), key(1)]);
}

#[test]
fn test_collection_move_missing_key_is_rejected() {
    let previous = LocalProjectionState::empty();
    let set = set_of(vec![move_op(9, 1, None)]);
    let err = apply_projection_patch_set(&previous, &set).expect_err("missing key rejected");
    assert_eq!(
        err,
        LocalProjectionApplicationError::KeyMissing {
            collection: node(9),
            key: key(1),
        }
    );
}

#[test]
fn test_collection_move_missing_before_key_is_rejected() {
    let previous = LocalProjectionState::empty();
    let set = set_of(vec![insert(9, 1, None, 10), move_op(9, 1, Some(999))]);
    let err = apply_projection_patch_set(&previous, &set).expect_err("missing before key rejected");
    assert_eq!(
        err,
        LocalProjectionApplicationError::BeforeKeyMissing {
            collection: node(9),
            before: key(999),
        }
    );
}

#[test]
fn test_collection_insert_missing_before_key_is_rejected() {
    let previous = LocalProjectionState::empty();
    let set = set_of(vec![insert(9, 1, Some(999), 10)]);
    let err = apply_projection_patch_set(&previous, &set).expect_err("missing before key rejected");
    assert_eq!(
        err,
        LocalProjectionApplicationError::BeforeKeyMissing {
            collection: node(9),
            before: key(999),
        }
    );
}

#[test]
fn test_atomic_no_partial_application_on_failure() {
    let mut previous = LocalProjectionState::empty();
    let baseline = set_of(vec![
        set_binding(1, 0, "baseline"),
        set_availability(2, ProjectionNodeAvailability::Available),
        insert(9, 1, None, 10),
    ]);
    previous = apply_projection_patch_set(&previous, &baseline).expect("baseline applies");
    let before = previous.clone();

    // Second and third operations would succeed in isolation, but the
    // fourth targets a missing collection key and must fail the whole batch.
    let failing = set_of(vec![
        set_binding(1, 0, "mutated"),
        set_availability(2, ProjectionNodeAvailability::Unavailable),
        insert(9, 2, None, 20),
        update(9, 999, 1), // fails: key 999 does not exist
    ]);

    let err = apply_projection_patch_set(&previous, &failing).expect_err("batch rejected");
    assert_eq!(
        err,
        LocalProjectionApplicationError::KeyMissing {
            collection: node(9),
            key: key(999),
        }
    );

    // `previous` (borrowed immutably throughout) is byte-identical to its
    // pre-call snapshot: no partial write from the rejected batch leaked.
    assert_eq!(previous, before);
    assert_eq!(
        previous.binding_value(node(1), BindingSlot(0)),
        Some(&ProjectionPatchValue::Text("baseline".into()))
    );
    assert_eq!(
        previous.node_availability(node(2)),
        Some(ProjectionNodeAvailability::Available)
    );
    assert_eq!(
        previous.collection_entries(node(9)),
        &[(key(1), ProjectionPatchValue::SignedScalar(10))]
    );
}

#[test]
fn test_multi_operation_batch_applies_in_order_across_kinds() {
    let previous = LocalProjectionState::empty();
    let set = set_of(vec![
        set_binding(1, 0, "value"),
        set_availability(2, ProjectionNodeAvailability::Hidden),
        insert(9, 1, None, 10),
        insert(9, 2, None, 20),
        update(9, 1, 11),
        move_op(9, 2, Some(1)),
        remove(9, 1),
    ]);
    let candidate = apply_projection_patch_set(&previous, &set).expect("applies");

    assert_eq!(
        candidate.binding_value(node(1), BindingSlot(0)),
        Some(&ProjectionPatchValue::Text("value".into()))
    );
    assert_eq!(
        candidate.node_availability(node(2)),
        Some(ProjectionNodeAvailability::Hidden)
    );
    assert_eq!(
        candidate.collection_entries(node(9)),
        &[(key(2), ProjectionPatchValue::SignedScalar(20))]
    );
}

#[test]
fn test_determinism_identical_input_produces_identical_output() {
    let previous = LocalProjectionState::empty();
    let set = set_of(vec![
        set_binding(1, 0, "value"),
        insert(9, 1, None, 10),
        insert(9, 2, None, 20),
    ]);
    let candidate_1 = apply_projection_patch_set(&previous, &set).expect("applies");
    let candidate_2 = apply_projection_patch_set(&previous, &set).expect("applies");
    assert_eq!(candidate_1, candidate_2);
}

#[test]
fn test_quad_and_unsigned_values_round_trip() {
    let previous = LocalProjectionState::empty();
    let set = set_of(vec![
        ProjectionPatchOperation::SetBindingValue {
            target: BindingTarget {
                node: node(1),
                slot: BindingSlot(0),
            },
            value: ProjectionPatchValue::Quad(ProjectionQuadState::T),
        },
        ProjectionPatchOperation::SetBindingValue {
            target: BindingTarget {
                node: node(1),
                slot: BindingSlot(1),
            },
            value: ProjectionPatchValue::UnsignedScalar(42),
        },
    ]);
    let candidate = apply_projection_patch_set(&previous, &set).expect("applies");
    assert_eq!(
        candidate.binding_value(node(1), BindingSlot(0)),
        Some(&ProjectionPatchValue::Quad(ProjectionQuadState::T))
    );
    assert_eq!(
        candidate.binding_value(node(1), BindingSlot(1)),
        Some(&ProjectionPatchValue::UnsignedScalar(42))
    );
}
