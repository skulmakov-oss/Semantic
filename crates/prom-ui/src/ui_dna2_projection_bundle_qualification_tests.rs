//! ProjectionBundle v0 codec qualification tests: canonical round trip,
//! golden vectors, structural/cross-artifact/compatibility/trust rejection
//! matrix, and deterministic self-consistency verification.

use alloc::string::String;
use alloc::vec::Vec;

use crate::action_ir::ActionIrDocument;
use crate::binding_graph::BindingGraphDocument;
use crate::contract_primitives::{
    ContractVersion, Epoch, Revision, SchemaVersion, SourceId, StaticDocumentId,
};
use crate::projection_bundle::{
    verify_projection_bundle_v0, BundleProvenance, ProjectionBundleCompatContext,
    ProjectionBundleErrorKind as Kind, ProjectionBundleLimits, ProjectionBundleSectionKind,
    ProjectionBundleStage as Stage, ProjectionBundleV0, PROJECTION_BUNDLE_V0_MAGIC,
};
use crate::projection_source_frontend::compile_projection_source_text_with_collection_anchors;
use crate::role_dictionary::RoleDictionary;
use crate::static_ir_artifact::StaticUiArtifactV1Limits;

const SOURCE_RAW: u64 = 71;
const DOCUMENT_RAW: u64 = 71;

fn source_id() -> SourceId {
    SourceId::new(SOURCE_RAW).expect("non-zero test source id")
}

fn document_id() -> StaticDocumentId {
    StaticDocumentId::new(DOCUMENT_RAW).expect("non-zero test document id")
}

fn dictionary() -> RoleDictionary {
    RoleDictionary::current()
}

fn static_ir_limits() -> StaticUiArtifactV1Limits {
    StaticUiArtifactV1Limits {
        max_input_bytes: 16 * 1024,
        max_surfaces: 16,
        max_nodes: 64,
        max_children_per_node: 32,
        max_role_bytes: 64,
    }
}

fn limits() -> ProjectionBundleLimits {
    ProjectionBundleLimits {
        max_bundle_bytes: 64 * 1024,
        max_binding_count: 64,
        max_action_route_count: 64,
        max_collection_anchor_count: 64,
        max_accessibility_entry_count: 64,
        static_ir_limits: static_ir_limits(),
    }
}

fn compat() -> ProjectionBundleCompatContext {
    ProjectionBundleCompatContext {
        accepted_schema_version: SchemaVersion::CURRENT,
        accepted_contract_version: ContractVersion::CURRENT,
        role_dictionary: dictionary(),
    }
}

fn minimal_source() -> String {
    String::from(concat!(
        "projection v0 {\n",
        "revision 1; epoch 1;\n",
        "surface 1 root 10 key 1;\n",
        "node 10 role root key 10 { child 11 order 0; }\n",
        "node 11 role text key 11 {}\n",
        "collection_anchor 11;\n",
        "}"
    ))
}

fn structured_source() -> String {
    String::from(concat!(
        "projection v0 {\n",
        "revision 3; epoch 4;\n",
        "surface 1 root 10 key 1;\n",
        "node 10 role root key 10 { child 11 order 0; child 12 order 1; }\n",
        "node 11 role numeric_readout key 11 {}\n",
        "node 12 role danger_action key 12 {}\n",
        "collection_anchor 11;\n",
        "collection_anchor 12;\n",
        "}"
    ))
}

fn provenance() -> BundleProvenance {
    BundleProvenance::new(
        String::from("ui_dna2_projection_bundle_qualification_tests"),
        source_id(),
        Revision::new(1),
        Epoch::new(1),
    )
}

fn build_bundle(source_text: &str, revision: u64, epoch: u64) -> ProjectionBundleV0 {
    let compiled = compile_projection_source_text_with_collection_anchors(
        source_id(),
        source_text,
        document_id(),
        dictionary(),
    )
    .expect("valid source compiles and qualifies");

    let binding_graph = BindingGraphDocument::build(Vec::new(), compiled.static_document())
        .expect("empty binding graph is trivially valid");
    let action_ir = ActionIrDocument::build(Vec::new(), compiled.static_document())
        .expect("empty action IR is trivially valid");

    ProjectionBundleV0::assemble(
        document_id(),
        Revision::new(revision),
        Epoch::new(epoch),
        dictionary(),
        compiled.static_document().clone(),
        binding_graph,
        action_ir,
        compiled.collection_anchors().clone(),
        provenance(),
    )
}

fn valid_bundle_bytes() -> Vec<u8> {
    build_bundle(&minimal_source(), 1, 1)
        .canonical_bytes(dictionary())
        .expect("valid bundle canonicalizes")
}

fn assert_rejected(bytes: &[u8], stage: Stage, kind: Kind) {
    match verify_projection_bundle_v0(bytes, compat(), limits()) {
        Err(err) => {
            assert_eq!(err.stage(), stage, "unexpected stage for {:?}", err.kind());
            assert_eq!(err.kind(), kind);
        }
        Ok(_) => panic!("expected rejection with kind {kind:?}, bundle verified successfully"),
    }
}

// ---- Positive: round trip, determinism, golden vectors -------------------

#[test]
fn round_trip_valid_minimal_bundle_verifies() {
    let bundle = build_bundle(&minimal_source(), 1, 1);
    let bytes = bundle.canonical_bytes(dictionary()).expect("canonicalizes");
    let verified = verify_projection_bundle_v0(&bytes, compat(), limits()).expect("verifies");

    assert_eq!(verified.bundle().static_document().id(), document_id());
    assert_eq!(verified.bundle().collection_anchors().len(), 1);
    assert_eq!(
        verified.bundle().collection_anchors().anchors()[0].raw(),
        11
    );
    assert_eq!(verified.bundle().accessibility_entries().len(), 2);
    assert_eq!(
        verified.bundle().provenance().compiler_identity(),
        "ui_dna2_projection_bundle_qualification_tests"
    );
}

#[test]
fn round_trip_valid_structured_bundle_verifies() {
    let bundle = build_bundle(&structured_source(), 3, 4);
    let bytes = bundle.canonical_bytes(dictionary()).expect("canonicalizes");
    let verified = verify_projection_bundle_v0(&bytes, compat(), limits()).expect("verifies");

    assert_eq!(verified.bundle().collection_anchors().len(), 2);
    assert_eq!(
        verified.bundle().collection_anchors().anchors()[0].raw(),
        11
    );
    assert_eq!(
        verified.bundle().collection_anchors().anchors()[1].raw(),
        12
    );
    assert_eq!(verified.bundle().accessibility_entries().len(), 3);
}

#[test]
fn verification_is_deterministic() {
    let bytes = valid_bundle_bytes();
    let first = verify_projection_bundle_v0(&bytes, compat(), limits()).expect("verifies");
    let second = verify_projection_bundle_v0(&bytes, compat(), limits()).expect("verifies");
    assert_eq!(first.bundle(), second.bundle());
}

#[test]
fn canonical_encoding_is_deterministic_across_repeated_encodes() {
    let bundle = build_bundle(&minimal_source(), 1, 1);
    let first = bundle.canonical_bytes(dictionary()).expect("canonicalizes");
    let second = bundle.canonical_bytes(dictionary()).expect("canonicalizes");
    assert_eq!(first, second);
}

#[test]
fn golden_vector_minimal_matches_committed_bytes() {
    const MINIMAL: &[u8] = include_bytes!("../tests/vectors/projection_bundle_v0/minimal.bin");
    let bytes = build_bundle(&minimal_source(), 1, 1)
        .canonical_bytes(dictionary())
        .expect("canonicalizes");
    assert_eq!(bytes, MINIMAL);
    verify_projection_bundle_v0(MINIMAL, compat(), limits()).expect("committed vector verifies");
}

#[test]
fn golden_vector_structured_matches_committed_bytes() {
    const STRUCTURED: &[u8] =
        include_bytes!("../tests/vectors/projection_bundle_v0/structured.bin");
    let bytes = build_bundle(&structured_source(), 3, 4)
        .canonical_bytes(dictionary())
        .expect("canonicalizes");
    assert_eq!(bytes, STRUCTURED);
    verify_projection_bundle_v0(STRUCTURED, compat(), limits()).expect("committed vector verifies");
}

// ---- Negative: truncation, malformed lengths, trailing bytes -------------

#[test]
fn rejects_truncated_at_last_byte() {
    let bytes = valid_bundle_bytes();
    let truncated = &bytes[..bytes.len() - 1];
    match verify_projection_bundle_v0(truncated, compat(), limits()) {
        Err(err) => assert!(matches!(
            err.kind(),
            Kind::TruncatedByteString | Kind::TruncatedPrimitive | Kind::TrailingBytes
        )),
        Ok(_) => panic!("truncated input must not verify"),
    }
}

#[test]
fn rejects_truncated_header() {
    let bytes = valid_bundle_bytes();
    let truncated = &bytes[..8];
    assert_rejected(truncated, Stage::Decode, Kind::TruncatedByteString);
}

#[test]
fn rejects_empty_input() {
    assert_rejected(&[], Stage::Decode, Kind::TruncatedPrimitive);
}

#[test]
fn rejects_trailing_bytes() {
    let mut bytes = valid_bundle_bytes();
    bytes.push(0xFF);
    assert_rejected(&bytes, Stage::CompleteConsumption, Kind::TrailingBytes);
}

#[test]
fn rejects_malformed_section_length_pointing_past_end_of_input() {
    let bytes = valid_bundle_bytes();
    // The first section's payload_length field is the u32 immediately
    // after the section tag byte, which follows the header (magic +
    // schema + contract + bundle_id + bundle_revision + bundle_epoch +
    // role_dictionary versions + section_count). Corrupting it to u32::MAX
    // must be rejected as a truncated byte-string, never accepted or
    // silently clamped.
    let header_len = 4 + PROJECTION_BUNDLE_V0_MAGIC.len() + 4 + 4 + 8 + 8 + 8 + 4 + 4 + 4;
    let length_field_offset = header_len + 1; // past the first section's tag byte
    let mut mutated = bytes.clone();
    mutated[length_field_offset..length_field_offset + 4].copy_from_slice(&u32::MAX.to_le_bytes());
    match verify_projection_bundle_v0(&mutated, compat(), limits()) {
        Err(err) => assert_eq!(err.kind(), Kind::TruncatedByteString),
        Ok(_) => panic!("malformed section length must not verify"),
    }
}

// ---- Negative: structural (missing/duplicate/unknown/noncanonical order) -

fn hand_encode_header(bundle_id: u64, revision: u64, epoch: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_bytes(&mut bytes, PROJECTION_BUNDLE_V0_MAGIC);
    push_u32(&mut bytes, 1);
    push_u32(&mut bytes, 1);
    push_u64(&mut bytes, bundle_id);
    push_u64(&mut bytes, revision);
    push_u64(&mut bytes, epoch);
    push_u32(&mut bytes, dictionary().schema_version().raw());
    push_u32(&mut bytes, dictionary().contract_version().raw());
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

fn extract_section_payloads(bytes: &[u8]) -> Vec<(u8, Vec<u8>)> {
    // Independently re-parses a valid encoded bundle's section table using
    // only length-prefix arithmetic, so mutation tests below can
    // reassemble a deliberately invalid section table without depending on
    // the module's own decoder.
    let mut offset = 0usize;
    let magic_len = u32::from_le_bytes(bytes[0..4].try_into().unwrap()) as usize;
    offset += 4 + magic_len;
    offset += 4 + 4 + 8 + 8 + 8 + 4 + 4; // schema, contract, id, rev, epoch, dict schema, dict contract
    let section_count = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
    offset += 4;
    let mut sections = Vec::with_capacity(section_count);
    for _ in 0..section_count {
        let tag = bytes[offset];
        offset += 1;
        let len = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let payload = bytes[offset..offset + len].to_vec();
        offset += len;
        sections.push((tag, payload));
    }
    assert_eq!(offset, bytes.len(), "test helper must consume all bytes");
    sections
}

fn reassemble(header: Vec<u8>, sections: &[(u8, Vec<u8>)]) -> Vec<u8> {
    let mut bytes = header;
    push_u32(&mut bytes, sections.len() as u32);
    for (tag, payload) in sections {
        bytes.push(*tag);
        push_bytes(&mut bytes, payload);
    }
    bytes
}

#[test]
fn rejects_missing_required_section() {
    let valid = valid_bundle_bytes();
    let sections = extract_section_payloads(&valid);
    let header = hand_encode_header(DOCUMENT_RAW, 1, 1);
    let truncated_sections: Vec<_> = sections[..sections.len() - 1].to_vec();
    let mutated = reassemble(header, &truncated_sections);
    assert_rejected(
        &mutated,
        Stage::Structural,
        Kind::MissingSection(ProjectionBundleSectionKind::StaticUiIr),
    );
}

#[test]
fn rejects_duplicate_section() {
    let valid = valid_bundle_bytes();
    let sections = extract_section_payloads(&valid);
    let header = hand_encode_header(DOCUMENT_RAW, 1, 1);
    let mut duplicated = sections[..sections.len() - 1].to_vec();
    duplicated.push(sections[0].clone()); // duplicate StaticUiIr in the final slot
    let mutated = reassemble(header, &duplicated);
    // The duplicate arrives out of ascending order (tag 0 in the position
    // expected to hold tag 5), so this is caught as noncanonical order
    // before the duplicate-tag check is reached -- both are Structural.
    match verify_projection_bundle_v0(&mutated, compat(), limits()) {
        Err(err) => assert_eq!(err.stage(), Stage::Structural),
        Ok(_) => panic!("duplicated section must not verify"),
    }
}

#[test]
fn rejects_unknown_section_kind() {
    let valid = valid_bundle_bytes();
    let mut sections = extract_section_payloads(&valid);
    sections[0].0 = 0xEE;
    let header = hand_encode_header(DOCUMENT_RAW, 1, 1);
    let mutated = reassemble(header, &sections);
    assert_rejected(&mutated, Stage::Structural, Kind::UnknownSectionKind);
}

#[test]
fn rejects_noncanonical_section_order() {
    let valid = valid_bundle_bytes();
    let mut sections = extract_section_payloads(&valid);
    sections.swap(0, 1);
    let header = hand_encode_header(DOCUMENT_RAW, 1, 1);
    let mutated = reassemble(header, &sections);
    assert_rejected(&mutated, Stage::Structural, Kind::NoncanonicalSectionOrder);
}

#[test]
fn rejects_binding_graph_referencing_nonexistent_node() {
    // A hand-crafted BindingGraph section payload declaring one binding
    // whose target node does not exist in the bundle's own Static UI IR:
    // id=1, node=999, slot=0, domain=Text(2), 0 dependencies, no
    // collection_key, no semantic_value, no source_ref.
    let mut malformed_binding_graph = Vec::new();
    push_u32(&mut malformed_binding_graph, 1); // one binding
    push_u64(&mut malformed_binding_graph, 1); // binding id
    push_u64(&mut malformed_binding_graph, 999); // nonexistent node
    push_u32(&mut malformed_binding_graph, 0); // slot
    malformed_binding_graph.push(2); // domain tag = Text
    push_u32(&mut malformed_binding_graph, 0); // 0 dependencies
    malformed_binding_graph.push(0); // no collection_key
    malformed_binding_graph.push(0); // no semantic_value
    malformed_binding_graph.push(0); // no source_ref

    let valid = valid_bundle_bytes();
    let mut sections = extract_section_payloads(&valid);
    sections[1].1 = malformed_binding_graph; // BindingGraph is tag 1
    let header = hand_encode_header(DOCUMENT_RAW, 1, 1);
    let mutated = reassemble(header, &sections);
    assert_rejected(&mutated, Stage::Structural, Kind::Structural);
}

// ---- Negative: cross-artifact, compatibility, resource, trust ------------

#[test]
fn rejects_collection_anchor_identity_mismatch() {
    let valid = valid_bundle_bytes();
    let mut sections = extract_section_payloads(&valid);
    // CollectionAnchors is tag 3: document_id(u64) revision(u64) epoch(u64) count anchors...
    let mut mutated_payload = sections[3].1.clone();
    // Corrupt the document_id field (first 8 bytes) to a value that
    // disagrees with the embedded Static UI IR's actual document id.
    mutated_payload[0..8].copy_from_slice(&999u64.to_le_bytes());
    sections[3].1 = mutated_payload;
    let header = hand_encode_header(DOCUMENT_RAW, 1, 1);
    let mutated = reassemble(header, &sections);
    assert_rejected(&mutated, Stage::CrossArtifact, Kind::CrossArtifactMismatch);
}

#[test]
fn rejects_collection_anchor_referencing_nonexistent_node() {
    let valid = valid_bundle_bytes();
    let mut sections = extract_section_payloads(&valid);
    let mut payload = Vec::new();
    push_u64(&mut payload, DOCUMENT_RAW);
    push_u64(&mut payload, 1);
    push_u64(&mut payload, 1);
    push_u32(&mut payload, 1);
    push_u64(&mut payload, 999); // nonexistent node
    sections[3].1 = payload;
    let header = hand_encode_header(DOCUMENT_RAW, 1, 1);
    let mutated = reassemble(header, &sections);
    assert_rejected(&mutated, Stage::CrossArtifact, Kind::CrossArtifactMismatch);
}

#[test]
fn rejects_unsupported_schema_version() {
    let bundle = build_bundle(&minimal_source(), 1, 1);
    let bytes = bundle.canonical_bytes(dictionary()).expect("canonicalizes");
    let sections = extract_section_payloads(&bytes);
    let mut header = hand_encode_header(DOCUMENT_RAW, 1, 1);
    // Overwrite the schema_version field (right after the length-prefixed
    // magic) with an unsupported value.
    let magic_len = u32::from_le_bytes(header[0..4].try_into().unwrap()) as usize;
    let schema_offset = 4 + magic_len;
    header[schema_offset..schema_offset + 4].copy_from_slice(&99u32.to_le_bytes());
    let mutated = reassemble(header, &sections);
    assert_rejected(&mutated, Stage::Header, Kind::UnsupportedSchemaVersion);
}

#[test]
fn rejects_incompatible_role_dictionary_version() {
    let valid = valid_bundle_bytes();
    let sections = extract_section_payloads(&valid);
    let mut header = hand_encode_header(DOCUMENT_RAW, 1, 1);
    let last = header.len();
    header[last - 4..last].copy_from_slice(&7u32.to_le_bytes()); // role dictionary contract version
    let mutated = reassemble(header, &sections);
    assert_rejected(
        &mutated,
        Stage::Compatibility,
        Kind::IncompatibleRoleDictionary,
    );
}

#[test]
fn rejects_input_exceeding_resource_limit() {
    let bytes = valid_bundle_bytes();
    let tight_limits = ProjectionBundleLimits {
        max_bundle_bytes: bytes.len() - 1,
        ..limits()
    };
    match verify_projection_bundle_v0(&bytes, compat(), tight_limits) {
        Err(err) => {
            assert_eq!(err.stage(), Stage::Resource);
            assert_eq!(err.kind(), Kind::InputTooLarge);
        }
        Ok(_) => panic!("oversized bundle must not verify"),
    }
}

#[test]
fn rejects_binding_count_exceeding_resource_limit() {
    let compiled = compile_projection_source_text_with_collection_anchors(
        source_id(),
        &minimal_source(),
        document_id(),
        dictionary(),
    )
    .expect("valid source");
    // Two distinct bindings targeting the same declared node/slot pair are
    // rejected by BindingGraphDocument::build itself as duplicate-slot, so
    // use two different slots on the same node to stay structurally valid
    // while exceeding a tight max_binding_count.
    let bindings = alloc::vec![
        crate::binding_graph::BindingDeclaration {
            id: crate::binding_graph::BindingId(1),
            target: crate::binding_graph::BindingTarget {
                node: crate::contract_primitives::StaticNodeId::new(11).unwrap(),
                slot: crate::binding_graph::BindingSlot(0),
            },
            domain: crate::binding_graph::BindingValueDomain::Text,
            dependencies: Vec::new(),
            collection_key: None,
            semantic_value: None,
            source_ref: None,
        },
        crate::binding_graph::BindingDeclaration {
            id: crate::binding_graph::BindingId(2),
            target: crate::binding_graph::BindingTarget {
                node: crate::contract_primitives::StaticNodeId::new(11).unwrap(),
                slot: crate::binding_graph::BindingSlot(1),
            },
            domain: crate::binding_graph::BindingValueDomain::Text,
            dependencies: Vec::new(),
            collection_key: None,
            semantic_value: None,
            source_ref: None,
        },
    ];
    let binding_graph =
        BindingGraphDocument::build(bindings, compiled.static_document()).expect("valid bindings");
    let action_ir = ActionIrDocument::build(Vec::new(), compiled.static_document())
        .expect("empty action IR is trivially valid");
    let bundle = ProjectionBundleV0::assemble(
        document_id(),
        Revision::new(1),
        Epoch::new(1),
        dictionary(),
        compiled.static_document().clone(),
        binding_graph,
        action_ir,
        compiled.collection_anchors().clone(),
        provenance(),
    );
    let bytes = bundle.canonical_bytes(dictionary()).expect("canonicalizes");
    let tight_limits = ProjectionBundleLimits {
        max_binding_count: 1,
        ..limits()
    };
    match verify_projection_bundle_v0(&bytes, compat(), tight_limits) {
        Err(err) => {
            assert_eq!(err.stage(), Stage::Resource);
            assert_eq!(err.kind(), Kind::ResourceLimitExceeded);
        }
        Ok(_) => panic!("binding count over limit must not verify"),
    }
}

#[test]
fn rejects_noncanonical_accessibility_ordering() {
    let valid = valid_bundle_bytes();
    let mut sections = extract_section_payloads(&valid);
    // Accessibility is tag 4. Build two entries in descending (noncanonical)
    // node-id order.
    let mut payload = Vec::new();
    push_u32(&mut payload, 2);
    push_u64(&mut payload, 12);
    payload.push(0);
    push_bytes(&mut payload, b"text");
    push_u64(&mut payload, 11);
    payload.push(0);
    push_bytes(&mut payload, b"root");
    sections[4].1 = payload;
    let header = hand_encode_header(DOCUMENT_RAW, 1, 1);
    let mutated = reassemble(header, &sections);
    match verify_projection_bundle_v0(&mutated, compat(), limits()) {
        Err(err) => assert_eq!(err.stage(), Stage::Structural),
        Ok(_) => panic!("noncanonical accessibility order must not verify"),
    }
}

#[test]
fn deterministic_diagnostic_ordering_same_input_same_error() {
    let valid = valid_bundle_bytes();
    let truncated = &valid[..8];
    let first = verify_projection_bundle_v0(truncated, compat(), limits());
    let second = verify_projection_bundle_v0(truncated, compat(), limits());
    assert_eq!(first, second);
}
