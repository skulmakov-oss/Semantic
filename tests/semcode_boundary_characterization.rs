use sm_emit::compile_program_to_semcode;
use sm_verify::{verify_semcode, verify_semcode_token, VerificationCode};
use sm_vm::{run_verified_entry_semcode, run_verified_semcode, run_verified_semcode_with_entry};

#[test]
fn raw_helper_current_behavior_short_and_unsupported_headers_are_rejected_consistently() {
    let bytes = vec![0u8; 4];
    let err = verify_semcode(&bytes).expect_err("expected rejection for short header");
    assert!(err
        .diagnostics
        .iter()
        .any(|d| d.code == VerificationCode::BadHeader));

    let mut bad_magic = b"BADC0DE0".to_vec();
    bad_magic.extend(vec![0u8; 100]);
    let err = verify_semcode(&bad_magic).expect_err("expected rejection for unsupported header");
    assert!(err
        .diagnostics
        .iter()
        .any(|d| d.code == VerificationCode::UnsupportedVersion));
}

#[test]
fn duplicate_function_names_are_rejected_by_verifier() {
    let mut bytes =
        compile_program_to_semcode("fn main() { return; } fn maib() { return; }").expect("compile");

    // Find "maib" and replace it with "main" to create a duplicate function
    let target = b"maib";
    let replacement = b"main";
    let mut found = false;
    for i in 0..bytes.len() - target.len() {
        if &bytes[i..i + target.len()] == target {
            bytes[i..i + target.len()].copy_from_slice(replacement);
            found = true;
            break;
        }
    }
    assert!(found, "could not find maib to replace");

    let err = verify_semcode(&bytes).expect_err("expected verifier to reject duplicate function");
    assert!(
        err.diagnostics
            .iter()
            .any(|d| d.code == VerificationCode::DuplicateFunction),
        "expected DuplicateFunction code, got {:?}",
        err.diagnostics
    );
}

#[test]
fn missing_main_entrypoint_current_behavior_is_verifier_ok_and_explicit_entry_ok() {
    let mut bytes = compile_program_to_semcode("fn main() { return; }").expect("compile");

    // Rename 'main' to 'help' to remove main entrypoint
    let target = b"main";
    let replacement = b"help";
    let mut found = false;
    for i in 0..bytes.len() - target.len() {
        if &bytes[i..i + target.len()] == target {
            bytes[i..i + target.len()].copy_from_slice(replacement);
            found = true;
            break;
        }
    }
    assert!(found, "could not find main to replace");

    // Verifier is OK with it
    verify_semcode(&bytes).expect("verifier should accept missing main");

    // Default VM run fails at lookup
    let err = run_verified_semcode(&bytes).expect_err("should fail to find main");
    assert!(
        err.to_string().contains("unknown function 'main'"),
        "actual error: {}",
        err
    );

    // Explicit entry lookup succeeds
    run_verified_semcode_with_entry(&bytes, "help").expect("should run explicitly");
}

#[test]
fn shim_and_token_path_produce_equivalent_success() {
    let bytes = compile_program_to_semcode("fn main() { return; }").expect("compile");

    // 1. Compatibility verified shim
    run_verified_semcode(&bytes).expect("shim should succeed");

    // 2. Canonical token path
    let token = verify_semcode_token(&bytes).expect("verify");
    let entry_token = token.require_entry("main").expect("require entry");
    run_verified_entry_semcode(&entry_token).expect("token path should succeed");
}
