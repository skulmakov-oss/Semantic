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

#[test]
fn token_retains_metadata_presence_in_program_metadata() {
    let bytes =
        compile_program_to_semcode("fn main() { let x = \"hello\"; return; }").expect("compile");
    let token = verify_semcode_token(&bytes).expect("verify");
    let main_fn = &token.program().functions[0];

    assert!(main_fn.string_count > 0, "token must retain string count");
}

#[test]
fn raw_helper_behavior_unchanged_for_valid_bytes() {
    let bytes = compile_program_to_semcode("fn main() { return; }").expect("compile");
    sm_vm::run_semcode(&bytes).expect("raw run should succeed");
    sm_vm::run_semcode_with_entry(&bytes, "main").expect("raw run with entry should succeed");
}

#[test]
fn disasm_behavior_unchanged() {
    let bytes = compile_program_to_semcode("fn main() { return; }").expect("compile");
    let disasm = sm_vm::disasm_semcode(&bytes).expect("disasm should succeed");
    assert!(disasm.contains("fn main"));
}

#[test]
fn host_and_capability_path_parity() {
    use prom_abi::RecordingHostAbi;
    use prom_cap::CapabilityManifest;
    use sm_runtime_core::{ExecutionConfig, ExecutionContext};
    use sm_vm::run_verified_entry_semcode_with_host_and_capabilities_and_config;

    let bytes = compile_program_to_semcode("fn main() { return; }").expect("compile");
    let token = verify_semcode_token(&bytes).expect("verify");
    let entry_token = token.require_entry("main").expect("require entry");

    let mut host = RecordingHostAbi::default();
    let capabilities = CapabilityManifest::new();
    let config = ExecutionConfig::for_context(ExecutionContext::VerifiedLocal);

    run_verified_entry_semcode_with_host_and_capabilities_and_config(
        &entry_token,
        &mut host,
        &capabilities,
        config,
    )
    .expect("host and capability path should succeed");
}

#[test]
fn ui_capability_path_parity() {
    use prom_abi::RecordingHostAbi;
    use prom_cap::CapabilityManifest;
    use sm_vm::run_verified_entry_semcode_with_ui_capabilities;

    let bytes = compile_program_to_semcode("fn main() { return; }").expect("compile");
    let token = verify_semcode_token(&bytes).expect("verify");
    let entry_token = token.require_entry("main").expect("require entry");

    let mut host = RecordingHostAbi::default();
    let capabilities = CapabilityManifest::new();
    let ui_capabilities = CapabilityManifest::new();

    run_verified_entry_semcode_with_ui_capabilities(
        &entry_token,
        &mut host,
        &capabilities,
        &ui_capabilities,
    )
    .expect("UI capability path should succeed");
}
