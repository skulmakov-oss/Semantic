#[path = "support/cli_artifact_support.rs"]
mod cli_artifact_support;

use cli_artifact_support::{
    check_source, compile_source_to_artifact, run_source, source_fixture, temp_semcode_artifact,
    verify_artifact,
};

fn check_run_compile_verify(rel: &str) {
    let source = source_fixture(rel);
    check_source(&source);
    run_source(&source);

    let artifact = temp_semcode_artifact("pcc8-stdlib", "smc_pcc8_stdlib_acceptance");
    compile_source_to_artifact(&source, &artifact);
    verify_artifact(&artifact);
}

#[test]
fn pcc8_assert_true_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc8_stdlib/positive_assert_true.sm");
}

#[test]
fn pcc8_print_text_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc8_stdlib/positive_print_text.sm");
}

#[test]
fn pcc8_to_text_basic_types_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc8_stdlib/positive_to_text_basic_types.sm");
}

#[test]
fn pcc8_text_helpers_basic_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc8_stdlib/positive_text_helpers_basic.sm");
}
