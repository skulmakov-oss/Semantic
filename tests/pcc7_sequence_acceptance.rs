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

    let artifact = temp_semcode_artifact("pcc7-sequence", "smc_pcc7_sequence_acceptance");
    compile_source_to_artifact(&source, &artifact);
    verify_artifact(&artifact);
}

#[test]
fn pcc7_sequence_indexing_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc7_sequence/positive_sequence_indexing.sm");
}

#[test]
fn pcc7_sequence_iteration_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc7_sequence/positive_sequence_iteration.sm");
}

#[test]
fn pcc7_sequence_len_empty_contains_fixture_passes_full_cli_path() {
    check_run_compile_verify(
        "tests/fixtures/pcc7_sequence/positive_sequence_len_empty_contains.sm",
    );
}

#[test]
fn pcc7_sequence_push_prepend_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc7_sequence/positive_sequence_push_prepend.sm");
}

#[test]
fn pcc7_sequence_pop_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc7_sequence/positive_sequence_pop.sm");
}

#[test]
fn pcc7_sequence_function_boundary_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc7_sequence/positive_sequence_function_boundary.sm");
}
