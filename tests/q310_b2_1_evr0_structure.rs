//! Structural EVR0 qualification only. These bytes do NOT authenticate an issuer.
use sm_format::semcode_decode::{decode_semcode, DecodeError};
use sm_format::semcode_format::{HEADER_V26, MAGIC25, MAGIC26};
use sm_ir::compile_program_to_semcode;
use sm_verify::{verify_semcode_token, VerificationCode};

const PROGRAM: &str = "fn main() { return; }\n";

/// Independent 249-byte test vector from the candidate specification.
/// The checksum and counts are checked; this is not an executable evidence proof.
fn evr0_section() -> Vec<u8> {
    let text = include_str!("../docs/spec/evr0_proof_section_v0.md");
    let mut out = Vec::new();
    for line in text.lines() {
        let Some((offset, data)) = line.split_once(": ") else {
            continue;
        };
        if offset.len() != 3 || !offset.bytes().all(|v| v.is_ascii_digit()) {
            continue;
        }
        let Ok(at) = offset.parse::<usize>() else {
            continue;
        };
        if at != out.len() {
            panic!("noncanonical EVR0 golden offset: {at} vs {}", out.len());
        }
        for s in data.split_whitespace() {
            out.push(u8::from_str_radix(s, 16).expect("golden hex"));
        }
    }
    assert_eq!(out.len(), 249);
    assert_eq!(out[0..4], *b"EVR0");
    assert_eq!(
        sm_format::sha256::format_hex(&sm_format::sha256::sha256(&out)),
        "cd91d079b4c5846f8bd4e9edf13fc2136d70fe935d808f6d17fdd1f318bd1870"
    );
    out
}

fn base_artifact() -> Vec<u8> {
    compile_program_to_semcode(PROGRAM).expect("baseline V22+ code")
}

fn artifact(section: &[u8]) -> Vec<u8> {
    let mut bytes = base_artifact();
    let adt_len = decode_semcode(&bytes)
        .unwrap()
        .adt_descriptors
        .unwrap()
        .encode_section()
        .len();
    let mut out = Vec::new();
    out.extend_from_slice(&MAGIC26);
    out.extend_from_slice(&bytes[8..8 + adt_len]);
    out.extend_from_slice(section);
    out.extend_from_slice(&bytes[8 + adt_len..]);
    bytes.clear();
    out
}

fn evidence_error(bytes: &[u8]) -> &'static str {
    match decode_semcode(bytes) {
        Err(DecodeError::InvalidEvidenceSection { msg, .. }) => msg,
        other => panic!("expected invalid EVR0, got {other:?}"),
    }
}

#[test]
fn b2_1_structural_golden_is_not_verifier_admission() {
    let legacy = base_artifact();
    assert!(decode_semcode(&legacy).unwrap().evidence.is_none());
    verify_semcode_token(&legacy).expect("legacy admitted");

    let section = evr0_section();
    let bytecode = artifact(&section);
    let parsed = decode_semcode(&bytecode).expect("structural decode");
    assert_eq!(parsed.header, HEADER_V26);
    let metadata = parsed.evidence.expect("mandatory EVR0");
    assert_eq!(metadata.payload_len, 241);
    assert_eq!(
        (
            metadata.proposition_count,
            metadata.proof_count,
            metadata.witness_count,
            metadata.parent_count
        ),
        (1, 1, 1, 0)
    );
    assert_eq!(metadata.policy_namespace, "p");
    assert_eq!(metadata.issuer_namespace, "i");
    // Fake fixture issuer/compiler_contract must never become positive Evidence.
    let rejected = verify_semcode_token(&bytecode).expect_err("issuer unavailable");
    assert_eq!(
        rejected.diagnostics[0].code,
        VerificationCode::UnsupportedVersion
    );
    assert!(
        matches!(
            sm_vm::run_semcode(&bytecode),
            Err(sm_vm::RuntimeError::UnsupportedBytecodeVersion { .. })
        ),
        "raw VM must reject structural-only rev27"
    );
}

#[test]
fn b2_1_rejects_missing_invalid_duplicate_or_truncated_evr0() {
    let section = evr0_section();
    let mut without = section.clone();
    without.clear();
    assert_eq!(
        evidence_error(&artifact(&without)),
        "missing mandatory EVR0 tag"
    );
    let mut invalid_tag = section.clone();
    invalid_tag[0] = b'X';
    assert_eq!(
        evidence_error(&artifact(&invalid_tag)),
        "missing mandatory EVR0 tag"
    );

    // Actual EOF inside section: no following function bytes to be consumed.
    let complete = artifact(&section);
    let section_start = complete.len()
        - section.len()
        - (base_artifact().len()
            - 8
            - decode_semcode(&base_artifact())
                .unwrap()
                .adt_descriptors
                .unwrap()
                .encode_section()
                .len());
    let physically_truncated = &complete[..section_start + section.len() - 1];
    assert_eq!(
        evidence_error(physically_truncated),
        "EVR0 payload truncated"
    );

    // If function bytes follow a short section, the complete SemCode still
    // rejects (function framing is shifted), but not necessarily as EVR0 EOF.
    let mut short_section = section.clone();
    short_section.pop();
    assert!(decode_semcode(&artifact(&short_section)).is_err());
    let mut huge = section.clone();
    huge[4..8].copy_from_slice(&4_194_305u32.to_le_bytes());
    assert_eq!(
        evidence_error(&artifact(&huge)),
        "EVR0 payload exceeds limit or is empty"
    );

    let mut duplicated = section.clone();
    duplicated.extend_from_slice(&section);
    assert_eq!(
        evidence_error(&artifact(&duplicated)),
        "duplicate EVR0 section"
    );
}

#[test]
fn b2_1_rejects_noncanonical_fields_and_hostile_counts() {
    let section = evr0_section();

    let mut flags = section.clone();
    flags[10] = 1;
    assert_eq!(
        evidence_error(&artifact(&flags)),
        "unsupported EVR0 schema or flags"
    );

    let mut unknown = section.clone();
    unknown[80] = 0xff; // policy namespace UTF-8, not trusted
    assert_eq!(
        evidence_error(&artifact(&unknown)),
        "invalid evidence UTF-8"
    );

    let mut count = section.clone();
    count[84..88].copy_from_slice(&65_537u32.to_le_bytes());
    assert_eq!(
        evidence_error(&artifact(&count)),
        "EVR0 table count exceeds limit"
    );

    let mut witness = section.clone();
    witness[174] = 1; // witness reserved byte
    assert_eq!(
        evidence_error(&artifact(&witness)),
        "nonzero evidence reserved bytes"
    );

    let mut wrong_identity = section.clone();
    wrong_identity[8 + 157 + 4..8 + 157 + 8].copy_from_slice(&1u32.to_le_bytes());
    assert_eq!(
        evidence_error(&artifact(&wrong_identity)),
        "EVR0 witness proposition out of bounds"
    );

    let mut backwards = section.clone();
    backwards[8 + 157 + 4 + 4 + 1 + 3 + 32..8 + 157 + 4 + 4 + 1 + 3 + 32 + 4]
        .copy_from_slice(&3u32.to_le_bytes());
    assert_eq!(
        evidence_error(&artifact(&backwards)),
        "EVR0 reversed witness span"
    );
}

#[test]
fn b2_1_legacy_downgrade_does_not_gain_evidence_authority() {
    let section = evr0_section();
    let mut bytes = artifact(&section);
    assert!(decode_semcode(&bytes).unwrap().evidence.is_some());
    bytes[0..8].copy_from_slice(&MAGIC25);
    // No content sniffing: post-ADT0 EVR0 bytes cannot be upgraded to trusted proof.
    assert!(decode_semcode(&bytes).is_err());
    assert!(verify_semcode_token(&bytes).is_err());
}

#[test]
fn b2_1_rejects_self_parent_cycle_and_noncanonical_ownership() {
    let mut section = evr0_section();
    // The original one-proof fixture has no parents. Extend the section by one
    // parent reference to its own proof, with internally consistent lengths.
    section[4..8].copy_from_slice(&245u32.to_le_bytes());
    section[96..100].copy_from_slice(&1u32.to_le_bytes()); // global parent count
    section[157..161].copy_from_slice(&1u32.to_le_bytes()); // proof parent count
    section.extend_from_slice(&0u32.to_le_bytes()); // self-reference
    assert_eq!(
        evidence_error(&artifact(&section)),
        "EVR0 invalid parent ordering or cycle"
    );

    let mut wrong_range = evr0_section();
    wrong_range[145..149].copy_from_slice(&1u32.to_le_bytes());
    assert_eq!(
        evidence_error(&artifact(&wrong_range)),
        "EVR0 noncanonical proof table ranges"
    );

    let mut witness_id = evr0_section();
    witness_id[165..169].copy_from_slice(&1u32.to_le_bytes());
    assert_eq!(
        evidence_error(&artifact(&witness_id)),
        "EVR0 witness IDs must be contiguous"
    );
}

#[test]
fn b2_1_adversarial_mutations_cannot_panic_decoder() {
    let section = evr0_section();
    for index in 0..section.len() {
        let mut mutation = section.clone();
        mutation[index] ^= 0xff;
        let bytes = artifact(&mutation);
        assert!(
            std::panic::catch_unwind(|| decode_semcode(&bytes)).is_ok(),
            "decoder panicked for byte offset {index}"
        );
    }
}

fn chain_evr0(depth: usize) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&1u16.to_le_bytes()); // schema
    payload.extend_from_slice(&0u16.to_le_bytes()); // flags
    payload.extend_from_slice(&[0; 64]); // source/contract digests (untrusted)
    payload.extend_from_slice(&0u16.to_le_bytes()); // epoch
    for namespace in [b"p", b"i"] {
        payload.extend_from_slice(&1u16.to_le_bytes());
        payload.extend_from_slice(namespace);
    }
    for count in [1usize, depth, 0, depth.saturating_sub(1)] {
        payload.extend_from_slice(&(count as u32).to_le_bytes());
    }
    payload.extend_from_slice(&1u16.to_le_bytes()); // proposition kind len
    payload.push(b'p');
    payload.extend_from_slice(&[0; 12]); // whole-program identity
    for i in 0..depth {
        payload.extend_from_slice(&(i as u32).to_le_bytes()); // proof ID
        payload.extend_from_slice(&0u32.to_le_bytes()); // proposition index
        payload.extend_from_slice(&1u16.to_le_bytes()); // rule ID len
        payload.push(b'R');
        payload.extend_from_slice(&0u32.to_le_bytes()); // authority
        payload.extend_from_slice(&0u32.to_le_bytes()); // witness_first
        payload.extend_from_slice(&0u32.to_le_bytes()); // witness_count
        payload.extend_from_slice(&(i.saturating_sub(1) as u32).to_le_bytes());
        payload.extend_from_slice(&(if i == 0 { 0u32 } else { 1 }).to_le_bytes());
        payload.extend_from_slice(&[0; 4]); // operation and reserved
    }
    for i in 1..depth {
        payload.extend_from_slice(&((i - 1) as u32).to_le_bytes());
    }
    let mut section = b"EVR0".to_vec();
    section.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    section.extend_from_slice(&payload);
    section
}

#[test]
fn b2_1_depth_budget_accepts_256_rejects_257() {
    let maximum = artifact(&chain_evr0(256));
    assert_eq!(
        decode_semcode(&maximum)
            .unwrap()
            .evidence
            .unwrap()
            .proof_count,
        256
    );
    assert_eq!(
        evidence_error(&artifact(&chain_evr0(257))),
        "EVR0 proof depth exceeded"
    );
}
