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

    let artifact = temp_semcode_artifact("pcc7-map", "smc_pcc7_map_acceptance");
    compile_source_to_artifact(&source, &artifact);
    verify_artifact(&artifact);
}

#[test]
fn pcc7_map_empty_contextual_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc7_map/positive_map_empty_contextual.sm");
}

#[test]
fn pcc7_map_basic_insert_lookup_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc7_map/positive_map_basic_insert_lookup.sm");
}

#[test]
fn pcc7_map_persistent_update_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc7_map/positive_map_persistent_update.sm");
}
