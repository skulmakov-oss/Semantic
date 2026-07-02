#[path = "support/cli_artifact_support.rs"]
mod cli_artifact_support;

use std::path::PathBuf;

use cli_artifact_support::{
    check_source, compile_source_to_artifact, run_smc_artifact, run_source, source_fixture,
    target_cli_smoke_artifact, verify_artifact,
};

fn smoke_canonical_fixture(rel: &str, fixture_name: &str) {
    let source = source_fixture(rel);
    check_source(&source);
    run_source(&source);

    let artifact = target_cli_smoke_artifact("cli-public-smoke", fixture_name);
    let target_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("cli-smoke");
    assert!(
        artifact.root_dir().starts_with(&target_root),
        "smoke artifact root must stay under target/cli-smoke: {}",
        artifact.root_dir().display()
    );

    compile_source_to_artifact(&source, &artifact);
    verify_artifact(&artifact);
    run_smc_artifact(&artifact);
}

#[test]
fn cli_public_smoke_matrix_covers_canonical_source_and_artifact_paths() {
    for (rel, fixture_name) in [
        (
            "examples/canonical/cli_batch_core/src/main.sm",
            "cli_batch_core",
        ),
        (
            "examples/canonical/data_audit_record_iterable/src/main.sm",
            "data_audit_record_iterable",
        ),
        (
            "examples/canonical/text_collections_toolbox/src/main.sm",
            "text_collections_toolbox",
        ),
        (
            "examples/canonical/stdlib_v0_helpers/src/main.sm",
            "stdlib_v0_helpers",
        ),
        (
            "examples/canonical/collections_core/src/main.sm",
            "collections_core",
        ),
        ("examples/canonical/text_core/src/main.sm", "text_core"),
        (
            "examples/canonical/match_control_flow/src/main.sm",
            "match_control_flow",
        ),
        (
            "examples/canonical/option_result_control_flow/src/main.sm",
            "option_result_control_flow",
        ),
        (
            "examples/canonical/loop_control_flow/src/main.sm",
            "loop_control_flow",
        ),
        (
            "examples/canonical/rule_state_decision/src/main.sm",
            "rule_state_decision",
        ),
    ] {
        smoke_canonical_fixture(rel, fixture_name);
    }
}
