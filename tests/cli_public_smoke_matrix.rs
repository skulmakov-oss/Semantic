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
            "examples/canonical/rule_state_decision/src/main.sm",
            "rule_state_decision",
        ),
    ] {
        smoke_canonical_fixture(rel, fixture_name);
    }
}
