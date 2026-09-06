use sm_emit::compile_program_to_semcode;
use sm_format::semcode_decode::{decode_semcode_envelope, DecodedAccessPathComponent};
use sm_format::semcode_format::{MAGIC20, MAGIC21};
use sm_verify::verify_semcode_token;
use sm_vm::run_verified_entry_semcode;

#[test]
fn positive_sequence_ownership_e2e_golden() {
    let src = include_str!("fixtures/pcc_sequence_ownership/positive_sequence_ownership.sm");
    let bytes = compile_program_to_semcode(src).expect("compile");

    let (_, envelopes) = decode_semcode_envelope(&bytes).expect("decode envelopes");
    let main = envelopes
        .iter()
        .find(|envelope| envelope.name == "main")
        .expect("main envelope");

    let mut seq_indexes = Vec::new();
    let mut saw_nested_path = false;

    for path in &main.borrowed_paths {
        let Some(first) = path.components.first() else {
            panic!("expected sequence ownership component");
        };
        match first {
            DecodedAccessPathComponent::SequenceIndexStatic(index) => {
                seq_indexes.push(*index);
            }
            other => panic!("expected top-level SequenceIndexStatic component, found {other:?}"),
        }
        if path.components.len() > 1 {
            saw_nested_path = true;
        }
    }

    for path in &main.write_paths {
        let Some(first) = path.components.first() else {
            panic!("expected sequence ownership component");
        };
        match first {
            DecodedAccessPathComponent::SequenceIndexStatic(index) => {
                seq_indexes.push(*index);
            }
            other => panic!("expected top-level SequenceIndexStatic component, found {other:?}"),
        }
        if path.components.len() > 1 {
            saw_nested_path = true;
        }
    }

    seq_indexes.sort_unstable();
    seq_indexes.dedup();

    assert!(seq_indexes.contains(&0), "expected seq[0] ownership path");
    assert!(seq_indexes.contains(&1), "expected seq[1] ownership path");
    assert!(
        saw_nested_path,
        "expected nested tuple payload components after sequence index"
    );

    let token = verify_semcode_token(&bytes).expect("token admission");
    let entry_token = token.require_entry("main").expect("entry resolution");
    run_verified_entry_semcode(&entry_token).expect("vm run");
}

// #1718 downgrade/re-encode audit (item 13): a real `HEADER_V21` artifact
// containing `SequenceIndexStatic` ownership paths must not become
// admissible again merely by relabeling its header magic down to
// `HEADER_V20` - the two headers share the identical OWN0 wire grammar
// (both are at/above `SEMCODE_OWNERSHIP_ANCHOR_MIN_REVISION`, so no byte
// other than the 8-byte magic prefix needs to change for this to be a
// structurally well-formed, if dishonestly labeled, `HEADER_V20` artifact),
// which is exactly the scenario a content-sniffing or "if bytes decode,
// admit them" implementation would get wrong. The real gate is
// `header_rev`-derived, not inferred from OWN0 content, so relabeling alone
// must still fail closed.
#[test]
fn v21_sequence_artifact_cannot_be_relabeled_to_v20_and_still_verify() {
    let src = include_str!("fixtures/pcc_sequence_ownership/positive_sequence_ownership.sm");
    let bytes = compile_program_to_semcode(src).expect("compile");
    assert_eq!(
        &bytes[..8],
        &MAGIC21,
        "this fixture must genuinely require HEADER_V21 for this audit to be meaningful"
    );

    let mut downgraded = bytes.clone();
    downgraded[..8].copy_from_slice(&MAGIC20);

    let err = verify_semcode_token(&downgraded)
        .expect_err("a relabeled-to-V20 Sequence artifact must not verify");
    assert!(err
        .diagnostics
        .iter()
        .any(|d| d.code == sm_verify::VerificationCode::InvalidOwnershipSection));
}
