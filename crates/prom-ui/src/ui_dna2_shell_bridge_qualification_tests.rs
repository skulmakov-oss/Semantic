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

// ---- Gate D bounded ProjectionBundle activation (Issue #1543) ------------

use crate::action_ir::ActionIrDocument;
use crate::binding_graph::BindingGraphDocument;
use crate::contract_primitives::{SourceId as ContractSourceId, StaticDocumentId};
use crate::projection_bundle::{BundleProvenance, ProjectionBundleV0};
use crate::projection_source_frontend::compile_projection_source_text_with_collection_anchors;
use crate::role_dictionary::RoleDictionary;
use crate::shell_bridge::{activate_projection_bundle_v0_gate_d, GateDActivationError};

fn gate_d_test_source() -> alloc::string::String {
    alloc::string::String::from(concat!(
        "projection v0 {\n",
        "revision 1; epoch 1;\n",
        "surface 1 root 10 key 1;\n",
        "node 10 role root key 10 { child 11 order 0; child 12 order 1; }\n",
        "node 11 role text key 11 {}\n",
        "node 12 role danger_action key 12 {}\n",
        "collection_anchor 12;\n",
        "}"
    ))
}

fn gate_d_test_bundle_bytes() -> Vec<u8> {
    let source_id = ContractSourceId::new(81).expect("non-zero test source id");
    let document_id = StaticDocumentId::new(81).expect("non-zero test document id");
    let dictionary = RoleDictionary::current();

    let compiled = compile_projection_source_text_with_collection_anchors(
        source_id,
        &gate_d_test_source(),
        document_id,
        dictionary,
    )
    .expect("valid source compiles and qualifies");

    let binding_graph = BindingGraphDocument::build(Vec::new(), compiled.static_document())
        .expect("empty binding graph is trivially valid");
    let action_ir = ActionIrDocument::build(Vec::new(), compiled.static_document())
        .expect("empty action IR is trivially valid");

    let bundle = ProjectionBundleV0::assemble(
        document_id,
        crate::contract_primitives::Revision::new(1),
        crate::contract_primitives::Epoch::new(1),
        dictionary,
        compiled.static_document().clone(),
        binding_graph,
        action_ir,
        compiled.collection_anchors().clone(),
        BundleProvenance::new(
            alloc::string::String::from("ui_dna2_shell_bridge_qualification_tests"),
            source_id,
            crate::contract_primitives::Revision::new(1),
            crate::contract_primitives::Epoch::new(1),
        ),
    );
    bundle.canonical_bytes(dictionary).expect("canonicalizes")
}

#[test]
fn gate_d_activates_valid_bundle_and_derives_snapshot() {
    let activated = activate_projection_bundle_v0_gate_d(&gate_d_test_bundle_bytes())
        .expect("valid bundle activates");

    assert_eq!(activated.root_node(), 10);
    assert_eq!(activated.nodes().len(), 3);
    assert_eq!(activated.nodes()[0].id(), 10);
    assert_eq!(activated.nodes()[0].role(), "root");
    assert_eq!(activated.nodes()[0].children(), &[11, 12]);

    let snapshot = activated.activation_snapshot();
    assert_eq!(snapshot.node_anchor_ids(), &[10, 11, 12]);
    assert_eq!(snapshot.collection_anchor_ids(), &[12]);

    assert_eq!(activated.accessibility_entries().len(), 3);
}

#[test]
fn gate_d_rejects_malformed_bundle_before_any_activation_evidence() {
    let mut bytes = gate_d_test_bundle_bytes();
    bytes.push(0xFF); // trailing byte -> structural rejection
    let result = activate_projection_bundle_v0_gate_d(&bytes);
    assert_eq!(result, Err(GateDActivationError::BundleRejected));
}

#[test]
fn gate_d_rejects_truncated_bundle() {
    let bytes = gate_d_test_bundle_bytes();
    let truncated = &bytes[..bytes.len() / 2];
    let result = activate_projection_bundle_v0_gate_d(truncated);
    assert_eq!(result, Err(GateDActivationError::BundleRejected));
}

#[test]
fn gate_d_activation_is_deterministic() {
    let bytes = gate_d_test_bundle_bytes();
    let first = activate_projection_bundle_v0_gate_d(&bytes).expect("activates");
    let second = activate_projection_bundle_v0_gate_d(&bytes).expect("activates");
    assert_eq!(first, second);
}

#[test]
fn compile_projection_source_to_bundle_v0_round_trips_through_gate_d_activation() {
    let source = concat!(
        "projection v0 {\n",
        "revision 1; epoch 1;\n",
        "surface 1 root 10 key 1;\n",
        "node 10 role root key 10 { child 11 order 0; }\n",
        "node 11 role text key 11 {}\n",
        "collection_anchor 11;\n",
        "}"
    );
    let bytes = crate::shell_bridge::compile_projection_source_to_bundle_v0(
        91,
        source,
        91,
        1,
        1,
        "ui_dna2_shell_bridge_qualification_tests",
    )
    .expect("valid source compiles to bundle bytes");

    let activated =
        activate_projection_bundle_v0_gate_d(&bytes).expect("compiled bundle activates");
    assert_eq!(activated.root_node(), 10);
    assert_eq!(activated.nodes().len(), 2);
    assert_eq!(
        activated.activation_snapshot().collection_anchor_ids(),
        &[11]
    );
}

#[test]
fn compile_projection_source_to_bundle_v0_rejects_malformed_source() {
    let result = crate::shell_bridge::compile_projection_source_to_bundle_v0(
        91,
        "projection v0 { revision 1; epoch 1; ",
        91,
        1,
        1,
        "test",
    );
    assert!(result.is_err());
}
