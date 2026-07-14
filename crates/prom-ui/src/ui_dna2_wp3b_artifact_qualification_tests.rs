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

const MAGIC_OFFSET: usize = 4;
const SCHEMA_VERSION_OFFSET: usize = 24;
const CONTRACT_VERSION_OFFSET: usize = 28;
const DOCUMENT_ID_OFFSET: usize = 32;
const MINIMAL_SURFACE_KEY_OFFSET: usize = 76;
const MINIMAL_SURFACE_SOURCE_TAG_OFFSET: usize = 84;
const MINIMAL_ROLE_OFFSET: usize = 101;
const MINIMAL_ACCESSIBILITY_TAG_OFFSET: usize = 118;
const STRUCTURED_SURFACE_SPAN_START_OFFSET: usize = 93;
const STRUCTURED_FIRST_ROLE_OFFSET: usize = 117;
const STRUCTURED_FIRST_NODE_OFFSET: usize = 105;
const STRUCTURED_SECOND_NODE_OFFSET: usize = 151;
const STRUCTURED_ROOT_NODE_OFFSET: usize = 208;
const STRUCTURED_FIRST_CHILD_OFFSET: usize = 253;
const STRUCTURED_SECOND_CHILD_OFFSET: usize = 265;
const STRUCTURED_ACCESSIBILITY_TAG_OFFSET: usize = 277;
const STRUCTURED_ACCESSIBILITY_ID_OFFSET: usize = 278;
const TWO_SURFACE_FIRST_RECORD_OFFSET: usize = 60;
const TWO_SURFACE_SECOND_RECORD_OFFSET: usize = 85;
const TWO_SURFACE_NODE_COUNT_OFFSET: usize = 110;

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
    byte_offset: Option<usize>,
) -> StaticUiArtifactV1Error {
    let error = error(bytes, limits());
    assert_eq!(error.stage(), stage);
    assert_eq!(error.kind(), &kind);
    assert_eq!(error.byte_offset(), byte_offset);
    error
}

type DiagnosticTuple = (StaticUiIrDiagnosticKind, Option<SourceRef>, u64, u64);

fn static_ir_diagnostics(error: &StaticUiArtifactV1Error) -> Vec<DiagnosticTuple> {
    match error.kind() {
        StaticUiArtifactV1ErrorKind::StaticIr(diagnostics) => diagnostics
            .diagnostics()
            .iter()
            .map(|diagnostic| {
                let coordinate = diagnostic.coordinate();
                (
                    diagnostic.kind(),
                    coordinate.source(),
                    coordinate.domain(),
                    coordinate.secondary(),
                )
            })
            .collect(),
        other => panic!("expected Static IR diagnostics, got {other:?}"),
    }
}

fn assert_static_ir(bytes: &[u8], expected: &[DiagnosticTuple]) {
    let error = error(bytes, limits());
    assert_eq!(error.stage(), StaticUiArtifactV1Stage::StaticIr);
    assert_eq!(error.byte_offset(), None);
    assert_eq!(static_ir_diagnostics(&error), expected);
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

// Normative invalid-artifact matrix:
//  1 truncated primitive -> rejects_malformed_header_and_byte_domain_rows
//  2 truncated byte string -> rejects_malformed_header_and_byte_domain_rows
//  3 trailing bytes -> named_mutations_are_guaranteed_rejections
//  4 wrong magic -> named_mutations_are_guaranteed_rejections
//  5 unsupported schema -> named_mutations_are_guaranteed_rejections
//  6 unsupported contract -> named_mutations_are_guaranteed_rejections
//  7 arithmetic overflow -> rejects_malformed_header_and_byte_domain_rows
//  8 resource limits -> enforces_each_caller_supplied_resource_limit
//  9 invalid UTF-8 -> named_mutations_are_guaranteed_rejections
// 10 invalid option tag -> named_mutations_are_guaranteed_rejections (source/accessibility)
// 11 invalid SourceSpan -> named_mutations_are_guaranteed_rejections
// 12 zero identifier/key -> named_mutations_are_guaranteed_rejections
// 13 unknown role -> preserves_unknown_role_identity_and_offset
// 14 duplicate IDs/keys -> preserves_duplicate_id_and_key_diagnostics (surface/node)
// 15 missing root/child -> preserves_missing_and_duplicate_child_diagnostics
// 16 duplicate child/order -> preserves_missing_and_duplicate_child_diagnostics
// 17 cycle -> preserves_forest_diagnostics
// 18 multiple parents/shared ownership -> preserves_forest_diagnostics
// 19 unreachable node -> preserves_forest_diagnostics
// 20 noncanonical surface/node order -> rejects_each_noncanonical_order_and_reencoding_mismatch
// 21 noncanonical child order -> rejects_each_noncanonical_order_and_reencoding_mismatch
// 22 canonical mismatch -> named_mutations_are_guaranteed_rejections (node/child records)
#[test]
fn ui_dna2_wp3b_artifact_rejects_malformed_header_and_byte_domain_rows() {
    let truncated_primitive = &MINIMAL[..3];
    let e = assert_kind(
        truncated_primitive,
        StaticUiArtifactV1Stage::Decode,
        StaticUiArtifactV1ErrorKind::TruncatedPrimitive,
        Some(0),
    );
    assert_eq!(e.byte_offset(), Some(0));

    assert_kind(
        &MINIMAL[..10],
        StaticUiArtifactV1Stage::Decode,
        StaticUiArtifactV1ErrorKind::TruncatedByteString,
        Some(4),
    );
    let mut trailing = MINIMAL.to_vec();
    trailing.push(0);
    assert_kind(
        &trailing,
        StaticUiArtifactV1Stage::CompleteConsumption,
        StaticUiArtifactV1ErrorKind::TrailingBytes,
        Some(MINIMAL.len()),
    );
    let mut wrong_magic = MINIMAL.to_vec();
    assert_eq!(wrong_magic[MAGIC_OFFSET], b'U');
    wrong_magic[MAGIC_OFFSET] ^= 1;
    assert_kind(
        &wrong_magic,
        StaticUiArtifactV1Stage::Header,
        StaticUiArtifactV1ErrorKind::InvalidMagic,
        Some(4),
    );
    let mut schema = MINIMAL.to_vec();
    assert_eq!(
        &schema[SCHEMA_VERSION_OFFSET..SCHEMA_VERSION_OFFSET + 4],
        &1_u32.to_le_bytes()
    );
    replace_u32(&mut schema, SCHEMA_VERSION_OFFSET, 2);
    assert_kind(
        &schema,
        StaticUiArtifactV1Stage::Header,
        StaticUiArtifactV1ErrorKind::UnsupportedSchemaVersion,
        Some(24),
    );
    let mut contract = MINIMAL.to_vec();
    assert_eq!(
        &contract[CONTRACT_VERSION_OFFSET..CONTRACT_VERSION_OFFSET + 4],
        &1_u32.to_le_bytes()
    );
    replace_u32(&mut contract, CONTRACT_VERSION_OFFSET, 2);
    assert_kind(
        &contract,
        StaticUiArtifactV1Stage::Header,
        StaticUiArtifactV1ErrorKind::UnsupportedContractVersion,
        Some(28),
    );
    let overflow = qualify_checked_cursor_overflow();
    assert_eq!(overflow.stage(), StaticUiArtifactV1Stage::Decode);
    assert_eq!(
        overflow.kind(),
        &StaticUiArtifactV1ErrorKind::ArithmeticOverflow
    );
    assert_eq!(overflow.byte_offset(), Some(1));
}

#[test]
fn ui_dna2_wp3b_artifact_enforces_each_caller_supplied_resource_limit() {
    let mut configured = limits();
    configured.max_input_bytes = MINIMAL.len() - 1;
    let actual = error(MINIMAL, configured);
    assert_eq!(actual.stage(), StaticUiArtifactV1Stage::Resource);
    assert_eq!(actual.kind(), &StaticUiArtifactV1ErrorKind::InputTooLarge);
    assert_eq!(actual.byte_offset(), None);

    for (field, expected, expected_offset) in [
        (0, StaticUiArtifactV1ErrorKind::SurfaceLimitExceeded, 56),
        (1, StaticUiArtifactV1ErrorKind::NodeLimitExceeded, 85),
        (2, StaticUiArtifactV1ErrorKind::ChildLimitExceeded, 249),
        (3, StaticUiArtifactV1ErrorKind::RoleLengthLimitExceeded, 97),
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
        assert_eq!(actual.byte_offset(), Some(expected_offset));
    }
}

#[test]
fn ui_dna2_wp3b_artifact_rejects_utf8_option_span_and_zero_representation_rows() {
    let mut utf8 = MINIMAL.to_vec();
    assert_eq!(utf8[MINIMAL_ROLE_OFFSET], b'r');
    utf8[MINIMAL_ROLE_OFFSET] = 0xff;
    assert_kind(
        &utf8,
        StaticUiArtifactV1Stage::Decode,
        StaticUiArtifactV1ErrorKind::InvalidUtf8,
        Some(101),
    );
    let mut option = MINIMAL.to_vec();
    assert_eq!(option[MINIMAL_SURFACE_SOURCE_TAG_OFFSET], 0);
    option[MINIMAL_SURFACE_SOURCE_TAG_OFFSET] = 2;
    assert_kind(
        &option,
        StaticUiArtifactV1Stage::Decode,
        StaticUiArtifactV1ErrorKind::InvalidOptionTag,
        Some(84),
    );
    let mut accessibility_option = MINIMAL.to_vec();
    assert_eq!(accessibility_option[MINIMAL_ACCESSIBILITY_TAG_OFFSET], 0);
    accessibility_option[MINIMAL_ACCESSIBILITY_TAG_OFFSET] = 2;
    assert_kind(
        &accessibility_option,
        StaticUiArtifactV1Stage::Decode,
        StaticUiArtifactV1ErrorKind::InvalidOptionTag,
        Some(118),
    );
    let mut span = STRUCTURED.to_vec();
    assert_eq!(
        &span[STRUCTURED_SURFACE_SPAN_START_OFFSET..STRUCTURED_SURFACE_SPAN_START_OFFSET + 4],
        &0_u32.to_le_bytes()
    );
    replace_u32(&mut span, STRUCTURED_SURFACE_SPAN_START_OFFSET, 29);
    assert_kind(
        &span,
        StaticUiArtifactV1Stage::Representation,
        StaticUiArtifactV1ErrorKind::InvalidSourceSpan,
        Some(93),
    );
    let mut zero_id = MINIMAL.to_vec();
    assert_eq!(
        &zero_id[DOCUMENT_ID_OFFSET..DOCUMENT_ID_OFFSET + 8],
        &1_u64.to_le_bytes()
    );
    replace_u64(&mut zero_id, DOCUMENT_ID_OFFSET, 0);
    assert_kind(
        &zero_id,
        StaticUiArtifactV1Stage::Representation,
        StaticUiArtifactV1ErrorKind::ZeroIdentifier(StaticUiArtifactV1Field::DocumentId),
        Some(32),
    );
    let mut zero_key = MINIMAL.to_vec();
    assert_eq!(
        &zero_key[MINIMAL_SURFACE_KEY_OFFSET..MINIMAL_SURFACE_KEY_OFFSET + 8],
        &1_u64.to_le_bytes()
    );
    replace_u64(&mut zero_key, MINIMAL_SURFACE_KEY_OFFSET, 0);
    assert_kind(
        &zero_key,
        StaticUiArtifactV1Stage::Representation,
        StaticUiArtifactV1ErrorKind::ZeroCollectionKey,
        Some(76),
    );
}

#[test]
fn ui_dna2_wp3b_artifact_preserves_unknown_role_identity_and_offset() {
    let mut unknown = MINIMAL.to_vec();
    assert_eq!(
        &unknown[MINIMAL_ROLE_OFFSET..MINIMAL_ROLE_OFFSET + 4],
        b"root"
    );
    unknown[MINIMAL_ROLE_OFFSET..MINIMAL_ROLE_OFFSET + 4].copy_from_slice(b"nope");
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
        &[
            (StaticUiIrDiagnosticKind::DuplicateSurfaceId, None, 1, 0),
            (StaticUiIrDiagnosticKind::DuplicateSurfaceRoot, None, 10, 1),
            (StaticUiIrDiagnosticKind::SharedAcrossSurfaces, None, 10, 1),
        ],
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
        &[
            (StaticUiIrDiagnosticKind::DuplicateSurfaceKey, None, 1, 0),
            (StaticUiIrDiagnosticKind::DuplicateSurfaceRoot, None, 10, 2),
            (StaticUiIrDiagnosticKind::SharedAcrossSurfaces, None, 10, 1),
        ],
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
        &[
            (StaticUiIrDiagnosticKind::DuplicateNodeId, None, 10, 0),
            (StaticUiIrDiagnosticKind::UnreachableNode, None, 10, 0),
        ],
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
        &[
            (StaticUiIrDiagnosticKind::DuplicateNodeKey, None, 10, 0),
            (StaticUiIrDiagnosticKind::UnreachableNode, None, 11, 0),
        ],
    );
}

#[test]
fn ui_dna2_wp3b_artifact_preserves_missing_and_duplicate_child_diagnostics() {
    let (mut surfaces, mut nodes) = minimal_specs();
    surfaces[0].root = 99;
    assert_static_ir(
        &encode(1, 0, 0, &surfaces, &nodes),
        &[
            (StaticUiIrDiagnosticKind::MissingRoot, None, 99, 0),
            (StaticUiIrDiagnosticKind::UnreachableNode, None, 10, 0),
        ],
    );
    let (surfaces, _) = minimal_specs();
    nodes[0].children = vec![(1, 99)];
    assert_static_ir(
        &encode(1, 0, 0, &surfaces, &nodes),
        &[(StaticUiIrDiagnosticKind::MissingChild, None, 10, 99)],
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
        &[
            (StaticUiIrDiagnosticKind::DuplicateChild, None, 10, 11),
            (StaticUiIrDiagnosticKind::MultipleParents, None, 11, 2),
        ],
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
        &[(StaticUiIrDiagnosticKind::DuplicateChildOrder, None, 10, 1)],
    );
}

#[test]
fn ui_dna2_wp3b_artifact_preserves_invalid_accessibility_diagnostic() {
    let (surfaces, mut nodes) = minimal_specs();
    nodes[0].accessibility_ref = Some(99);
    assert_static_ir(
        &encode(1, 0, 0, &surfaces, &nodes),
        &[(
            StaticUiIrDiagnosticKind::InvalidAccessibilityReference,
            None,
            10,
            99,
        )],
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
        &[
            (StaticUiIrDiagnosticKind::Cycle, None, 10, 0),
            (StaticUiIrDiagnosticKind::RootUsedAsChild, None, 10, 0),
        ],
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
        &[(StaticUiIrDiagnosticKind::MultipleParents, None, 12, 2)],
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
        &[
            (StaticUiIrDiagnosticKind::MultipleParents, None, 12, 2),
            (StaticUiIrDiagnosticKind::SharedAcrossSurfaces, None, 12, 1),
        ],
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
        &[(StaticUiIrDiagnosticKind::UnreachableNode, None, 11, 0)],
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
        Some(60),
    );

    let canonical_surfaces = vec![surfaces[1].clone(), surfaces[0].clone()];
    let reversed_nodes = vec![canonical_nodes[1].clone(), canonical_nodes[0].clone()];
    assert_kind(
        &encode(1, 0, 0, &canonical_surfaces, &reversed_nodes),
        StaticUiArtifactV1Stage::Canonical,
        StaticUiArtifactV1ErrorKind::CanonicalMismatch,
        Some(114),
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
        Some(178),
    );

    let all_reordered = vec![canonical_nodes[1].clone(), canonical_nodes[0].clone()];
    assert_kind(
        &encode(1, 0, 0, &surfaces, &all_reordered),
        StaticUiArtifactV1Stage::Canonical,
        StaticUiArtifactV1ErrorKind::CanonicalMismatch,
        Some(60),
    );
}

#[test]
fn ui_dna2_wp3b_artifact_failure_precedence_is_fail_closed() {
    let mut oversized_bad_magic = MINIMAL.to_vec();
    assert_eq!(oversized_bad_magic[MAGIC_OFFSET], b'U');
    oversized_bad_magic[MAGIC_OFFSET] ^= 1;
    let mut configured = limits();
    configured.max_input_bytes = MINIMAL.len() - 1;
    let oversized_error = error(&oversized_bad_magic, configured);
    assert_eq!(oversized_error.stage(), StaticUiArtifactV1Stage::Resource);
    assert_eq!(
        oversized_error.kind(),
        &StaticUiArtifactV1ErrorKind::InputTooLarge
    );
    assert_eq!(oversized_error.byte_offset(), None);

    let mut wrong_magic_trailing = MINIMAL.to_vec();
    assert_eq!(wrong_magic_trailing[MAGIC_OFFSET], b'U');
    wrong_magic_trailing[MAGIC_OFFSET] ^= 1;
    wrong_magic_trailing.push(0);
    assert_kind(
        &wrong_magic_trailing,
        StaticUiArtifactV1Stage::Header,
        StaticUiArtifactV1ErrorKind::InvalidMagic,
        Some(MAGIC_OFFSET),
    );

    let mut wrong_magic_truncated = MINIMAL[..MINIMAL.len() - 1].to_vec();
    assert_eq!(wrong_magic_truncated[MAGIC_OFFSET], b'U');
    wrong_magic_truncated[MAGIC_OFFSET] ^= 1;
    assert_kind(
        &wrong_magic_truncated,
        StaticUiArtifactV1Stage::Decode,
        StaticUiArtifactV1ErrorKind::TruncatedPrimitive,
        Some(MINIMAL_ACCESSIBILITY_TAG_OFFSET),
    );

    let (mut surfaces, nodes) = minimal_specs();
    surfaces[0].root = 99;
    let mut trailing_structural = encode(1, 0, 0, &surfaces, &nodes);
    trailing_structural.push(0);
    assert_kind(
        &trailing_structural,
        StaticUiArtifactV1Stage::CompleteConsumption,
        StaticUiArtifactV1ErrorKind::TrailingBytes,
        Some(MINIMAL.len()),
    );

    let mut structural_unknown = nodes;
    structural_unknown[0].role = "unknown";
    let structural_error = error(&encode(1, 0, 0, &surfaces, &structural_unknown), limits());
    assert_eq!(structural_error.stage(), StaticUiArtifactV1Stage::StaticIr);
    assert_ne!(structural_error.stage(), StaticUiArtifactV1Stage::Role);
    assert_eq!(structural_error.byte_offset(), None);
    assert_eq!(
        static_ir_diagnostics(&structural_error),
        [
            (StaticUiIrDiagnosticKind::MissingRoot, None, 99, 0),
            (StaticUiIrDiagnosticKind::UnreachableNode, None, 10, 0),
        ]
    );

    let (valid_surfaces, mut unknown_nodes) = minimal_specs();
    unknown_nodes[0].role = "unknown";
    let role_error = error(&encode(1, 0, 0, &valid_surfaces, &unknown_nodes), limits());
    assert_eq!(role_error.stage(), StaticUiArtifactV1Stage::Role);
    assert_eq!(role_error.byte_offset(), Some(101));
    assert_eq!(
        role_error.kind(),
        &StaticUiArtifactV1ErrorKind::UnknownRole {
            node_id: 10,
            authored_role: String::from("unknown"),
            role_byte_offset: 101,
        }
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
}

#[test]
fn ui_dna2_wp3b_artifact_named_mutations_are_guaranteed_rejections() {
    let mut magic = STRUCTURED.to_vec();
    assert_eq!(magic[MAGIC_OFFSET], b'U');
    magic[MAGIC_OFFSET] ^= 1;
    assert_kind(
        &magic,
        StaticUiArtifactV1Stage::Header,
        StaticUiArtifactV1ErrorKind::InvalidMagic,
        Some(MAGIC_OFFSET),
    );

    let mut schema = STRUCTURED.to_vec();
    assert_eq!(
        &schema[SCHEMA_VERSION_OFFSET..SCHEMA_VERSION_OFFSET + 4],
        &1_u32.to_le_bytes()
    );
    replace_u32(&mut schema, SCHEMA_VERSION_OFFSET, 2);
    assert_kind(
        &schema,
        StaticUiArtifactV1Stage::Header,
        StaticUiArtifactV1ErrorKind::UnsupportedSchemaVersion,
        Some(SCHEMA_VERSION_OFFSET),
    );

    let mut contract = STRUCTURED.to_vec();
    assert_eq!(
        &contract[CONTRACT_VERSION_OFFSET..CONTRACT_VERSION_OFFSET + 4],
        &1_u32.to_le_bytes()
    );
    replace_u32(&mut contract, CONTRACT_VERSION_OFFSET, 2);
    assert_kind(
        &contract,
        StaticUiArtifactV1Stage::Header,
        StaticUiArtifactV1ErrorKind::UnsupportedContractVersion,
        Some(CONTRACT_VERSION_OFFSET),
    );

    let mut invalid_role = STRUCTURED.to_vec();
    assert_eq!(invalid_role[STRUCTURED_FIRST_ROLE_OFFSET], b't');
    invalid_role[STRUCTURED_FIRST_ROLE_OFFSET] = 0xff;
    assert_kind(
        &invalid_role,
        StaticUiArtifactV1Stage::Decode,
        StaticUiArtifactV1ErrorKind::InvalidUtf8,
        Some(STRUCTURED_FIRST_ROLE_OFFSET),
    );

    let mut surface_option = STRUCTURED.to_vec();
    assert_eq!(surface_option[MINIMAL_SURFACE_SOURCE_TAG_OFFSET], 1);
    surface_option[MINIMAL_SURFACE_SOURCE_TAG_OFFSET] = 2;
    assert_kind(
        &surface_option,
        StaticUiArtifactV1Stage::Decode,
        StaticUiArtifactV1ErrorKind::InvalidOptionTag,
        Some(MINIMAL_SURFACE_SOURCE_TAG_OFFSET),
    );

    let mut node_option = STRUCTURED.to_vec();
    assert_eq!(node_option[129], 1);
    node_option[129] = 2;
    assert_kind(
        &node_option,
        StaticUiArtifactV1Stage::Decode,
        StaticUiArtifactV1ErrorKind::InvalidOptionTag,
        Some(129),
    );

    let mut accessibility_option = STRUCTURED.to_vec();
    assert_eq!(accessibility_option[STRUCTURED_ACCESSIBILITY_TAG_OFFSET], 1);
    accessibility_option[STRUCTURED_ACCESSIBILITY_TAG_OFFSET] = 2;
    assert_kind(
        &accessibility_option,
        StaticUiArtifactV1Stage::Decode,
        StaticUiArtifactV1ErrorKind::InvalidOptionTag,
        Some(STRUCTURED_ACCESSIBILITY_TAG_OFFSET),
    );

    let mut document_id = STRUCTURED.to_vec();
    assert_eq!(
        &document_id[DOCUMENT_ID_OFFSET..DOCUMENT_ID_OFFSET + 8],
        &73_u64.to_le_bytes()
    );
    replace_u64(&mut document_id, DOCUMENT_ID_OFFSET, 0);
    assert_kind(
        &document_id,
        StaticUiArtifactV1Stage::Representation,
        StaticUiArtifactV1ErrorKind::ZeroIdentifier(StaticUiArtifactV1Field::DocumentId),
        Some(DOCUMENT_ID_OFFSET),
    );

    let mut surface_id = STRUCTURED.to_vec();
    assert_eq!(&surface_id[60..68], &2_u64.to_le_bytes());
    replace_u64(&mut surface_id, 60, 0);
    assert_kind(
        &surface_id,
        StaticUiArtifactV1Stage::Representation,
        StaticUiArtifactV1ErrorKind::ZeroIdentifier(StaticUiArtifactV1Field::SurfaceId),
        Some(60),
    );

    let mut surface_key = STRUCTURED.to_vec();
    assert_eq!(
        &surface_key[MINIMAL_SURFACE_KEY_OFFSET..MINIMAL_SURFACE_KEY_OFFSET + 8],
        &200_u64.to_le_bytes()
    );
    replace_u64(&mut surface_key, MINIMAL_SURFACE_KEY_OFFSET, 0);
    assert_kind(
        &surface_key,
        StaticUiArtifactV1Stage::Representation,
        StaticUiArtifactV1ErrorKind::ZeroCollectionKey,
        Some(MINIMAL_SURFACE_KEY_OFFSET),
    );

    let mut node_id = STRUCTURED.to_vec();
    assert_eq!(
        &node_id[STRUCTURED_FIRST_NODE_OFFSET..STRUCTURED_FIRST_NODE_OFFSET + 8],
        &21_u64.to_le_bytes()
    );
    replace_u64(&mut node_id, STRUCTURED_FIRST_NODE_OFFSET, 0);
    assert_kind(
        &node_id,
        StaticUiArtifactV1Stage::Representation,
        StaticUiArtifactV1ErrorKind::ZeroIdentifier(StaticUiArtifactV1Field::NodeId),
        Some(STRUCTURED_FIRST_NODE_OFFSET),
    );

    let mut source_id = STRUCTURED.to_vec();
    assert_eq!(&source_id[85..93], &41_u64.to_le_bytes());
    replace_u64(&mut source_id, 85, 0);
    assert_kind(
        &source_id,
        StaticUiArtifactV1Stage::Representation,
        StaticUiArtifactV1ErrorKind::ZeroIdentifier(StaticUiArtifactV1Field::SourceId),
        Some(85),
    );

    let mut span = STRUCTURED.to_vec();
    assert_eq!(
        &span[STRUCTURED_SURFACE_SPAN_START_OFFSET..STRUCTURED_SURFACE_SPAN_START_OFFSET + 4],
        &0_u32.to_le_bytes()
    );
    replace_u32(&mut span, STRUCTURED_SURFACE_SPAN_START_OFFSET, 29);
    assert_kind(
        &span,
        StaticUiArtifactV1Stage::Representation,
        StaticUiArtifactV1ErrorKind::InvalidSourceSpan,
        Some(STRUCTURED_SURFACE_SPAN_START_OFFSET),
    );

    let mut child_id = STRUCTURED.to_vec();
    assert_eq!(&child_id[257..265], &21_u64.to_le_bytes());
    replace_u64(&mut child_id, 257, 0);
    assert_kind(
        &child_id,
        StaticUiArtifactV1Stage::Representation,
        StaticUiArtifactV1ErrorKind::ZeroIdentifier(StaticUiArtifactV1Field::ChildNodeId),
        Some(257),
    );

    let mut accessibility_id = STRUCTURED.to_vec();
    assert_eq!(
        &accessibility_id
            [STRUCTURED_ACCESSIBILITY_ID_OFFSET..STRUCTURED_ACCESSIBILITY_ID_OFFSET + 8],
        &22_u64.to_le_bytes()
    );
    replace_u64(&mut accessibility_id, STRUCTURED_ACCESSIBILITY_ID_OFFSET, 0);
    assert_kind(
        &accessibility_id,
        StaticUiArtifactV1Stage::Representation,
        StaticUiArtifactV1ErrorKind::ZeroIdentifier(StaticUiArtifactV1Field::AccessibilityNodeId),
        Some(STRUCTURED_ACCESSIBILITY_ID_OFFSET),
    );

    let mut trailing = STRUCTURED.to_vec();
    trailing.push(0);
    assert_kind(
        &trailing,
        StaticUiArtifactV1Stage::CompleteConsumption,
        StaticUiArtifactV1ErrorKind::TrailingBytes,
        Some(STRUCTURED.len()),
    );

    assert_eq!(
        &STRUCTURED[STRUCTURED_ACCESSIBILITY_ID_OFFSET..STRUCTURED_ACCESSIBILITY_ID_OFFSET + 8],
        &22_u64.to_le_bytes()
    );
    assert_kind(
        &STRUCTURED[..STRUCTURED.len() - 1],
        StaticUiArtifactV1Stage::Decode,
        StaticUiArtifactV1ErrorKind::TruncatedPrimitive,
        Some(STRUCTURED_ACCESSIBILITY_ID_OFFSET),
    );

    let surface_specs = [
        SurfaceSpec {
            id: 1,
            root: 10,
            key: 1,
            source: None,
        },
        SurfaceSpec {
            id: 2,
            root: 20,
            key: 2,
            source: None,
        },
    ];
    let surface_nodes = [
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
    let mut surface_order = encode(1, 0, 0, &surface_specs, &surface_nodes);
    assert_eq!(
        &surface_order[TWO_SURFACE_FIRST_RECORD_OFFSET..TWO_SURFACE_FIRST_RECORD_OFFSET + 8],
        &1_u64.to_le_bytes()
    );
    assert_eq!(
        &surface_order[TWO_SURFACE_SECOND_RECORD_OFFSET..TWO_SURFACE_SECOND_RECORD_OFFSET + 8],
        &2_u64.to_le_bytes()
    );
    assert_eq!(
        &surface_order[TWO_SURFACE_FIRST_RECORD_OFFSET + 16..TWO_SURFACE_FIRST_RECORD_OFFSET + 24],
        &1_u64.to_le_bytes()
    );
    assert_eq!(
        &surface_order
            [TWO_SURFACE_SECOND_RECORD_OFFSET + 16..TWO_SURFACE_SECOND_RECORD_OFFSET + 24],
        &2_u64.to_le_bytes()
    );
    assert_eq!(surface_order[TWO_SURFACE_SECOND_RECORD_OFFSET - 1], 0);
    assert_eq!(surface_order[TWO_SURFACE_NODE_COUNT_OFFSET - 1], 0);
    let first_surface =
        surface_order[TWO_SURFACE_FIRST_RECORD_OFFSET..TWO_SURFACE_SECOND_RECORD_OFFSET].to_vec();
    let second_surface =
        surface_order[TWO_SURFACE_SECOND_RECORD_OFFSET..TWO_SURFACE_NODE_COUNT_OFFSET].to_vec();
    surface_order[TWO_SURFACE_FIRST_RECORD_OFFSET..TWO_SURFACE_SECOND_RECORD_OFFSET]
        .copy_from_slice(&second_surface);
    surface_order[TWO_SURFACE_SECOND_RECORD_OFFSET..TWO_SURFACE_NODE_COUNT_OFFSET]
        .copy_from_slice(&first_surface);
    assert_kind(
        &surface_order,
        StaticUiArtifactV1Stage::Canonical,
        StaticUiArtifactV1ErrorKind::CanonicalMismatch,
        Some(TWO_SURFACE_FIRST_RECORD_OFFSET),
    );

    assert_eq!(
        &STRUCTURED[STRUCTURED_FIRST_NODE_OFFSET..STRUCTURED_FIRST_NODE_OFFSET + 8],
        &21_u64.to_le_bytes()
    );
    assert_eq!(
        &STRUCTURED[STRUCTURED_SECOND_NODE_OFFSET..STRUCTURED_SECOND_NODE_OFFSET + 8],
        &22_u64.to_le_bytes()
    );
    let mut swapped_nodes = Vec::new();
    swapped_nodes.extend_from_slice(&STRUCTURED[..STRUCTURED_FIRST_NODE_OFFSET]);
    swapped_nodes
        .extend_from_slice(&STRUCTURED[STRUCTURED_SECOND_NODE_OFFSET..STRUCTURED_ROOT_NODE_OFFSET]);
    swapped_nodes.extend_from_slice(
        &STRUCTURED[STRUCTURED_FIRST_NODE_OFFSET..STRUCTURED_SECOND_NODE_OFFSET],
    );
    swapped_nodes.extend_from_slice(&STRUCTURED[STRUCTURED_ROOT_NODE_OFFSET..]);
    assert_kind(
        &swapped_nodes,
        StaticUiArtifactV1Stage::Canonical,
        StaticUiArtifactV1ErrorKind::CanonicalMismatch,
        Some(STRUCTURED_FIRST_NODE_OFFSET),
    );

    assert_eq!(
        &STRUCTURED[STRUCTURED_FIRST_CHILD_OFFSET..STRUCTURED_FIRST_CHILD_OFFSET + 4],
        &1_u32.to_le_bytes()
    );
    assert_eq!(
        &STRUCTURED[STRUCTURED_SECOND_CHILD_OFFSET..STRUCTURED_SECOND_CHILD_OFFSET + 4],
        &4_u32.to_le_bytes()
    );
    let mut swapped_children = STRUCTURED.to_vec();
    let first_child =
        STRUCTURED[STRUCTURED_FIRST_CHILD_OFFSET..STRUCTURED_SECOND_CHILD_OFFSET].to_vec();
    let second_child =
        STRUCTURED[STRUCTURED_SECOND_CHILD_OFFSET..STRUCTURED_ACCESSIBILITY_TAG_OFFSET].to_vec();
    swapped_children[STRUCTURED_FIRST_CHILD_OFFSET..STRUCTURED_SECOND_CHILD_OFFSET]
        .copy_from_slice(&second_child);
    swapped_children[STRUCTURED_SECOND_CHILD_OFFSET..STRUCTURED_ACCESSIBILITY_TAG_OFFSET]
        .copy_from_slice(&first_child);
    assert_kind(
        &swapped_children,
        StaticUiArtifactV1Stage::Canonical,
        StaticUiArtifactV1ErrorKind::CanonicalMismatch,
        Some(STRUCTURED_FIRST_CHILD_OFFSET),
    );
}
