//! Q3.10 B2-1 independent structural, adversarial and fail-closed qualification.
//! No test in this file treats structural success as trusted evidence.

use sm_format::semcode_decode::{decode_semcode, DecodeError};
use sm_format::semcode_format::{MAGIC25, MAGIC26, SEMCODE_EVR0_MIN_REVISION};
use sm_verify::{verify_semcode_token, VerificationCode};

const FIXTURE: &[u8] = include_bytes!("vectors/q3_10_evr0_structural_only.smc");
const SECTION: &[u8] = include_bytes!("vectors/q3_10_evr0_section.bin");
const SECTION_AT: usize = FIXTURE.len() - SECTION.len();
const PAYLOAD_AT: usize = SECTION_AT + 8;

fn valid() -> Vec<u8> {
    FIXTURE.to_vec()
}
fn invalid(bytes: &[u8]) {
    assert!(
        decode_semcode(bytes).is_err(),
        "malformed EVR0 decoded successfully"
    );
}
fn write_u32(bytes: &mut [u8], payload_offset: usize, value: u32) {
    bytes[PAYLOAD_AT + payload_offset..PAYLOAD_AT + payload_offset + 4]
        .copy_from_slice(&value.to_le_bytes());
}

#[test]
fn byte_golden_is_complete_and_structurally_only() {
    assert_eq!(&FIXTURE[..8], &MAGIC26);
    let canonical_adt = sm_format::semcode_format::AdtDescriptorTable::with_builtins(vec![])
        .expect("canonical table")
        .encode_section();
    assert_eq!(&FIXTURE[8..SECTION_AT], canonical_adt);
    assert_eq!(&FIXTURE[SECTION_AT..], SECTION);
    assert_eq!(SECTION.len(), 249);
    let decoded = decode_semcode(FIXTURE).expect("bounded canonical structural vector");
    assert_eq!(decoded.header.rev, SEMCODE_EVR0_MIN_REVISION);
    assert!(decoded.adt_descriptors.is_some());
    assert!(decoded.functions.is_empty());
    let evr0 = decoded.evr0.expect("mandatory rev27 evidence section");
    assert_eq!(evr0.propositions.len(), 1);
    assert_eq!(evr0.proofs.len(), 1);
    assert_eq!(evr0.witnesses.len(), 1);
    assert_eq!(evr0.parents.len(), 0);
    assert_eq!(evr0.proofs[0].rule_id, "POS-WITNESS-1");
    assert_eq!(evr0.issuer_namespace, "i");
    let report = verify_semcode_token(FIXTURE).expect_err("structural data cannot mint a token");
    assert_eq!(
        report.diagnostics[0].code,
        VerificationCode::EvidenceIssuerUnavailable
    );
    assert_eq!(report.diagnostics[0].code.code_token(), "V0034");
    let err = sm_vm::run_semcode(FIXTURE).expect_err("raw VM must reject rev27");
    assert!(err.to_string().contains("structural-only"), "{err:?}");
}

#[test]
fn missing_truncated_and_unsupported_revisions_are_rejected() {
    let mut missing = valid();
    missing.truncate(SECTION_AT);
    assert!(matches!(
        decode_semcode(&missing),
        Err(DecodeError::InvalidEvidenceSection { .. })
    ));
    let mut wrong_tag = valid();
    wrong_tag[SECTION_AT] ^= 1;
    invalid(&wrong_tag);
    let mut truncated = valid();
    truncated.pop();
    invalid(&truncated);
    let mut unknown = valid();
    unknown[0..8].copy_from_slice(b"SEMCOD27");
    assert!(matches!(
        decode_semcode(&unknown),
        Err(DecodeError::UnsupportedVersion { .. })
    ));
    let mut old = valid();
    old[0..8].copy_from_slice(&MAGIC25);
    // Old revision never sniffs the EVR0 section as proof. It must not mint
    // anything resembling an evidence token.
    assert!(decode_semcode(&old).is_err());
}

#[test]
fn section_length_and_counts_are_bounded() {
    let mut empty = valid();
    empty[SECTION_AT + 4..SECTION_AT + 8].copy_from_slice(&0u32.to_le_bytes());
    invalid(&empty);
    let mut oversized = valid();
    oversized[SECTION_AT + 4..SECTION_AT + 8].copy_from_slice(&4_194_305u32.to_le_bytes());
    assert!(matches!(
        decode_semcode(&oversized),
        Err(DecodeError::ResourceLimit { .. })
    ));
    let mut bad_count = valid();
    write_u32(&mut bad_count, 76, 65_537);
    assert!(matches!(
        decode_semcode(&bad_count),
        Err(DecodeError::ResourceLimit { .. })
    ));
    let mut impossible = valid();
    write_u32(&mut impossible, 80, 50_000);
    invalid(&impossible);
    let mut overflowed_length = valid();
    overflowed_length[SECTION_AT + 4..SECTION_AT + 8].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(matches!(
        decode_semcode(&overflowed_length),
        Err(DecodeError::ResourceLimit { .. })
    ));
}

#[test]
fn malformed_schema_text_flags_identity_and_reserved_reject() {
    let mut schema = valid();
    schema[PAYLOAD_AT] = 2;
    invalid(&schema);
    let mut flags = valid();
    flags[PAYLOAD_AT + 2] = 1;
    invalid(&flags);
    let mut utf8 = valid();
    utf8[PAYLOAD_AT + 72] = 0xff; // policy namespace text
    invalid(&utf8);
    let mut wrong_proof_id = valid();
    write_u32(&mut wrong_proof_id, 110, 1);
    invalid(&wrong_proof_id);
    let mut wrong_proposition = valid();
    write_u32(&mut wrong_proposition, 114, 1);
    invalid(&wrong_proposition);
    let mut bad_rule = valid();
    bad_rule[PAYLOAD_AT + 120] = b'X';
    invalid(&bad_rule);
    let mut bad_reserved = valid();
    bad_reserved[PAYLOAD_AT + 166] = 1; // witness reserved bytes
    invalid(&bad_reserved);
    let mut bad_span = valid();
    write_u32(&mut bad_span, 201, 10);
    write_u32(&mut bad_span, 205, 2);
    invalid(&bad_span);
    let mut wrong_polarity = valid();
    wrong_polarity[PAYLOAD_AT + 165] = 0;
    invalid(&wrong_polarity);
    let mut bad_trailing = valid();
    bad_trailing[SECTION_AT + 4..SECTION_AT + 8].copy_from_slice(&242u32.to_le_bytes());
    bad_trailing.push(0);
    invalid(&bad_trailing);
}

fn text(out: &mut Vec<u8>, data: &str) {
    out.extend_from_slice(&(data.len() as u16).to_le_bytes());
    out.extend_from_slice(data.as_bytes());
}
fn u32le(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

/// A canonical proof chain with forward-only parent references. Its semantic
/// authority is NOT checked by B2-1 and verification always rejects it.
fn chain_fixture(proof_count: u32) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&1u16.to_le_bytes()); // schema
    payload.extend_from_slice(&0u16.to_le_bytes()); // flags
    payload.extend_from_slice(&[0u8; 64]); // source set and compiler contract
    payload.extend_from_slice(&0u16.to_le_bytes());
    text(&mut payload, "p");
    text(&mut payload, "i");
    for count in [1, proof_count, 1, proof_count - 1] {
        u32le(&mut payload, count);
    }
    text(&mut payload, "type");
    for value in [1, 2, 3] {
        u32le(&mut payload, value);
    }
    for id in 0..proof_count {
        u32le(&mut payload, id);
        u32le(&mut payload, 0); // same proposition
        text(
            &mut payload,
            if id == 0 {
                "POS-WITNESS-1"
            } else {
                "OP-T-ONLY-1"
            },
        );
        u32le(&mut payload, 0); // authority context
        u32le(&mut payload, if id == 0 { 0 } else { 1 });
        u32le(&mut payload, if id == 0 { 1 } else { 0 });
        u32le(&mut payload, if id == 0 { 0 } else { id - 1 });
        u32le(&mut payload, if id == 0 { 0 } else { 1 });
        payload.push(if id == 0 { 0 } else { 1 });
        payload.extend_from_slice(&[0; 3]);
    }
    u32le(&mut payload, 0);
    u32le(&mut payload, 0);
    payload.extend_from_slice(&[1, 0, 0, 0]); // positive witness, zero reserved
    payload.extend_from_slice(&[0; 32]);
    u32le(&mut payload, 0);
    u32le(&mut payload, 2);
    payload.extend_from_slice(&[0; 32]);
    for parent in 0..(proof_count - 1) {
        u32le(&mut payload, parent);
    }
    let mut artifact = b"SEMCOD26".to_vec();
    artifact.extend_from_slice(
        &sm_format::semcode_format::AdtDescriptorTable::with_builtins(vec![])
            .expect("canonical ADT0")
            .encode_section(),
    );
    artifact.extend_from_slice(b"EVR0");
    artifact.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    artifact.extend_from_slice(&payload);
    artifact
}

#[test]
fn graph_depth_duplicate_forward_and_parent_identity_reject() {
    let safe = chain_fixture(256);
    let structural = decode_semcode(&safe).expect("256-depth chain is structurally valid");
    assert_eq!(structural.evr0.unwrap().proofs.len(), 256);
    let too_deep = chain_fixture(257);
    assert!(matches!(
        decode_semcode(&too_deep),
        Err(DecodeError::ResourceLimit { .. })
    ));
    let mut self_parent = chain_fixture(2);
    // last u32 is first proof's parent (0); self-parent of proof 1 is 1.
    let n = self_parent.len();
    self_parent[n - 4..].copy_from_slice(&1u32.to_le_bytes());
    invalid(&self_parent);
    let mut out_of_range = chain_fixture(2);
    out_of_range[n - 4..].copy_from_slice(&99u32.to_le_bytes());
    invalid(&out_of_range);
}

#[test]
fn every_truncated_prefix_and_single_byte_mutation_is_panic_free() {
    for cut in 0..FIXTURE.len() {
        let prefix = &FIXTURE[..cut];
        let result = std::panic::catch_unwind(|| decode_semcode(prefix));
        assert!(result.is_ok(), "decoder panicked on prefix of {cut} bytes");
        assert!(
            result.unwrap().is_err(),
            "truncated prefix admitted at {cut}"
        );
    }
    for offset in 0..FIXTURE.len() {
        let mut modified = valid();
        modified[offset] ^= 0x80;
        let result = std::panic::catch_unwind(|| decode_semcode(&modified));
        let decoded = result.expect("hostile byte mutation must not panic");
        if decoded.is_ok() {
            let report = verify_semcode_token(&modified)
                .expect_err("corruption must never mint a trusted token");
            assert_eq!(
                report.diagnostics[0].code,
                VerificationCode::EvidenceIssuerUnavailable
            );
        }
    }
}
