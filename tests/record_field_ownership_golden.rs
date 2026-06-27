use sm_emit::compile_program_to_semcode;
use sm_format::semcode_decode::{decode_semcode_envelope, DecodedAccessPathComponent};

#[test]
fn positive_record_field_ownership_e2e_golden() {
    let src = include_str!("fixtures/pcc4_records/positive_record_field_ownership.sm");
    let bytes = compile_program_to_semcode(src).expect("compile");

    let (_, envelopes) = decode_semcode_envelope(&bytes).expect("decode envelopes");
    let main = envelopes
        .iter()
        .find(|envelope| envelope.name == "main")
        .expect("main envelope");

    let mut field_symbols = Vec::new();
    let mut saw_borrow_field = false;
    let mut saw_write_field = false;

    for path in &main.borrowed_paths {
        if path
            .components
            .iter()
            .any(|component| matches!(component, DecodedAccessPathComponent::FieldSymbol(_)))
        {
            saw_borrow_field = true;
        }
        for component in &path.components {
            if let DecodedAccessPathComponent::FieldSymbol(field) = component {
                field_symbols.push(*field);
            }
        }
    }

    for path in &main.write_paths {
        if path
            .components
            .iter()
            .any(|component| matches!(component, DecodedAccessPathComponent::FieldSymbol(_)))
        {
            saw_write_field = true;
        }
        for component in &path.components {
            if let DecodedAccessPathComponent::FieldSymbol(field) = component {
                field_symbols.push(*field);
            }
        }
    }

    field_symbols.sort_unstable();
    field_symbols.dedup();

    assert!(
        field_symbols.len() >= 2,
        "expected at least two distinct record field symbols, found {:?}",
        field_symbols
    );
    assert!(
        saw_borrow_field,
        "expected at least one borrowed field path"
    );
    assert!(saw_write_field, "expected at least one write field path");

    let token = sm_verify::verify_semcode_token(&bytes).expect("token admission");
    let entry_token = token.require_entry("main").expect("entry resolution");
    sm_vm::run_verified_entry_semcode(&entry_token).expect("vm run");
}
