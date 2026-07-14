use alloc::string::String;
use alloc::vec::Vec;

use super::contract_primitives::{
    ChildOrder, CollectionKey, Epoch, Revision, SourceId, SourceRef, SourceSpan, StaticDocumentId,
    StaticNodeId, StaticSurfaceId,
};
use super::role_dictionary::{RoleDictionary, RoleId};
use super::static_ir::{
    StaticUiChild, StaticUiDocument, StaticUiIrDiagnosticKind, StaticUiNode, StaticUiSurface,
};
use super::static_ir_artifact::{
    qualify_checked_cursor_overflow, verify_static_ui_ir_artifact_v1, StaticUiArtifactV1Error,
    StaticUiArtifactV1ErrorKind, StaticUiArtifactV1Field, StaticUiArtifactV1Limits,
    StaticUiArtifactV1Stage, STATIC_UI_IR_ARTIFACT_V1_MAGIC,
};

const MINIMAL: &[u8] = include_bytes!("../tests/vectors/static_ui_ir_v1/minimal.bin");
const STRUCTURED: &[u8] = include_bytes!("../tests/vectors/static_ui_ir_v1/structured.bin");

#[derive(Clone)]
struct SurfaceSpec {
    id: u64,
    root: u64,
    key: u64,
    source: Option<(u64, u32, u32)>,
}

#[derive(Clone)]
struct NodeSpec<'a> {
    id: u64,
    role: &'a str,
    key: u64,
    source: Option<(u64, u32, u32)>,
    children: Vec<(u32, u64)>,
    accessibility_ref: Option<u64>,
}

fn limits() -> StaticUiArtifactV1Limits {
    StaticUiArtifactV1Limits {
        max_input_bytes: 16 * 1024,
        max_surfaces: 16,
        max_nodes: 64,
        max_children_per_node: 32,
        max_role_bytes: 64,
    }
}

fn minimal_specs<'a>() -> (Vec<SurfaceSpec>, Vec<NodeSpec<'a>>) {
    (
        vec![SurfaceSpec {
            id: 1,
            root: 10,
            key: 1,
            source: None,
        }],
        vec![NodeSpec {
            id: 10,
            role: "root",
            key: 10,
            source: None,
            children: Vec::new(),
            accessibility_ref: None,
        }],
    )
}

fn encode(
    document_id: u64,
    revision: u64,
    epoch: u64,
    surfaces: &[SurfaceSpec],
    nodes: &[NodeSpec<'_>],
) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_bytes(&mut bytes, STATIC_UI_IR_ARTIFACT_V1_MAGIC);
    push_u32(&mut bytes, 1);
    push_u32(&mut bytes, 1);
    push_u64(&mut bytes, document_id);
    push_u64(&mut bytes, revision);
    push_u64(&mut bytes, epoch);
    push_u32(&mut bytes, surfaces.len() as u32);
    for surface in surfaces {
        push_u64(&mut bytes, surface.id);
        push_u64(&mut bytes, surface.root);
        push_u64(&mut bytes, surface.key);
        push_source(&mut bytes, surface.source);
    }
    push_u32(&mut bytes, nodes.len() as u32);
    for node in nodes {
        push_u64(&mut bytes, node.id);
        push_bytes(&mut bytes, node.role.as_bytes());
        push_u64(&mut bytes, node.key);
        push_source(&mut bytes, node.source);
        push_u32(&mut bytes, node.children.len() as u32);
        for (order, child) in &node.children {
            push_u32(&mut bytes, *order);
            push_u64(&mut bytes, *child);
        }
        match node.accessibility_ref {
            Some(target) => {
                bytes.push(1);
                push_u64(&mut bytes, target);
            }
            None => bytes.push(0),
        }
    }
    bytes
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_bytes(bytes: &mut Vec<u8>, value: &[u8]) {
    push_u32(bytes, value.len() as u32);
    bytes.extend_from_slice(value);
}

fn push_source(bytes: &mut Vec<u8>, source: Option<(u64, u32, u32)>) {
    match source {
        Some((source_id, start, end)) => {
            bytes.push(1);
            push_u64(bytes, source_id);
            push_u32(bytes, start);
            push_u32(bytes, end);
        }
        None => bytes.push(0),
    }
}

fn replace_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn replace_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn error(bytes: &[u8], configured: StaticUiArtifactV1Limits) -> StaticUiArtifactV1Error {
    let first = verify_static_ui_ir_artifact_v1(bytes, RoleDictionary::current(), configured)
        .expect_err("fixture must be rejected");
    let second = verify_static_ui_ir_artifact_v1(bytes, RoleDictionary::current(), configured)
        .expect_err("repeated fixture must be rejected");
    assert_eq!(first, second);
    first
}

fn assert_kind(
    bytes: &[u8],
    stage: StaticUiArtifactV1Stage,
    kind: StaticUiArtifactV1ErrorKind,
) -> StaticUiArtifactV1Error {
    let error = error(bytes, limits());
    assert_eq!(error.stage(), stage);
    assert_eq!(error.kind(), &kind);
    error
}

fn static_ir_kinds(error: &StaticUiArtifactV1Error) -> Vec<StaticUiIrDiagnosticKind> {
    match error.kind() {
        StaticUiArtifactV1ErrorKind::StaticIr(diagnostics) => diagnostics
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.kind())
            .collect(),
        other => panic!("expected Static IR diagnostics, got {other:?}"),
    }
}

fn assert_static_ir(bytes: &[u8], expected: StaticUiIrDiagnosticKind) {
    let error = error(bytes, limits());
    assert_eq!(error.stage(), StaticUiArtifactV1Stage::StaticIr);
    assert!(static_ir_kinds(&error).contains(&expected));
}

fn id<T>(result: Result<T, super::contract_primitives::ContractPrimitiveError>) -> T {
    result.expect("fixture identifier must be nonzero")
}

fn source(source: u64, start: u32, end: u32) -> SourceRef {
    SourceRef::new(
        id(SourceId::new(source)),
        SourceSpan::new(start, end).expect("fixture span must be valid"),
    )
}

fn minimal_document() -> StaticUiDocument {
    let mut document = StaticUiDocument::new(
        id(StaticDocumentId::new(1)),
        Revision::new(0),
        Epoch::new(0),
    );
    document.push_surface(StaticUiSurface::new(
        id(StaticSurfaceId::new(1)),
        id(StaticNodeId::new(10)),
        id(CollectionKey::new(1)),
        None,
    ));
    document.push_node(StaticUiNode::new(
        id(StaticNodeId::new(10)),
        RoleId::new("root"),
        id(CollectionKey::new(10)),
        None,
    ));
    document
}

fn structured_document() -> StaticUiDocument {
    let mut document = StaticUiDocument::new(
        id(StaticDocumentId::new(73)),
        Revision::new(7),
        Epoch::new(9),
    );
    document.push_surface(StaticUiSurface::new(
        id(StaticSurfaceId::new(2)),
        id(StaticNodeId::new(20)),
        id(CollectionKey::new(200)),
        Some(source(41, 0, 28)),
    ));
    document.push_node(StaticUiNode::new(
        id(StaticNodeId::new(21)),
        RoleId::new("text"),
        id(CollectionKey::new(101)),
        Some(source(41, 5, 13)),
    ));
    document.push_node(StaticUiNode::new(
        id(StaticNodeId::new(22)),
        RoleId::new("numeric_readout"),
        id(CollectionKey::new(102)),
        Some(source(41, 14, 28)),
    ));
    let mut root = StaticUiNode::new(
        id(StaticNodeId::new(20)),
        RoleId::new("root"),
        id(CollectionKey::new(201)),
        Some(source(41, 0, 4)),
    );
    root.push_child(StaticUiChild::new(
        id(StaticNodeId::new(21)),
        ChildOrder::new(1),
    ));
    root.push_child(StaticUiChild::new(
        id(StaticNodeId::new(22)),
        ChildOrder::new(4),
    ));
    root.set_accessibility_ref(id(StaticNodeId::new(22)));
    document.push_node(root);
    document
}

#[test]
fn ui_dna2_wp3b_artifact_golden_vectors_verify_and_round_trip_exactly() {
    for (bytes, expected) in [
        (MINIMAL, minimal_document()),
        (STRUCTURED, structured_document()),
    ] {
        let first = verify_static_ui_ir_artifact_v1(bytes, RoleDictionary::current(), limits())
            .expect("golden vector must verify");
        let second = verify_static_ui_ir_artifact_v1(bytes, RoleDictionary::current(), limits())
            .expect("repeated golden verification must succeed");
        assert_eq!(first, second);
        assert_eq!(first.document(), &expected);
        assert_eq!(
            expected
                .canonical_bytes(RoleDictionary::current())
                .expect("fixture document must encode"),
            bytes
        );
    }
}

#[test]
fn ui_dna2_wp3b_artifact_golden_vectors_match_independent_layout_encoding() {
    let (minimal_surfaces, minimal_nodes) = minimal_specs();
    assert_eq!(encode(1, 0, 0, &minimal_surfaces, &minimal_nodes), MINIMAL);

    let surfaces = vec![SurfaceSpec {
        id: 2,
        root: 20,
        key: 200,
        source: Some((41, 0, 28)),
    }];
    let nodes = vec![
        NodeSpec {
            id: 21,
            role: "text",
            key: 101,
            source: Some((41, 5, 13)),
            children: Vec::new(),
            accessibility_ref: None,
        },
        NodeSpec {
            id: 22,
            role: "numeric_readout",
            key: 102,
            source: Some((41, 14, 28)),
            children: Vec::new(),
            accessibility_ref: None,
        },
        NodeSpec {
            id: 20,
            role: "root",
            key: 201,
            source: Some((41, 0, 4)),
            children: vec![(1, 21), (4, 22)],
            accessibility_ref: Some(22),
        },
    ];
    assert_eq!(encode(73, 7, 9, &surfaces, &nodes), STRUCTURED);
}

#[test]
fn ui_dna2_wp3b_artifact_rejects_malformed_header_and_byte_domain_rows() {
    let truncated_primitive = &MINIMAL[..3];
    let e = assert_kind(
        truncated_primitive,
        StaticUiArtifactV1Stage::Decode,
        StaticUiArtifactV1ErrorKind::TruncatedPrimitive,
    );
    assert_eq!(e.byte_offset(), Some(0));

    assert_kind(
        &MINIMAL[..10],
        StaticUiArtifactV1Stage::Decode,
        StaticUiArtifactV1ErrorKind::TruncatedByteString,
    );
    let mut trailing = MINIMAL.to_vec();
    trailing.push(0);
    assert_kind(
        &trailing,
        StaticUiArtifactV1Stage::CompleteConsumption,
        StaticUiArtifactV1ErrorKind::TrailingBytes,
    );
    let mut wrong_magic = MINIMAL.to_vec();
    wrong_magic[4] ^= 1;
    assert_kind(
        &wrong_magic,
        StaticUiArtifactV1Stage::Header,
        StaticUiArtifactV1ErrorKind::InvalidMagic,
    );
    let mut schema = MINIMAL.to_vec();
    replace_u32(&mut schema, 24, 2);
    assert_kind(
        &schema,
        StaticUiArtifactV1Stage::Header,
        StaticUiArtifactV1ErrorKind::UnsupportedSchemaVersion,
    );
    let mut contract = MINIMAL.to_vec();
    replace_u32(&mut contract, 28, 2);
    assert_kind(
        &contract,
        StaticUiArtifactV1Stage::Header,
        StaticUiArtifactV1ErrorKind::UnsupportedContractVersion,
    );
    let overflow = qualify_checked_cursor_overflow();
    assert_eq!(overflow.stage(), StaticUiArtifactV1Stage::Decode);
    assert_eq!(
        overflow.kind(),
        &StaticUiArtifactV1ErrorKind::ArithmeticOverflow
    );
}

#[test]
fn ui_dna2_wp3b_artifact_enforces_each_caller_supplied_resource_limit() {
    let mut configured = limits();
    configured.max_input_bytes = MINIMAL.len() - 1;
    assert_eq!(
        error(MINIMAL, configured).stage(),
        StaticUiArtifactV1Stage::Resource
    );

    for (field, expected) in [
        (0, StaticUiArtifactV1ErrorKind::SurfaceLimitExceeded),
        (1, StaticUiArtifactV1ErrorKind::NodeLimitExceeded),
        (2, StaticUiArtifactV1ErrorKind::ChildLimitExceeded),
        (3, StaticUiArtifactV1ErrorKind::RoleLengthLimitExceeded),
    ] {
        let mut configured = limits();
        match field {
            0 => configured.max_surfaces = 0,
            1 => configured.max_nodes = 0,
            2 => configured.max_children_per_node = 0,
            _ => configured.max_role_bytes = 3,
        }
        let bytes = if field == 2 { STRUCTURED } else { MINIMAL };
        let actual = error(bytes, configured);
        assert_eq!(actual.stage(), StaticUiArtifactV1Stage::Resource);
        assert_eq!(actual.kind(), &expected);
    }
}

#[test]
fn ui_dna2_wp3b_artifact_rejects_utf8_option_span_and_zero_representation_rows() {
    let mut utf8 = MINIMAL.to_vec();
    utf8[101] = 0xff;
    assert_kind(
        &utf8,
        StaticUiArtifactV1Stage::Decode,
        StaticUiArtifactV1ErrorKind::InvalidUtf8,
    );
    let mut option = MINIMAL.to_vec();
    option[84] = 2;
    assert_kind(
        &option,
        StaticUiArtifactV1Stage::Decode,
        StaticUiArtifactV1ErrorKind::InvalidOptionTag,
    );
    let mut accessibility_option = MINIMAL.to_vec();
    accessibility_option[MINIMAL.len() - 1] = 2;
    assert_kind(
        &accessibility_option,
        StaticUiArtifactV1Stage::Decode,
        StaticUiArtifactV1ErrorKind::InvalidOptionTag,
    );
    let mut span = STRUCTURED.to_vec();
    replace_u32(&mut span, 93, 29);
    assert_kind(
        &span,
        StaticUiArtifactV1Stage::Representation,
        StaticUiArtifactV1ErrorKind::InvalidSourceSpan,
    );
    let mut zero_id = MINIMAL.to_vec();
    replace_u64(&mut zero_id, 32, 0);
    assert_kind(
        &zero_id,
        StaticUiArtifactV1Stage::Representation,
        StaticUiArtifactV1ErrorKind::ZeroIdentifier(StaticUiArtifactV1Field::DocumentId),
    );
    let mut zero_key = MINIMAL.to_vec();
    replace_u64(&mut zero_key, 76, 0);
    assert_kind(
        &zero_key,
        StaticUiArtifactV1Stage::Representation,
        StaticUiArtifactV1ErrorKind::ZeroCollectionKey,
    );
}

#[test]
fn ui_dna2_wp3b_artifact_preserves_unknown_role_identity_and_offset() {
    let mut unknown = MINIMAL.to_vec();
    unknown[101..105].copy_from_slice(b"nope");
    let error = error(&unknown, limits());
    assert_eq!(error.stage(), StaticUiArtifactV1Stage::Role);
    assert_eq!(error.byte_offset(), Some(101));
    assert_eq!(
        error.kind(),
        &StaticUiArtifactV1ErrorKind::UnknownRole {
            node_id: 10,
            authored_role: String::from("nope"),
            role_byte_offset: 101,
        }
    );
}

#[test]
fn ui_dna2_wp3b_artifact_preserves_duplicate_id_and_key_diagnostics() {
    let (surfaces, nodes) = minimal_specs();
    let duplicate_surface_id = vec![
        surfaces[0].clone(),
        SurfaceSpec {
            key: 2,
            ..surfaces[0].clone()
        },
    ];
    assert_static_ir(
        &encode(1, 0, 0, &duplicate_surface_id, &nodes),
        StaticUiIrDiagnosticKind::DuplicateSurfaceId,
    );
    let duplicate_surface_key = vec![
        surfaces[0].clone(),
        SurfaceSpec {
            id: 2,
            ..surfaces[0].clone()
        },
    ];
    assert_static_ir(
        &encode(1, 0, 0, &duplicate_surface_key, &nodes),
        StaticUiIrDiagnosticKind::DuplicateSurfaceKey,
    );
    let duplicate_node_id = vec![
        nodes[0].clone(),
        NodeSpec {
            key: 11,
            ..nodes[0].clone()
        },
    ];
    assert_static_ir(
        &encode(1, 0, 0, &surfaces, &duplicate_node_id),
        StaticUiIrDiagnosticKind::DuplicateNodeId,
    );
    let duplicate_node_key = vec![
        nodes[0].clone(),
        NodeSpec {
            id: 11,
            ..nodes[0].clone()
        },
    ];
    assert_static_ir(
        &encode(1, 0, 0, &surfaces, &duplicate_node_key),
        StaticUiIrDiagnosticKind::DuplicateNodeKey,
    );
}

#[test]
fn ui_dna2_wp3b_artifact_preserves_missing_and_duplicate_child_diagnostics() {
    let (mut surfaces, mut nodes) = minimal_specs();
    surfaces[0].root = 99;
    assert_static_ir(
        &encode(1, 0, 0, &surfaces, &nodes),
        StaticUiIrDiagnosticKind::MissingRoot,
    );
    let (surfaces, _) = minimal_specs();
    nodes[0].children = vec![(1, 99)];
    assert_static_ir(
        &encode(1, 0, 0, &surfaces, &nodes),
        StaticUiIrDiagnosticKind::MissingChild,
    );

    let child = NodeSpec {
        id: 11,
        role: "text",
        key: 11,
        source: None,
        children: Vec::new(),
        accessibility_ref: None,
    };
    let mut duplicate_child_nodes = nodes.clone();
    duplicate_child_nodes[0].children = vec![(1, 11), (2, 11)];
    duplicate_child_nodes.push(child.clone());
    assert_static_ir(
        &encode(1, 0, 0, &surfaces, &duplicate_child_nodes),
        StaticUiIrDiagnosticKind::DuplicateChild,
    );
    let mut duplicate_order_nodes = nodes;
    duplicate_order_nodes[0].children = vec![(1, 11), (1, 12)];
    duplicate_order_nodes.push(child);
    duplicate_order_nodes.push(NodeSpec {
        id: 12,
        role: "text",
        key: 12,
        source: None,
        children: Vec::new(),
        accessibility_ref: None,
    });
    assert_static_ir(
        &encode(1, 0, 0, &surfaces, &duplicate_order_nodes),
        StaticUiIrDiagnosticKind::DuplicateChildOrder,
    );
}

#[test]
fn ui_dna2_wp3b_artifact_preserves_forest_diagnostics() {
    let surfaces = vec![SurfaceSpec {
        id: 1,
        root: 10,
        key: 1,
        source: None,
    }];
    let cycle_nodes = vec![
        NodeSpec {
            id: 10,
            role: "root",
            key: 10,
            source: None,
            children: vec![(1, 11)],
            accessibility_ref: None,
        },
        NodeSpec {
            id: 11,
            role: "text",
            key: 11,
            source: None,
            children: vec![(1, 10)],
            accessibility_ref: None,
        },
    ];
    assert_static_ir(
        &encode(1, 0, 0, &surfaces, &cycle_nodes),
        StaticUiIrDiagnosticKind::Cycle,
    );

    let multiple_parent_nodes = vec![
        NodeSpec {
            id: 10,
            role: "root",
            key: 10,
            source: None,
            children: vec![(1, 11), (2, 12)],
            accessibility_ref: None,
        },
        NodeSpec {
            id: 11,
            role: "text",
            key: 11,
            source: None,
            children: vec![(1, 12)],
            accessibility_ref: None,
        },
        NodeSpec {
            id: 12,
            role: "text",
            key: 12,
            source: None,
            children: Vec::new(),
            accessibility_ref: None,
        },
    ];
    assert_static_ir(
        &encode(1, 0, 0, &surfaces, &multiple_parent_nodes),
        StaticUiIrDiagnosticKind::MultipleParents,
    );

    let shared_surfaces = vec![
        surfaces[0].clone(),
        SurfaceSpec {
            id: 2,
            root: 11,
            key: 2,
            source: None,
        },
    ];
    let shared_nodes = vec![
        NodeSpec {
            id: 10,
            role: "root",
            key: 10,
            source: None,
            children: vec![(1, 12)],
            accessibility_ref: None,
        },
        NodeSpec {
            id: 11,
            role: "root",
            key: 11,
            source: None,
            children: vec![(1, 12)],
            accessibility_ref: None,
        },
        NodeSpec {
            id: 12,
            role: "text",
            key: 12,
            source: None,
            children: Vec::new(),
            accessibility_ref: None,
        },
    ];
    assert_static_ir(
        &encode(1, 0, 0, &shared_surfaces, &shared_nodes),
        StaticUiIrDiagnosticKind::SharedAcrossSurfaces,
    );

    let unreachable_nodes = vec![
        NodeSpec {
            id: 10,
            role: "root",
            key: 10,
            source: None,
            children: Vec::new(),
            accessibility_ref: None,
        },
        NodeSpec {
            id: 11,
            role: "text",
            key: 11,
            source: None,
            children: Vec::new(),
            accessibility_ref: None,
        },
    ];
    assert_static_ir(
        &encode(1, 0, 0, &surfaces, &unreachable_nodes),
        StaticUiIrDiagnosticKind::UnreachableNode,
    );
}

#[test]
fn ui_dna2_wp3b_artifact_rejects_each_noncanonical_order_and_reencoding_mismatch() {
    let surfaces = vec![
        SurfaceSpec {
            id: 2,
            root: 20,
            key: 2,
            source: None,
        },
        SurfaceSpec {
            id: 1,
            root: 10,
            key: 1,
            source: None,
        },
    ];
    let canonical_nodes = vec![
        NodeSpec {
            id: 10,
            role: "root",
            key: 10,
            source: None,
            children: Vec::new(),
            accessibility_ref: None,
        },
        NodeSpec {
            id: 20,
            role: "root",
            key: 20,
            source: None,
            children: Vec::new(),
            accessibility_ref: None,
        },
    ];
    assert_kind(
        &encode(1, 0, 0, &surfaces, &canonical_nodes),
        StaticUiArtifactV1Stage::Canonical,
        StaticUiArtifactV1ErrorKind::CanonicalMismatch,
    );

    let canonical_surfaces = vec![surfaces[1].clone(), surfaces[0].clone()];
    let reversed_nodes = vec![canonical_nodes[1].clone(), canonical_nodes[0].clone()];
    assert_kind(
        &encode(1, 0, 0, &canonical_surfaces, &reversed_nodes),
        StaticUiArtifactV1Stage::Canonical,
        StaticUiArtifactV1ErrorKind::CanonicalMismatch,
    );

    let child_nodes = vec![
        NodeSpec {
            id: 11,
            role: "text",
            key: 11,
            source: None,
            children: Vec::new(),
            accessibility_ref: None,
        },
        NodeSpec {
            id: 12,
            role: "text",
            key: 12,
            source: None,
            children: Vec::new(),
            accessibility_ref: None,
        },
        NodeSpec {
            id: 10,
            role: "root",
            key: 20,
            source: None,
            children: vec![(2, 12), (1, 11)],
            accessibility_ref: None,
        },
    ];
    assert_kind(
        &encode(
            1,
            0,
            0,
            &[SurfaceSpec {
                id: 1,
                root: 10,
                key: 1,
                source: None,
            }],
            &child_nodes,
        ),
        StaticUiArtifactV1Stage::Canonical,
        StaticUiArtifactV1ErrorKind::CanonicalMismatch,
    );

    let all_reordered = vec![canonical_nodes[1].clone(), canonical_nodes[0].clone()];
    assert_kind(
        &encode(1, 0, 0, &surfaces, &all_reordered),
        StaticUiArtifactV1Stage::Canonical,
        StaticUiArtifactV1ErrorKind::CanonicalMismatch,
    );
}

#[test]
fn ui_dna2_wp3b_artifact_failure_precedence_is_fail_closed() {
    let mut oversized_bad_magic = MINIMAL.to_vec();
    oversized_bad_magic[4] ^= 1;
    let mut configured = limits();
    configured.max_input_bytes = MINIMAL.len() - 1;
    assert_eq!(
        error(&oversized_bad_magic, configured).stage(),
        StaticUiArtifactV1Stage::Resource
    );

    let mut wrong_magic_trailing = MINIMAL.to_vec();
    wrong_magic_trailing[4] ^= 1;
    wrong_magic_trailing.push(0);
    assert_eq!(
        error(&wrong_magic_trailing, limits()).stage(),
        StaticUiArtifactV1Stage::Header
    );

    let mut wrong_magic_truncated = MINIMAL[..MINIMAL.len() - 1].to_vec();
    wrong_magic_truncated[4] ^= 1;
    assert_eq!(
        error(&wrong_magic_truncated, limits()).stage(),
        StaticUiArtifactV1Stage::Decode
    );

    let (mut surfaces, nodes) = minimal_specs();
    surfaces[0].root = 99;
    let mut trailing_structural = encode(1, 0, 0, &surfaces, &nodes);
    trailing_structural.push(0);
    assert_eq!(
        error(&trailing_structural, limits()).stage(),
        StaticUiArtifactV1Stage::CompleteConsumption
    );

    let mut structural_unknown = nodes;
    structural_unknown[0].role = "unknown";
    assert_eq!(
        error(&encode(1, 0, 0, &surfaces, &structural_unknown), limits()).stage(),
        StaticUiArtifactV1Stage::StaticIr
    );

    let (valid_surfaces, mut unknown_nodes) = minimal_specs();
    unknown_nodes[0].role = "unknown";
    assert_eq!(
        error(&encode(1, 0, 0, &valid_surfaces, &unknown_nodes), limits()).stage(),
        StaticUiArtifactV1Stage::Role
    );
}

#[test]
fn ui_dna2_wp3b_artifact_all_prefixes_and_bounded_mutations_are_panic_free() {
    for length in 0..=MINIMAL.len() {
        let first = verify_static_ui_ir_artifact_v1(
            &MINIMAL[..length],
            RoleDictionary::current(),
            limits(),
        );
        let second = verify_static_ui_ir_artifact_v1(
            &MINIMAL[..length],
            RoleDictionary::current(),
            limits(),
        );
        assert_eq!(first, second);
        assert_eq!(first.is_ok(), length == MINIMAL.len());
    }
    for position in (0..STRUCTURED.len()).step_by(7) {
        let mut mutated = STRUCTURED.to_vec();
        mutated[position] ^= 0x5a;
        let first = verify_static_ui_ir_artifact_v1(&mutated, RoleDictionary::current(), limits());
        let second = verify_static_ui_ir_artifact_v1(&mutated, RoleDictionary::current(), limits());
        assert_eq!(first, second);
    }
}
