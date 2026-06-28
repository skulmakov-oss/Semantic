use sm_emit::compile_program_to_semcode;
use sm_format::semcode_decode::{decode_semcode_envelope, DecodedAccessPathComponent};
use sm_verify::verify_semcode_token;
use sm_vm::run_verified_entry_semcode;

#[test]
fn positive_tuple_ownership_e2e_golden() {
    let src = include_str!("fixtures/pcc_tuple_ownership/positive_tuple_ownership.sm");
    let bytes = compile_program_to_semcode(src).expect("compile");

    let (_, envelopes) = decode_semcode_envelope(&bytes).expect("decode envelopes");
    let main = envelopes
        .iter()
        .find(|envelope| envelope.name == "main")
        .expect("main envelope");

    let mut tuple_indexes = Vec::new();
    let mut saw_borrow_tuple = false;

    for path in &main.borrowed_paths {
        for component in &path.components {
            if let DecodedAccessPathComponent::TupleIndex(index) = component {
                tuple_indexes.push(*index);
                saw_borrow_tuple = true;
            }
        }
    }

    for path in &main.write_paths {
        for component in &path.components {
            if let DecodedAccessPathComponent::TupleIndex(index) = component {
                tuple_indexes.push(*index);
            }
        }
    }

    tuple_indexes.sort_unstable();
    tuple_indexes.dedup();

    assert!(
        tuple_indexes.contains(&0) && tuple_indexes.contains(&1),
        "expected tuple index ownership components 0 and 1, found {:?}",
        tuple_indexes
    );
    assert!(
        saw_borrow_tuple,
        "expected at least one borrowed tuple path"
    );

    let token = verify_semcode_token(&bytes).expect("token admission");
    let entry_token = token.require_entry("main").expect("entry resolution");
    run_verified_entry_semcode(&entry_token).expect("vm run");
}
