use sm_emit::compile_program_to_semcode;
use sm_format::semcode_decode::{decode_semcode_envelope, DecodedAccessPathComponent};

#[test]
fn positive_option_result_ownership_e2e_golden() {
    let src =
        include_str!("fixtures/pcc6_option_result_diagnostics/positive_option_result_ownership.sm");
    let bytes = compile_program_to_semcode(src).expect("compile");

    // Decode semcode to assert OWN0 contains AdtPayload
    let (_, envelopes) = decode_semcode_envelope(&bytes).expect("decode envelopes");

    let mut adt_payload_count = 0;
    for envelope in envelopes {
        for path in envelope
            .borrowed_paths
            .iter()
            .chain(envelope.write_paths.iter())
        {
            for comp in &path.components {
                if matches!(comp, DecodedAccessPathComponent::AdtPayload { .. }) {
                    adt_payload_count += 1;
                }
            }
        }
    }

    // read_option (Option::Some), read_result_ok (Result::Ok), read_result_err (Result::Err)
    assert!(
        adt_payload_count >= 3,
        "Expected at least 3 AdtPayload components, found {}",
        adt_payload_count
    );

    // Verify SemCode
    let token = sm_verify::verify_semcode_token(&bytes).expect("token admission");
    let entry_token = token.require_entry("main").expect("entry resolution");

    // Run VM
    sm_vm::run_verified_entry_semcode(&entry_token).expect("vm run");
}
