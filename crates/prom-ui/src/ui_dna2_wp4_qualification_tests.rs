//! WP4 Contract Qualification Tests

use alloc::string::String;
use alloc::vec::Vec;

use crate::binding_graph::{BindingSlot, BindingTarget};
use crate::contract_primitives::{
    CollectionKey, DiagnosticCode, Epoch, Revision, StaticDocumentId, StaticNodeId, StaticSurfaceId,
};
use crate::projection_patch::{
    ProjectionNodeAvailability, ProjectionPatch, ProjectionPatchEnvelope, ProjectionPatchId,
    ProjectionPatchOperation, ProjectionPatchSequence, ProjectionPatchSet, ProjectionPatchStreamId,
    ProjectionPatchValue, ProjectionQuadState,
};

fn patch_id(raw: u64) -> ProjectionPatchId {
    ProjectionPatchId::new(raw).expect("patch id")
}

fn stream_id(raw: u64) -> ProjectionPatchStreamId {
    ProjectionPatchStreamId::new(raw).expect("stream id")
}

fn document_id(raw: u64) -> StaticDocumentId {
    StaticDocumentId::new(raw).expect("document id")
}

fn surface_id(raw: u64) -> StaticSurfaceId {
    StaticSurfaceId::new(raw).expect("surface id")
}

fn node_id(raw: u64) -> StaticNodeId {
    StaticNodeId::new(raw).expect("node id")
}

fn key(raw: u64) -> CollectionKey {
    CollectionKey::new(raw).expect("key")
}

#[allow(clippy::too_many_arguments)]
fn envelope(
    patch: u64,
    stream: u64,
    document: u64,
    surface: Option<u64>,
    previous_revision: u64,
    revision: u64,
    epoch: u64,
    sequence: u64,
) -> ProjectionPatchEnvelope {
    ProjectionPatchEnvelope::new(
        patch_id(patch),
        stream_id(stream),
        document_id(document),
        surface.map(surface_id),
        Revision::new(previous_revision),
        Revision::new(revision),
        Epoch::new(epoch),
        ProjectionPatchSequence::new(sequence),
    )
}

fn binding_target(node: u64, slot: u32) -> BindingTarget {
    BindingTarget {
        node: node_id(node),
        slot: BindingSlot(slot),
    }
}

fn binding_update_patch(patch: u64, sequence: u64, revision: u64) -> ProjectionPatch {
    ProjectionPatch::new(
        envelope(
            patch,
            1,
            1,
            Some(1),
            revision.saturating_sub(1),
            revision,
            1,
            sequence,
        ),
        alloc::vec![ProjectionPatchOperation::SetBindingValue {
            target: binding_target(10, 1),
            value: ProjectionPatchValue::Quad(ProjectionQuadState::T),
        }],
    )
    .expect("valid binding update patch")
}

fn quad_patch(state: ProjectionQuadState) -> ProjectionPatch {
    ProjectionPatch::new(
        envelope(1, 1, 1, Some(1), 0, 1, 1, 5),
        alloc::vec![ProjectionPatchOperation::SetBindingValue {
            target: binding_target(10, 1),
            value: ProjectionPatchValue::Quad(state),
        }],
    )
    .expect("valid quad patch")
}

fn patch_with_operation(operation: ProjectionPatchOperation) -> ProjectionPatch {
    ProjectionPatch::new(
        envelope(1, 1, 1, Some(1), 0, 1, 1, 0),
        alloc::vec![operation],
    )
    .expect("valid patch")
}

fn patch_with_operations(operations: Vec<ProjectionPatchOperation>) -> ProjectionPatch {
    ProjectionPatch::new(envelope(1, 1, 1, Some(1), 0, 1, 1, 0), operations).expect("valid patch")
}

fn diagnostic_codes(
    diagnostics: &[crate::projection_patch::ProjectionPatchDiagnostic],
) -> Vec<&'static str> {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.coordinate().code().as_str())
        .collect()
}

#[test]
fn ui_dna2_wp4_reject_zero_patch_id() {
    assert!(ProjectionPatchId::new(0).is_err());
}

#[test]
fn ui_dna2_wp4_reject_zero_stream_id() {
    assert!(ProjectionPatchStreamId::new(0).is_err());
}

#[test]
fn ui_dna2_wp4_preserve_exact_envelope_values() {
    let envelope = envelope(7, 8, 9, Some(10), 11, 12, 13, 14);

    assert_eq!(envelope.patch_id().raw(), 7);
    assert_eq!(envelope.stream_id().raw(), 8);
    assert_eq!(envelope.document_id().raw(), 9);
    assert_eq!(envelope.surface_id().expect("surface").raw(), 10);
    assert_eq!(envelope.previous_projection_revision().raw(), 11);
    assert_eq!(envelope.projection_revision().raw(), 12);
    assert_eq!(envelope.epoch().raw(), 13);
    assert_eq!(envelope.sequence().raw(), 14);
}

#[test]
fn ui_dna2_wp4_patch_and_stream_identity_are_nominally_distinct() {
    fn takes_patch_id(value: ProjectionPatchId) -> u64 {
        value.raw()
    }

    fn takes_stream_id(value: ProjectionPatchStreamId) -> u64 {
        value.raw()
    }

    assert_eq!(takes_patch_id(patch_id(1)), 1);
    assert_eq!(takes_stream_id(stream_id(2)), 2);
}

#[test]
fn ui_dna2_wp4_quad_states_remain_four_distinct_values() {
    use core::mem::discriminant;

    assert_ne!(
        discriminant(&ProjectionQuadState::N),
        discriminant(&ProjectionQuadState::F)
    );
    assert_ne!(
        discriminant(&ProjectionQuadState::F),
        discriminant(&ProjectionQuadState::T)
    );
    assert_ne!(
        discriminant(&ProjectionQuadState::T),
        discriminant(&ProjectionQuadState::S)
    );
    assert_ne!(
        discriminant(&ProjectionQuadState::N),
        discriminant(&ProjectionQuadState::S)
    );
}

#[test]
fn ui_dna2_wp4_canonical_encoding_distinguishes_all_quad_states() {
    let n = ProjectionPatchSet::new(alloc::vec![quad_patch(ProjectionQuadState::N)])
        .expect("n set")
        .canonical_bytes()
        .expect("n bytes");
    let f = ProjectionPatchSet::new(alloc::vec![quad_patch(ProjectionQuadState::F)])
        .expect("f set")
        .canonical_bytes()
        .expect("f bytes");
    let t = ProjectionPatchSet::new(alloc::vec![quad_patch(ProjectionQuadState::T)])
        .expect("t set")
        .canonical_bytes()
        .expect("t bytes");
    let s = ProjectionPatchSet::new(alloc::vec![quad_patch(ProjectionQuadState::S)])
        .expect("s set")
        .canonical_bytes()
        .expect("s bytes");

    assert_ne!(n, f);
    assert_ne!(n, t);
    assert_ne!(n, s);
    assert_ne!(f, t);
    assert_ne!(f, s);
    assert_ne!(t, s);
}

#[test]
fn ui_dna2_wp4_unknown_is_not_represented_as_false() {
    assert_ne!(
        ProjectionPatchValue::Quad(ProjectionQuadState::N),
        ProjectionPatchValue::Quad(ProjectionQuadState::F)
    );
}

#[test]
fn ui_dna2_wp4_conflict_is_not_represented_as_error() {
    assert_ne!(
        ProjectionPatchValue::Quad(ProjectionQuadState::S),
        ProjectionPatchValue::Quad(ProjectionQuadState::T)
    );
}

#[test]
fn ui_dna2_wp4_accept_one_valid_patch() {
    let patch = binding_update_patch(1, 0, 1);
    let set = ProjectionPatchSet::new(alloc::vec![patch]).expect("valid set");
    assert_eq!(set.patches().len(), 1);
}

#[test]
fn ui_dna2_wp4_accept_valid_multi_patch_chain() {
    let first = binding_update_patch(1, 0, 1);
    let second = ProjectionPatch::new(
        envelope(2, 1, 1, Some(1), 1, 2, 1, 1),
        alloc::vec![ProjectionPatchOperation::SetNodeAvailability {
            node: node_id(10),
            availability: ProjectionNodeAvailability::Available,
        }],
    )
    .expect("second patch");

    let set = ProjectionPatchSet::new(alloc::vec![first, second]).expect("chain");
    assert_eq!(set.patches().len(), 2);
}

#[test]
fn ui_dna2_wp4_accept_non_zero_initial_previous_revision() {
    let patch = ProjectionPatch::new(
        envelope(1, 1, 1, Some(1), 10, 11, 1, 7),
        alloc::vec![ProjectionPatchOperation::SetBindingValue {
            target: binding_target(10, 1),
            value: ProjectionPatchValue::SignedScalar(41),
        }],
    )
    .expect("valid patch");

    assert_eq!(patch.envelope().previous_projection_revision().raw(), 10);
}

#[test]
fn ui_dna2_wp4_preserve_operation_order() {
    let patch = ProjectionPatch::new(
        envelope(1, 1, 1, Some(1), 0, 1, 1, 0),
        alloc::vec![
            ProjectionPatchOperation::SetNodeAvailability {
                node: node_id(10),
                availability: ProjectionNodeAvailability::Pending,
            },
            ProjectionPatchOperation::SetBindingValue {
                target: binding_target(10, 1),
                value: ProjectionPatchValue::SignedScalar(9),
            },
        ],
    )
    .expect("patch");

    assert!(matches!(
        &patch.operations()[0],
        ProjectionPatchOperation::SetNodeAvailability { .. }
    ));
    assert!(matches!(
        &patch.operations()[1],
        ProjectionPatchOperation::SetBindingValue { .. }
    ));
}

#[test]
fn ui_dna2_wp4_preserve_patch_order() {
    let first = binding_update_patch(1, 0, 1);
    let second = ProjectionPatch::new(
        envelope(2, 1, 1, Some(1), 1, 2, 1, 1),
        alloc::vec![ProjectionPatchOperation::SetBindingValue {
            target: binding_target(10, 2),
            value: ProjectionPatchValue::SignedScalar(2),
        }],
    )
    .expect("patch");
    let set = ProjectionPatchSet::new(alloc::vec![first, second]).expect("set");

    assert_eq!(set.patches()[0].envelope().patch_id().raw(), 1);
    assert_eq!(set.patches()[1].envelope().patch_id().raw(), 2);
}

#[test]
fn ui_dna2_wp4_reject_non_increasing_revision() {
    let error = ProjectionPatch::new(
        envelope(1, 1, 1, Some(1), 5, 5, 1, 0),
        alloc::vec![ProjectionPatchOperation::SetBindingValue {
            target: binding_target(10, 1),
            value: ProjectionPatchValue::SignedScalar(1),
        }],
    )
    .unwrap_err();

    assert_eq!(
        diagnostic_codes(&error),
        alloc::vec!["PP_NON_INCREASING_REVISION"]
    );
}

#[test]
fn ui_dna2_wp4_reject_broken_adjacent_revision_chain() {
    let first = binding_update_patch(1, 0, 2);
    let second = ProjectionPatch::new(
        envelope(2, 1, 1, Some(1), 99, 100, 1, 1),
        alloc::vec![ProjectionPatchOperation::SetBindingValue {
            target: binding_target(10, 2),
            value: ProjectionPatchValue::SignedScalar(2),
        }],
    )
    .expect("patch");

    let error = ProjectionPatchSet::new(alloc::vec![first, second]).unwrap_err();
    assert_eq!(
        diagnostic_codes(&error),
        alloc::vec!["PP_BROKEN_REVISION_CHAIN"]
    );
}

#[test]
fn ui_dna2_wp4_reject_duplicate_sequence() {
    let first = binding_update_patch(1, 1, 1);
    let second = ProjectionPatch::new(
        envelope(2, 1, 1, Some(1), 1, 2, 1, 1),
        alloc::vec![ProjectionPatchOperation::SetBindingValue {
            target: binding_target(10, 2),
            value: ProjectionPatchValue::SignedScalar(3),
        }],
    )
    .expect("patch");
    let error = ProjectionPatchSet::new(alloc::vec![first, second]).unwrap_err();
    assert_eq!(diagnostic_codes(&error), alloc::vec!["PP_DUP_SEQUENCE"]);
}

#[test]
fn ui_dna2_wp4_reject_descending_sequence() {
    let first = binding_update_patch(1, 3, 1);
    let second = ProjectionPatch::new(
        envelope(2, 1, 1, Some(1), 1, 2, 1, 2),
        alloc::vec![ProjectionPatchOperation::SetBindingValue {
            target: binding_target(10, 2),
            value: ProjectionPatchValue::SignedScalar(3),
        }],
    )
    .expect("patch");
    let error = ProjectionPatchSet::new(alloc::vec![first, second]).unwrap_err();
    assert_eq!(
        diagnostic_codes(&error),
        alloc::vec!["PP_OUT_OF_ORDER_SEQUENCE"]
    );
}

#[test]
fn ui_dna2_wp4_reject_sequence_gaps() {
    let first = binding_update_patch(1, 0, 1);
    let second = ProjectionPatch::new(
        envelope(2, 1, 1, Some(1), 1, 2, 1, 2),
        alloc::vec![ProjectionPatchOperation::SetBindingValue {
            target: binding_target(10, 2),
            value: ProjectionPatchValue::SignedScalar(3),
        }],
    )
    .expect("patch");
    let error = ProjectionPatchSet::new(alloc::vec![first, second]).unwrap_err();
    assert_eq!(
        diagnostic_codes(&error),
        alloc::vec!["PP_OUT_OF_ORDER_SEQUENCE"]
    );
}

#[test]
fn ui_dna2_wp4_reject_sequence_overflow_where_applicable() {
    let first = binding_update_patch(1, u64::MAX, 1);
    let second = ProjectionPatch::new(
        envelope(2, 1, 1, Some(1), 1, 2, 1, 0),
        alloc::vec![ProjectionPatchOperation::SetBindingValue {
            target: binding_target(10, 2),
            value: ProjectionPatchValue::SignedScalar(3),
        }],
    )
    .expect("patch");
    let error = ProjectionPatchSet::new(alloc::vec![first, second]).unwrap_err();
    assert_eq!(
        diagnostic_codes(&error),
        alloc::vec!["PP_SEQUENCE_OVERFLOW"]
    );
}

#[test]
fn ui_dna2_wp4_reject_mixed_epochs() {
    let first = binding_update_patch(1, 0, 1);
    let second = ProjectionPatch::new(
        envelope(2, 1, 1, Some(1), 1, 2, 2, 1),
        alloc::vec![ProjectionPatchOperation::SetBindingValue {
            target: binding_target(10, 2),
            value: ProjectionPatchValue::SignedScalar(3),
        }],
    )
    .expect("patch");
    let error = ProjectionPatchSet::new(alloc::vec![first, second]).unwrap_err();
    assert_eq!(diagnostic_codes(&error), alloc::vec!["PP_MIXED_EPOCH"]);
}

#[test]
fn ui_dna2_wp4_reject_mixed_streams() {
    let first = binding_update_patch(1, 0, 1);
    let second = ProjectionPatch::new(
        envelope(2, 2, 1, Some(1), 1, 2, 1, 1),
        alloc::vec![ProjectionPatchOperation::SetBindingValue {
            target: binding_target(10, 2),
            value: ProjectionPatchValue::SignedScalar(3),
        }],
    )
    .expect("patch");
    let error = ProjectionPatchSet::new(alloc::vec![first, second]).unwrap_err();
    assert_eq!(diagnostic_codes(&error), alloc::vec!["PP_MIXED_STREAM"]);
}

#[test]
fn ui_dna2_wp4_reject_mixed_documents() {
    let first = binding_update_patch(1, 0, 1);
    let second = ProjectionPatch::new(
        envelope(2, 1, 2, Some(1), 1, 2, 1, 1),
        alloc::vec![ProjectionPatchOperation::SetBindingValue {
            target: binding_target(10, 2),
            value: ProjectionPatchValue::SignedScalar(3),
        }],
    )
    .expect("patch");
    let error = ProjectionPatchSet::new(alloc::vec![first, second]).unwrap_err();
    assert_eq!(diagnostic_codes(&error), alloc::vec!["PP_MIXED_DOCUMENT"]);
}

#[test]
fn ui_dna2_wp4_reject_duplicate_patch_id() {
    let first = binding_update_patch(1, 0, 1);
    let second = ProjectionPatch::new(
        envelope(1, 1, 1, Some(1), 1, 2, 1, 1),
        alloc::vec![ProjectionPatchOperation::SetBindingValue {
            target: binding_target(10, 2),
            value: ProjectionPatchValue::SignedScalar(3),
        }],
    )
    .expect("patch");
    let error = ProjectionPatchSet::new(alloc::vec![first, second]).unwrap_err();
    assert_eq!(diagnostic_codes(&error), alloc::vec!["PP_DUP_PATCH_ID"]);
}

#[test]
fn ui_dna2_wp4_report_duplicate_patch_id_deterministically() {
    let first = binding_update_patch(7, 0, 1);
    let second = ProjectionPatch::new(
        envelope(7, 1, 1, Some(1), 1, 2, 1, 1),
        alloc::vec![ProjectionPatchOperation::SetBindingValue {
            target: binding_target(10, 2),
            value: ProjectionPatchValue::SignedScalar(4),
        }],
    )
    .expect("patch");

    let error_a = ProjectionPatchSet::new(alloc::vec![first.clone(), second.clone()]).unwrap_err();
    let error_b = ProjectionPatchSet::new(alloc::vec![first, second]).unwrap_err();
    assert_eq!(error_a, error_b);
}

#[test]
fn ui_dna2_wp4_reject_empty_operation_list() {
    let error =
        ProjectionPatch::new(envelope(1, 1, 1, Some(1), 0, 1, 1, 0), Vec::new()).unwrap_err();
    assert_eq!(diagnostic_codes(&error), alloc::vec!["PP_EMPTY_OPERATIONS"]);
}

#[test]
fn ui_dna2_wp4_accept_binding_value_update() {
    let patch = patch_with_operation(ProjectionPatchOperation::SetBindingValue {
        target: binding_target(10, 1),
        value: ProjectionPatchValue::Text(String::from("hello")),
    });
    assert!(matches!(
        &patch.operations()[0],
        ProjectionPatchOperation::SetBindingValue { .. }
    ));
}

#[test]
fn ui_dna2_wp4_accept_node_availability_update() {
    let patch = patch_with_operation(ProjectionPatchOperation::SetNodeAvailability {
        node: node_id(10),
        availability: ProjectionNodeAvailability::Hidden,
    });
    assert!(matches!(
        &patch.operations()[0],
        ProjectionPatchOperation::SetNodeAvailability { .. }
    ));
}

#[test]
fn ui_dna2_wp4_accept_keyed_collection_insert() {
    let patch = patch_with_operation(ProjectionPatchOperation::CollectionInsert {
        collection: node_id(20),
        key: key(1),
        before: Some(key(2)),
        value: ProjectionPatchValue::SignedScalar(1),
    });
    assert!(matches!(
        &patch.operations()[0],
        ProjectionPatchOperation::CollectionInsert { .. }
    ));
}

#[test]
fn ui_dna2_wp4_accept_keyed_collection_update() {
    let patch = patch_with_operation(ProjectionPatchOperation::CollectionUpdate {
        collection: node_id(20),
        key: key(1),
        value: ProjectionPatchValue::SignedScalar(2),
    });
    assert!(matches!(
        &patch.operations()[0],
        ProjectionPatchOperation::CollectionUpdate { .. }
    ));
}

#[test]
fn ui_dna2_wp4_accept_keyed_collection_remove() {
    let patch = patch_with_operation(ProjectionPatchOperation::CollectionRemove {
        collection: node_id(20),
        key: key(1),
    });
    assert!(matches!(
        &patch.operations()[0],
        ProjectionPatchOperation::CollectionRemove { .. }
    ));
}

#[test]
fn ui_dna2_wp4_accept_keyed_collection_move() {
    let patch = patch_with_operation(ProjectionPatchOperation::CollectionMove {
        collection: node_id(20),
        key: key(1),
        before: Some(key(2)),
    });
    assert!(matches!(
        &patch.operations()[0],
        ProjectionPatchOperation::CollectionMove { .. }
    ));
}

#[test]
fn ui_dna2_wp4_reject_move_before_self() {
    let error = ProjectionPatch::new(
        envelope(1, 1, 1, Some(1), 0, 1, 1, 0),
        alloc::vec![ProjectionPatchOperation::CollectionMove {
            collection: node_id(20),
            key: key(1),
            before: Some(key(1)),
        }],
    )
    .unwrap_err();
    assert_eq!(
        diagnostic_codes(&error),
        alloc::vec!["PP_COLLECTION_BEFORE_SELF"]
    );
}

#[test]
fn ui_dna2_wp4_reject_insert_before_self() {
    let error = ProjectionPatch::new(
        envelope(1, 1, 1, Some(1), 0, 1, 1, 0),
        alloc::vec![ProjectionPatchOperation::CollectionInsert {
            collection: node_id(20),
            key: key(1),
            before: Some(key(1)),
            value: ProjectionPatchValue::SignedScalar(1),
        }],
    )
    .unwrap_err();
    assert_eq!(
        diagnostic_codes(&error),
        alloc::vec!["PP_COLLECTION_BEFORE_SELF"]
    );
}

#[test]
fn ui_dna2_wp4_reject_ambiguous_duplicate_mutation_target() {
    let error = ProjectionPatch::new(
        envelope(1, 1, 1, Some(1), 0, 1, 1, 0),
        alloc::vec![
            ProjectionPatchOperation::SetBindingValue {
                target: binding_target(10, 1),
                value: ProjectionPatchValue::SignedScalar(1),
            },
            ProjectionPatchOperation::SetBindingValue {
                target: binding_target(10, 1),
                value: ProjectionPatchValue::SignedScalar(2),
            },
        ],
    )
    .unwrap_err();
    assert_eq!(
        diagnostic_codes(&error),
        alloc::vec!["PP_DUP_MUTATION_TARGET"]
    );
}

#[test]
fn ui_dna2_wp4_accept_same_key_insert_then_update() {
    let patch = patch_with_operations(alloc::vec![
        ProjectionPatchOperation::CollectionInsert {
            collection: node_id(20),
            key: key(1),
            before: Some(key(2)),
            value: ProjectionPatchValue::SignedScalar(1),
        },
        ProjectionPatchOperation::CollectionUpdate {
            collection: node_id(20),
            key: key(1),
            value: ProjectionPatchValue::SignedScalar(2),
        },
    ]);

    assert!(matches!(
        &patch.operations()[0],
        ProjectionPatchOperation::CollectionInsert { .. }
    ));
    assert!(matches!(
        &patch.operations()[1],
        ProjectionPatchOperation::CollectionUpdate { .. }
    ));
}

#[test]
fn ui_dna2_wp4_accept_same_key_insert_then_move() {
    let patch = patch_with_operations(alloc::vec![
        ProjectionPatchOperation::CollectionInsert {
            collection: node_id(20),
            key: key(1),
            before: Some(key(2)),
            value: ProjectionPatchValue::SignedScalar(1),
        },
        ProjectionPatchOperation::CollectionMove {
            collection: node_id(20),
            key: key(1),
            before: Some(key(3)),
        },
    ]);

    assert_eq!(patch.operations().len(), 2);
}

#[test]
fn ui_dna2_wp4_accept_same_key_update_then_remove() {
    let patch = patch_with_operations(alloc::vec![
        ProjectionPatchOperation::CollectionUpdate {
            collection: node_id(20),
            key: key(1),
            value: ProjectionPatchValue::SignedScalar(2),
        },
        ProjectionPatchOperation::CollectionRemove {
            collection: node_id(20),
            key: key(1),
        },
    ]);

    assert_eq!(patch.operations().len(), 2);
}

#[test]
fn ui_dna2_wp4_reject_same_key_insert_then_insert() {
    let error = ProjectionPatch::new(
        envelope(1, 1, 1, Some(1), 0, 1, 1, 0),
        alloc::vec![
            ProjectionPatchOperation::CollectionInsert {
                collection: node_id(20),
                key: key(1),
                before: Some(key(2)),
                value: ProjectionPatchValue::SignedScalar(1),
            },
            ProjectionPatchOperation::CollectionInsert {
                collection: node_id(20),
                key: key(1),
                before: Some(key(3)),
                value: ProjectionPatchValue::SignedScalar(2),
            },
        ],
    )
    .unwrap_err();

    assert_eq!(
        diagnostic_codes(&error),
        alloc::vec!["PP_DUP_MUTATION_TARGET"]
    );
}

#[test]
fn ui_dna2_wp4_reject_same_key_update_then_update() {
    let error = ProjectionPatch::new(
        envelope(1, 1, 1, Some(1), 0, 1, 1, 0),
        alloc::vec![
            ProjectionPatchOperation::CollectionUpdate {
                collection: node_id(20),
                key: key(1),
                value: ProjectionPatchValue::SignedScalar(1),
            },
            ProjectionPatchOperation::CollectionUpdate {
                collection: node_id(20),
                key: key(1),
                value: ProjectionPatchValue::SignedScalar(2),
            },
        ],
    )
    .unwrap_err();

    assert_eq!(
        diagnostic_codes(&error),
        alloc::vec!["PP_DUP_MUTATION_TARGET"]
    );
}

#[test]
fn ui_dna2_wp4_reject_same_key_remove_then_remove() {
    let error = ProjectionPatch::new(
        envelope(1, 1, 1, Some(1), 0, 1, 1, 0),
        alloc::vec![
            ProjectionPatchOperation::CollectionRemove {
                collection: node_id(20),
                key: key(1),
            },
            ProjectionPatchOperation::CollectionRemove {
                collection: node_id(20),
                key: key(1),
            },
        ],
    )
    .unwrap_err();

    assert_eq!(
        diagnostic_codes(&error),
        alloc::vec!["PP_DUP_MUTATION_TARGET"]
    );
}

#[test]
fn ui_dna2_wp4_reject_same_key_move_then_move() {
    let error = ProjectionPatch::new(
        envelope(1, 1, 1, Some(1), 0, 1, 1, 0),
        alloc::vec![
            ProjectionPatchOperation::CollectionMove {
                collection: node_id(20),
                key: key(1),
                before: Some(key(2)),
            },
            ProjectionPatchOperation::CollectionMove {
                collection: node_id(20),
                key: key(1),
                before: Some(key(3)),
            },
        ],
    )
    .unwrap_err();

    assert_eq!(
        diagnostic_codes(&error),
        alloc::vec!["PP_DUP_MUTATION_TARGET"]
    );
}

#[test]
fn ui_dna2_wp4_reject_duplicate_node_availability_target() {
    let error = ProjectionPatch::new(
        envelope(1, 1, 1, Some(1), 0, 1, 1, 0),
        alloc::vec![
            ProjectionPatchOperation::SetNodeAvailability {
                node: node_id(10),
                availability: ProjectionNodeAvailability::Pending,
            },
            ProjectionPatchOperation::SetNodeAvailability {
                node: node_id(10),
                availability: ProjectionNodeAvailability::Available,
            },
        ],
    )
    .unwrap_err();

    assert_eq!(
        diagnostic_codes(&error),
        alloc::vec!["PP_DUP_MUTATION_TARGET"]
    );
}

#[test]
fn ui_dna2_wp4_accept_same_kind_collection_ops_for_distinct_keys() {
    let patch = patch_with_operations(alloc::vec![
        ProjectionPatchOperation::CollectionUpdate {
            collection: node_id(20),
            key: key(1),
            value: ProjectionPatchValue::SignedScalar(1),
        },
        ProjectionPatchOperation::CollectionUpdate {
            collection: node_id(20),
            key: key(2),
            value: ProjectionPatchValue::SignedScalar(2),
        },
    ]);

    assert_eq!(patch.operations().len(), 2);
}

#[test]
fn ui_dna2_wp4_same_key_cross_kind_order_remains_observable_in_bytes() {
    let insert_then_update = patch_with_operations(alloc::vec![
        ProjectionPatchOperation::CollectionInsert {
            collection: node_id(20),
            key: key(1),
            before: Some(key(2)),
            value: ProjectionPatchValue::SignedScalar(1),
        },
        ProjectionPatchOperation::CollectionUpdate {
            collection: node_id(20),
            key: key(1),
            value: ProjectionPatchValue::SignedScalar(2),
        },
    ]);
    let update_then_insert = patch_with_operations(alloc::vec![
        ProjectionPatchOperation::CollectionUpdate {
            collection: node_id(20),
            key: key(1),
            value: ProjectionPatchValue::SignedScalar(2),
        },
        ProjectionPatchOperation::CollectionInsert {
            collection: node_id(20),
            key: key(1),
            before: Some(key(2)),
            value: ProjectionPatchValue::SignedScalar(1),
        },
    ]);

    let bytes_a = ProjectionPatchSet::new(alloc::vec![insert_then_update])
        .expect("set a")
        .canonical_bytes()
        .expect("bytes a");
    let bytes_b = ProjectionPatchSet::new(alloc::vec![update_then_insert])
        .expect("set b")
        .canonical_bytes()
        .expect("bytes b");

    assert_ne!(bytes_a, bytes_b);
}

#[test]
fn ui_dna2_wp4_same_kind_duplicate_diagnostics_are_deterministic() {
    let invalid = || {
        ProjectionPatch::new(
            envelope(1, 1, 1, Some(1), 0, 1, 1, 0),
            alloc::vec![
                ProjectionPatchOperation::CollectionRemove {
                    collection: node_id(20),
                    key: key(1),
                },
                ProjectionPatchOperation::CollectionRemove {
                    collection: node_id(20),
                    key: key(1),
                },
            ],
        )
        .unwrap_err()
    };

    assert_eq!(invalid(), invalid());
}

#[test]
fn ui_dna2_wp4_same_valid_input_produces_identical_canonical_bytes() {
    let first = ProjectionPatchSet::new(alloc::vec![binding_update_patch(1, 0, 1)]).expect("set");
    let second = ProjectionPatchSet::new(alloc::vec![binding_update_patch(1, 0, 1)]).expect("set");

    assert_eq!(
        first.canonical_bytes().expect("bytes"),
        second.canonical_bytes().expect("bytes")
    );
}

#[test]
fn ui_dna2_wp4_same_invalid_input_produces_identical_ordered_diagnostics() {
    let invalid =
        || ProjectionPatch::new(envelope(1, 1, 1, Some(1), 0, 0, 1, 0), alloc::vec![]).unwrap_err();

    assert_eq!(invalid(), invalid());
}

#[test]
fn ui_dna2_wp4_different_operation_order_produces_different_canonical_bytes() {
    let patch_a = ProjectionPatch::new(
        envelope(1, 1, 1, Some(1), 0, 1, 1, 0),
        alloc::vec![
            ProjectionPatchOperation::SetNodeAvailability {
                node: node_id(10),
                availability: ProjectionNodeAvailability::Available,
            },
            ProjectionPatchOperation::SetBindingValue {
                target: binding_target(10, 1),
                value: ProjectionPatchValue::SignedScalar(2),
            },
        ],
    )
    .expect("patch a");
    let patch_b = ProjectionPatch::new(
        envelope(1, 1, 1, Some(1), 0, 1, 1, 0),
        alloc::vec![
            ProjectionPatchOperation::SetBindingValue {
                target: binding_target(10, 1),
                value: ProjectionPatchValue::SignedScalar(2),
            },
            ProjectionPatchOperation::SetNodeAvailability {
                node: node_id(10),
                availability: ProjectionNodeAvailability::Available,
            },
        ],
    )
    .expect("patch b");

    let bytes_a = ProjectionPatchSet::new(alloc::vec![patch_a])
        .expect("set a")
        .canonical_bytes()
        .expect("bytes a");
    let bytes_b = ProjectionPatchSet::new(alloc::vec![patch_b])
        .expect("set b")
        .canonical_bytes()
        .expect("bytes b");

    assert_ne!(bytes_a, bytes_b);
}

#[test]
fn ui_dna2_wp4_out_of_order_patches_are_rejected_rather_than_silently_sorted() {
    let later = binding_update_patch(2, 1, 2);
    let earlier = binding_update_patch(1, 0, 1);

    let error = ProjectionPatchSet::new(alloc::vec![later, earlier]).unwrap_err();
    assert_eq!(
        diagnostic_codes(&error),
        alloc::vec!["PP_OUT_OF_ORDER_SEQUENCE"]
    );
}

#[test]
fn ui_dna2_wp4_diagnostics_are_sorted_and_deduplicated() {
    let error = ProjectionPatch::new(
        envelope(1, 1, 1, Some(1), 2, 2, 1, 0),
        alloc::vec![
            ProjectionPatchOperation::CollectionMove {
                collection: node_id(20),
                key: key(1),
                before: Some(key(1)),
            },
            ProjectionPatchOperation::CollectionMove {
                collection: node_id(20),
                key: key(1),
                before: Some(key(1)),
            },
        ],
    )
    .unwrap_err();

    let codes = diagnostic_codes(&error);
    assert_eq!(
        codes,
        alloc::vec![
            "PP_COLLECTION_BEFORE_SELF",
            "PP_DUP_MUTATION_TARGET",
            "PP_NON_INCREASING_REVISION"
        ]
    );
    assert_eq!(
        codes[0],
        DiagnosticCode::new("PP_COLLECTION_BEFORE_SELF").as_str()
    );
}
