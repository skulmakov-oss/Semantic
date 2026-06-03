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

    let artifact = temp_semcode_artifact("pcc5-adt", "smc_pcc5_adt_acceptance");
    compile_source_to_artifact(&source, &artifact);
    verify_artifact(&artifact);
}

#[test]
fn pcc5_adt_declaration_only_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc5_adt/pcc5_adt_declaration_only.sm");
}

#[test]
fn pcc5_adt_unit_constructor_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc5_adt/pcc5_adt_unit_constructor.sm");
}

#[test]
fn pcc5_adt_constructor_function_boundary_fixture_passes_full_cli_path() {
    check_run_compile_verify("tests/fixtures/pcc5_adt/pcc5_adt_constructor_function_boundary.sm");
}
