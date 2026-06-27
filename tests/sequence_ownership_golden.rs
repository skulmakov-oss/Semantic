use sm_emit::compile_program_to_semcode;
use sm_format::semcode_decode::{decode_semcode_envelope, DecodedAccessPathComponent};
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
