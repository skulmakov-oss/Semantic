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

    let artifact = temp_semcode_artifact("pcc6-result", "smc_pcc6_result_acceptance");
    compile_source_to_artifact(&source, &artifact);
    verify_artifact(&artifact);
}

#[test]
fn pcc6_result_ok_match_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc6_result/positive_result_ok_match.sm");
}

#[test]
fn pcc6_result_err_match_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc6_result/positive_result_err_match.sm");
}

#[test]
fn pcc6_result_function_boundary_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc6_result/positive_result_function_boundary.sm");
}
