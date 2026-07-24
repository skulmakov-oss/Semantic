//! UI-DNA2 Shell Player bridge qualification tests.
//!
//! These tests qualify the public `shell_bridge` surface: patch admission,
//! prepared-submission derivation, activation-target snapshot derivation,
//! and local-state application/query. They exercise the same surface
//! `prom-ui-runtime` consumes across the crate boundary.

use alloc::vec;
use alloc::vec::Vec;

use crate::shell_bridge::{
    admit_projection_patch_batch, apply_prepared_patch_submission, binding_value,
    collection_entries, demo_activation_snapshot, initial_local_projection_state,
    node_availability, prepare_patch_submission, prepared_submission_entries,
    prepared_submission_patch_count, prepared_submission_target_reference_count,
    BridgeAdmissionError, BridgeApplicationError, BridgeAvailability, BridgeManifestError,
    BridgePatchEnvelope, BridgePatchOperation, BridgeQuad, BridgeTargetRole, BridgeValue,
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
fn test_submission_reflects_operations() {
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
    let submission = prepare_patch_submission(batch).expect("derives");

    assert_eq!(prepared_submission_target_reference_count(&submission), 3);
    assert_eq!(prepared_submission_patch_count(&submission), 1);
    let entries = prepared_submission_entries(&submission);
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].role, BridgeTargetRole::Node { node: 1 });
    assert_eq!(
        entries[1].role,
        BridgeTargetRole::Binding { node: 2, slot: 0 }
    );
    assert_eq!(
        entries[2].role,
        BridgeTargetRole::Collection { collection: 4 }
    );
    assert_eq!(entries[0].patch_ordinal, 0);
    assert_eq!(entries[0].operation_ordinal, 0);
    assert_eq!(entries[2].operation_ordinal, 2);
}

#[test]
fn test_submission_is_deterministic() {
    let ops = vec![BridgePatchOperation::SetNodeAvailability {
        node: 1,
        availability: BridgeAvailability::Available,
    }];
    let batch_1 =
        admit_projection_patch_batch(envelope(1, 1, 0, 1), ops.clone()).expect("valid batch");
    let batch_2 = admit_projection_patch_batch(envelope(1, 1, 0, 1), ops).expect("valid batch");
    let submission_1 = prepare_patch_submission(batch_1).expect("derives");
    let submission_2 = prepare_patch_submission(batch_2).expect("derives");
    assert_eq!(
        prepared_submission_entries(&submission_1),
        prepared_submission_entries(&submission_2)
    );
    assert_eq!(
        prepared_submission_target_reference_count(&submission_1),
        prepared_submission_target_reference_count(&submission_2)
    );
}

#[test]
fn test_submission_patch_count_matches_admitted_batch() {
    let ops = vec![BridgePatchOperation::SetNodeAvailability {
        node: 1,
        availability: BridgeAvailability::Available,
    }];
    let batch = admit_projection_patch_batch(envelope(1, 1, 0, 1), ops).expect("valid batch");
    let submission = prepare_patch_submission(batch).expect("derives");
    // Exactly one `ProjectionPatch` is admitted per `admit_projection_patch_batch` call today.
    assert_eq!(prepared_submission_patch_count(&submission), 1);
}

#[test]
fn test_demo_activation_snapshot_has_expected_shape() {
    let snapshot = demo_activation_snapshot();

    // root(1) + label(2) + status(3) + list(4) = 4 node anchors.
    assert_eq!(snapshot.node_anchor_ids(), &[1, 2, 3, 4]);
    // one declared binding: label node (2), slot 0.
    assert_eq!(snapshot.binding_anchor_ids(), &[(2, 0)]);
    // one explicitly declared collection anchor: list node (4).
    assert_eq!(snapshot.collection_anchor_ids(), &[4]);
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
    let submission = prepare_patch_submission(batch).expect("derives");
    let committed = apply_prepared_patch_submission(&previous, &submission).expect("applies");

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
    let submission = prepare_patch_submission(batch).expect("derives");

    let err =
        apply_prepared_patch_submission(&previous, &submission).expect_err("missing key rejected");
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

/// P1 regression: a submission always applies exactly the operations its
/// own manifest was derived from. There is no `apply_*` entry point that
/// takes a manifest/submission and a *separate* batch, so two different
/// submissions can never have their evidence and operations cross-applied.
#[test]
fn test_two_submissions_remain_independent() {
    let ops_a = vec![BridgePatchOperation::SetNodeAvailability {
        node: 1,
        availability: BridgeAvailability::Available,
    }];
    let ops_b = vec![BridgePatchOperation::SetNodeAvailability {
        node: 2,
        availability: BridgeAvailability::Hidden,
    }];
    let batch_a = admit_projection_patch_batch(envelope(1, 1, 0, 1), ops_a).expect("valid");
    let batch_b = admit_projection_patch_batch(envelope(2, 1, 0, 1), ops_b).expect("valid");
    let submission_a = prepare_patch_submission(batch_a).expect("derives");
    let submission_b = prepare_patch_submission(batch_b).expect("derives");

    assert_eq!(
        prepared_submission_entries(&submission_a)[0].role,
        BridgeTargetRole::Node { node: 1 }
    );
    assert_eq!(
        prepared_submission_entries(&submission_b)[0].role,
        BridgeTargetRole::Node { node: 2 }
    );

    let previous = initial_local_projection_state();
    let after_a = apply_prepared_patch_submission(&previous, &submission_a).expect("applies");
    assert_eq!(
        node_availability(&after_a, 1),
        Some(BridgeAvailability::Available)
    );
    assert_eq!(node_availability(&after_a, 2), None);

    let after_b = apply_prepared_patch_submission(&previous, &submission_b).expect("applies");
    assert_eq!(node_availability(&after_b, 1), None);
    assert_eq!(
        node_availability(&after_b, 2),
        Some(BridgeAvailability::Hidden)
    );
}
