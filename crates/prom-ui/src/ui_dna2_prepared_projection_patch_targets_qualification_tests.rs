//! UI-DNA2 `PreparedProjectionPatchTargets` qualification tests.
//!
//! These tests qualify the crate-private, pure in-memory
//! `ProjectionPatchSet` -> `PreparedProjectionPatchTargets` derivation path
//! only. They do not exercise `PreparedActiveProjectionTargets`, the runtime
//! catalog, stage-4/5/6, patch application, or public-API surfaces, which
//! remain separately unauthorized.

use alloc::vec;
use alloc::vec::Vec;

use crate::binding_graph::{BindingSlot, BindingTarget};
use crate::contract_primitives::{CollectionKey, Epoch, Revision, StaticDocumentId, StaticNodeId};
use crate::prepared_projection_patch_targets::derive_prepared_projection_patch_targets;
use crate::projection_patch::{
    ProjectionNodeAvailability, ProjectionPatch, ProjectionPatchEnvelope, ProjectionPatchId,
    ProjectionPatchOperation, ProjectionPatchSequence, ProjectionPatchSet, ProjectionPatchStreamId,
    ProjectionPatchValue, ProjectionQuadState,
};
use crate::target_anchor::TargetAnchor;

fn node(raw: u64) -> StaticNodeId {
    StaticNodeId::new(raw).expect("static node id")
}

fn doc(raw: u64) -> StaticDocumentId {
    StaticDocumentId::new(raw).expect("static document id")
}

fn stream(raw: u64) -> ProjectionPatchStreamId {
    ProjectionPatchStreamId::new(raw).expect("stream id")
}

fn patch_id(raw: u64) -> ProjectionPatchId {
    ProjectionPatchId::new(raw).expect("patch id")
}

fn envelope(patch: u64, seq: u64, prev_rev: u64, rev: u64) -> ProjectionPatchEnvelope {
    ProjectionPatchEnvelope::new(
        patch_id(patch),
        stream(1),
        doc(1),
        None,
        Revision::new(prev_rev),
        Revision::new(rev),
        Epoch::new(1),
        ProjectionPatchSequence::new(seq),
    )
}

fn patch(
    patch_no: u64,
    seq: u64,
    prev_rev: u64,
    rev: u64,
    operations: Vec<ProjectionPatchOperation>,
) -> ProjectionPatch {
    ProjectionPatch::new(envelope(patch_no, seq, prev_rev, rev), operations).expect("valid patch")
}

fn set_binding(node_raw: u64, slot_raw: u32) -> ProjectionPatchOperation {
    ProjectionPatchOperation::SetBindingValue {
        target: BindingTarget {
            node: node(node_raw),
            slot: BindingSlot(slot_raw),
        },
        value: ProjectionPatchValue::Quad(ProjectionQuadState::T),
    }
}

fn set_availability(node_raw: u64) -> ProjectionPatchOperation {
    ProjectionPatchOperation::SetNodeAvailability {
        node: node(node_raw),
        availability: ProjectionNodeAvailability::Available,
    }
}

fn key(raw: u64) -> CollectionKey {
    CollectionKey::new(raw).expect("collection key")
}

fn collection_insert(collection_raw: u64, key_raw: u64) -> ProjectionPatchOperation {
    ProjectionPatchOperation::CollectionInsert {
        collection: node(collection_raw),
        key: key(key_raw),
        before: None,
        value: ProjectionPatchValue::UnsignedScalar(1),
    }
}

fn collection_update(collection_raw: u64, key_raw: u64) -> ProjectionPatchOperation {
    ProjectionPatchOperation::CollectionUpdate {
        collection: node(collection_raw),
        key: key(key_raw),
        value: ProjectionPatchValue::UnsignedScalar(2),
    }
}

fn collection_remove(collection_raw: u64, key_raw: u64) -> ProjectionPatchOperation {
    ProjectionPatchOperation::CollectionRemove {
        collection: node(collection_raw),
        key: key(key_raw),
    }
}

fn collection_move(collection_raw: u64, key_raw: u64) -> ProjectionPatchOperation {
    ProjectionPatchOperation::CollectionMove {
        collection: node(collection_raw),
        key: key(key_raw),
        before: None,
    }
}

#[test]
fn test_node_availability_operation_produces_node_anchor() {
    let set = ProjectionPatchSet::new(vec![patch(1, 1, 0, 1, vec![set_availability(5)])])
        .expect("valid set");
    let prepared = derive_prepared_projection_patch_targets(&set).expect("derivation succeeds");

    assert_eq!(prepared.target_reference_count(), 1);
    match prepared.entries()[0].anchor() {
        TargetAnchor::Node(anchor) => assert_eq!(anchor.node(), node(5)),
        other => panic!("expected NodeAnchor, got {other:?}"),
    }
}

#[test]
fn test_binding_value_operation_produces_binding_anchor() {
    let set = ProjectionPatchSet::new(vec![patch(1, 1, 0, 1, vec![set_binding(7, 3)])])
        .expect("valid set");
    let prepared = derive_prepared_projection_patch_targets(&set).expect("derivation succeeds");

    assert_eq!(prepared.target_reference_count(), 1);
    match prepared.entries()[0].anchor() {
        TargetAnchor::Binding(anchor) => {
            assert_eq!(anchor.node(), node(7));
            assert_eq!(anchor.slot(), BindingSlot(3));
        }
        other => panic!("expected BindingAnchor, got {other:?}"),
    }
}

#[test]
fn test_every_collection_operation_kind_produces_collection_anchor() {
    let ops = vec![
        collection_insert(9, 1),
        collection_update(9, 1),
        collection_remove(9, 1),
        collection_move(9, 2),
    ];
    let set = ProjectionPatchSet::new(vec![patch(1, 1, 0, 1, ops)]).expect("valid set");
    let prepared = derive_prepared_projection_patch_targets(&set).expect("derivation succeeds");

    assert_eq!(prepared.target_reference_count(), 4);
    for entry in prepared.entries() {
        match entry.anchor() {
            TargetAnchor::Collection(anchor) => assert_eq!(anchor.collection(), node(9)),
            other => panic!("expected CollectionAnchor, got {other:?}"),
        }
    }
}

#[test]
fn test_mixed_operation_kinds_classify_independently() {
    let ops = vec![
        set_availability(1),
        set_binding(2, 0),
        collection_insert(3, 1),
    ];
    let set = ProjectionPatchSet::new(vec![patch(1, 1, 0, 1, ops)]).expect("valid set");
    let prepared = derive_prepared_projection_patch_targets(&set).expect("derivation succeeds");

    assert_eq!(prepared.target_reference_count(), 3);
    assert!(matches!(
        prepared.entries()[0].anchor(),
        TargetAnchor::Node(_)
    ));
    assert!(matches!(
        prepared.entries()[1].anchor(),
        TargetAnchor::Binding(_)
    ));
    assert!(matches!(
        prepared.entries()[2].anchor(),
        TargetAnchor::Collection(_)
    ));
}

#[test]
fn test_stable_order_within_one_patch() {
    let ops = vec![
        set_availability(1),
        set_availability(2),
        set_availability(3),
    ];
    let set = ProjectionPatchSet::new(vec![patch(1, 1, 0, 1, ops)]).expect("valid set");
    let prepared = derive_prepared_projection_patch_targets(&set).expect("derivation succeeds");

    let coordinates: Vec<_> = prepared.entries().iter().map(|e| e.coordinate()).collect();
    let mut sorted = coordinates.clone();
    sorted.sort();
    assert_eq!(
        coordinates, sorted,
        "manifest must already be in stable order"
    );
    assert_eq!(coordinates[0].operation_ordinal(), 0);
    assert_eq!(coordinates[1].operation_ordinal(), 1);
    assert_eq!(coordinates[2].operation_ordinal(), 2);
}

#[test]
fn test_stable_order_across_multiple_patches() {
    let set = ProjectionPatchSet::new(vec![
        patch(1, 1, 0, 1, vec![set_availability(1), set_availability(2)]),
        patch(2, 2, 1, 2, vec![set_availability(3)]),
    ])
    .expect("valid set");
    let prepared = derive_prepared_projection_patch_targets(&set).expect("derivation succeeds");

    assert_eq!(prepared.target_reference_count(), 3);
    let coordinates: Vec<_> = prepared.entries().iter().map(|e| e.coordinate()).collect();
    assert_eq!(coordinates[0].patch_ordinal(), 0);
    assert_eq!(coordinates[0].operation_ordinal(), 0);
    assert_eq!(coordinates[1].patch_ordinal(), 0);
    assert_eq!(coordinates[1].operation_ordinal(), 1);
    assert_eq!(coordinates[2].patch_ordinal(), 1);
    assert_eq!(coordinates[2].operation_ordinal(), 0);
}

#[test]
fn test_target_reference_count_matches_entry_count() {
    let set = ProjectionPatchSet::new(vec![patch(
        1,
        1,
        0,
        1,
        vec![
            set_availability(1),
            set_binding(2, 0),
            collection_insert(3, 1),
        ],
    )])
    .expect("valid set");
    let prepared = derive_prepared_projection_patch_targets(&set).expect("derivation succeeds");

    assert_eq!(prepared.target_reference_count(), prepared.entries().len());
}

#[test]
fn test_empty_patch_set_produces_empty_manifest() {
    let set = ProjectionPatchSet::new(Vec::new()).expect("empty set is valid");
    let prepared = derive_prepared_projection_patch_targets(&set).expect("derivation succeeds");

    assert_eq!(prepared.target_reference_count(), 0);
    assert!(prepared.entries().is_empty());
    assert_eq!(prepared.batch_binding(), None);
}

#[test]
fn test_batch_binding_reflects_first_patch_envelope() {
    let set = ProjectionPatchSet::new(vec![patch(1, 1, 0, 1, vec![set_availability(1)])])
        .expect("valid set");
    let prepared = derive_prepared_projection_patch_targets(&set).expect("derivation succeeds");

    let binding = prepared
        .batch_binding()
        .expect("non-empty set has a binding");
    assert_eq!(binding.stream_id(), stream(1));
    assert_eq!(binding.document_id(), doc(1));
    assert_eq!(binding.epoch(), Epoch::new(1));
}

#[test]
fn test_batches_touching_different_targets_produce_distinguishable_evidence() {
    let set_a = ProjectionPatchSet::new(vec![patch(1, 1, 0, 1, vec![set_availability(1)])])
        .expect("valid set");
    let set_b = ProjectionPatchSet::new(vec![patch(1, 1, 0, 1, vec![set_availability(2)])])
        .expect("valid set");

    let prepared_a = derive_prepared_projection_patch_targets(&set_a).expect("derivation succeeds");
    let prepared_b = derive_prepared_projection_patch_targets(&set_b).expect("derivation succeeds");

    assert_ne!(prepared_a, prepared_b);
}

#[test]
fn test_evidence_depends_only_on_target_structure_not_sequence_or_revision() {
    // The manifest is target evidence, not a transaction fingerprint: two
    // envelopes that differ only in sequence/revision but touch the same
    // targets in the same order must derive equal evidence.
    let set_a = ProjectionPatchSet::new(vec![patch(1, 1, 0, 1, vec![set_availability(1)])])
        .expect("valid set");
    let set_b = ProjectionPatchSet::new(vec![patch(1, 5, 4, 5, vec![set_availability(1)])])
        .expect("valid set");

    let prepared_a = derive_prepared_projection_patch_targets(&set_a).expect("derivation succeeds");
    let prepared_b = derive_prepared_projection_patch_targets(&set_b).expect("derivation succeeds");

    assert_eq!(prepared_a, prepared_b);
}

#[test]
fn test_determinism_identical_input_produces_identical_output() {
    let set = ProjectionPatchSet::new(vec![patch(
        1,
        1,
        0,
        1,
        vec![
            set_availability(1),
            set_binding(2, 0),
            collection_insert(3, 1),
        ],
    )])
    .expect("valid set");

    let prepared_1 = derive_prepared_projection_patch_targets(&set).expect("derivation succeeds");
    let prepared_2 = derive_prepared_projection_patch_targets(&set).expect("derivation succeeds");

    assert_eq!(prepared_1, prepared_2);
}
