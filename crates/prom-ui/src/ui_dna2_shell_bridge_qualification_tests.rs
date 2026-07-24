//! UI-DNA2 Shell Player bridge qualification tests.
//!
//! These tests qualify the public `shell_bridge` surface: patch admission,
//! prepared-evidence snapshot derivation, activation-target snapshot
//! derivation, and local-state application/query. They exercise the same
//! surface `prom-ui-runtime` consumes across the crate boundary.

use alloc::vec;
use alloc::vec::Vec;

use crate::shell_bridge::{
    admit_projection_patch_batch, apply_admitted_patch_batch, binding_value, collection_entries,
    demo_activation_snapshot, initial_local_projection_state, node_availability,
    prepared_manifest_snapshot, BridgeAdmissionError, BridgeApplicationError, BridgeAvailability,
    BridgeManifestError, BridgePatchEnvelope, BridgePatchOperation, BridgeQuad, BridgeTargetRole,
    BridgeValue,
};

fn envelope(patch_id: u64, seq: u64, prev_rev: u64, rev: u64) -> BridgePatchEnvelope {
    BridgePatchEnvelope {
        patch_id,
        stream_id: 1,
        document_id: 1,
        previous_revision: prev_rev,
        revision: rev,
        epoch: 1,
        sequence: seq,
    }
}

#[test]
fn test_admit_valid_batch_succeeds() {
    let ops = vec![BridgePatchOperation::SetNodeAvailability {
        node: 1,
        availability: BridgeAvailability::Available,
    }];
    let batch = admit_projection_patch_batch(envelope(1, 1, 0, 1), ops);
    assert!(batch.is_ok());
}

#[test]
fn test_admit_rejects_zero_identity() {
    let ops = vec![BridgePatchOperation::SetNodeAvailability {
        node: 1,
        availability: BridgeAvailability::Available,
    }];
    let batch = admit_projection_patch_batch(envelope(0, 1, 0, 1), ops);
    assert_eq!(batch.unwrap_err(), BridgeAdmissionError::InvalidIdentity);
}

#[test]
fn test_admit_rejects_zero_node_id() {
    let ops = vec![BridgePatchOperation::SetNodeAvailability {
        node: 0,
        availability: BridgeAvailability::Available,
    }];
    let batch = admit_projection_patch_batch(envelope(1, 1, 0, 1), ops);
    assert_eq!(batch.unwrap_err(), BridgeAdmissionError::InvalidIdentity);
}

#[test]
fn test_admit_rejects_empty_operations() {
    let batch = admit_projection_patch_batch(envelope(1, 1, 0, 1), vec![]);
    assert_eq!(batch.unwrap_err(), BridgeAdmissionError::InvalidPatch);
}

#[test]
fn test_admit_rejects_duplicate_mutation_target() {
    let ops = vec![
        BridgePatchOperation::SetNodeAvailability {
            node: 1,
            availability: BridgeAvailability::Available,
        },
        BridgePatchOperation::SetNodeAvailability {
            node: 1,
            availability: BridgeAvailability::Hidden,
        },
    ];
    let batch = admit_projection_patch_batch(envelope(1, 1, 0, 1), ops);
    assert_eq!(batch.unwrap_err(), BridgeAdmissionError::InvalidPatch);
}

#[test]
fn test_manifest_snapshot_reflects_operations() {
    let ops = vec![
        BridgePatchOperation::SetNodeAvailability {
            node: 1,
            availability: BridgeAvailability::Available,
        },
        BridgePatchOperation::SetBindingValue {
            node: 2,
            slot: 0,
            value: BridgeValue::Text("hi".into()),
        },
        BridgePatchOperation::CollectionInsert {
            collection: 4,
            key: 1,
            before: None,
            value: BridgeValue::Unsigned(1),
        },
    ];
    let batch = admit_projection_patch_batch(envelope(1, 1, 0, 1), ops).expect("valid batch");
    let snapshot = prepared_manifest_snapshot(&batch).expect("derives");

    assert_eq!(snapshot.target_reference_count, 3);
    assert_eq!(snapshot.entries.len(), 3);
    assert_eq!(snapshot.entries[0].role, BridgeTargetRole::Node { node: 1 });
    assert_eq!(
        snapshot.entries[1].role,
        BridgeTargetRole::Binding { node: 2, slot: 0 }
    );
    assert_eq!(
        snapshot.entries[2].role,
        BridgeTargetRole::Collection { collection: 4 }
    );
    assert_eq!(snapshot.entries[0].patch_ordinal, 0);
    assert_eq!(snapshot.entries[0].operation_ordinal, 0);
    assert_eq!(snapshot.entries[2].operation_ordinal, 2);
}

#[test]
fn test_manifest_snapshot_is_deterministic() {
    let ops = vec![BridgePatchOperation::SetNodeAvailability {
        node: 1,
        availability: BridgeAvailability::Available,
    }];
    let batch = admit_projection_patch_batch(envelope(1, 1, 0, 1), ops).expect("valid batch");
    let snapshot_1 = prepared_manifest_snapshot(&batch).expect("derives");
    let snapshot_2 = prepared_manifest_snapshot(&batch).expect("derives");
    assert_eq!(snapshot_1, snapshot_2);
}

#[test]
fn test_demo_activation_snapshot_has_expected_shape() {
    let snapshot = demo_activation_snapshot();

    // root(1) + label(2) + status(3) + list(4) = 4 node anchors.
    assert_eq!(snapshot.node_anchor_ids, vec![1, 2, 3, 4]);
    // one declared binding: label node (2), slot 0.
    assert_eq!(snapshot.binding_anchor_ids, vec![(2, 0)]);
    // one explicitly declared collection anchor: list node (4).
    assert_eq!(snapshot.collection_anchor_ids, vec![4]);
}

#[test]
fn test_demo_activation_snapshot_is_deterministic() {
    assert_eq!(demo_activation_snapshot(), demo_activation_snapshot());
}

#[test]
fn test_apply_and_query_round_trip() {
    let previous = initial_local_projection_state();
    assert_eq!(node_availability(&previous, 1), None);

    let ops = vec![
        BridgePatchOperation::SetNodeAvailability {
            node: 1,
            availability: BridgeAvailability::Unavailable,
        },
        BridgePatchOperation::SetBindingValue {
            node: 2,
            slot: 0,
            value: BridgeValue::Text("hello".into()),
        },
        BridgePatchOperation::CollectionInsert {
            collection: 4,
            key: 10,
            before: None,
            value: BridgeValue::Quad(BridgeQuad::T),
        },
    ];
    let batch = admit_projection_patch_batch(envelope(1, 1, 0, 1), ops).expect("valid batch");
    let committed = apply_admitted_patch_batch(&previous, &batch).expect("applies");

    assert_eq!(
        node_availability(&committed, 1),
        Some(BridgeAvailability::Unavailable)
    );
    assert_eq!(
        binding_value(&committed, 2, 0),
        Some(BridgeValue::Text("hello".into()))
    );
    assert_eq!(
        collection_entries(&committed, 4),
        vec![(10, BridgeValue::Quad(BridgeQuad::T))]
    );

    // `previous` remains untouched.
    assert_eq!(node_availability(&previous, 1), None);
}

#[test]
fn test_apply_rejects_and_preserves_previous_on_missing_collection_key() {
    let previous = initial_local_projection_state();
    let ops = vec![BridgePatchOperation::CollectionUpdate {
        collection: 4,
        key: 999,
        value: BridgeValue::Unsigned(1),
    }];
    let batch = admit_projection_patch_batch(envelope(1, 1, 0, 1), ops).expect("valid batch");

    let err = apply_admitted_patch_batch(&previous, &batch).expect_err("missing key rejected");
    assert_eq!(
        err,
        BridgeApplicationError::KeyMissing {
            collection: 4,
            key: 999,
        }
    );
    assert_eq!(collection_entries(&previous, 4), Vec::new());
}

#[test]
fn test_binding_value_and_availability_absent_for_untouched_targets() {
    let state = initial_local_projection_state();
    assert_eq!(binding_value(&state, 1, 0), None);
    assert_eq!(node_availability(&state, 1), None);
    assert!(collection_entries(&state, 1).is_empty());
}

#[test]
fn test_manifest_error_type_is_reachable() {
    // Exercised indirectly: the error variant exists for the (practically
    // unreachable) replay-index-overflow case and is part of the bridge's
    // public contract.
    let _ = BridgeManifestError::ReplayIndexOverflow;
}
