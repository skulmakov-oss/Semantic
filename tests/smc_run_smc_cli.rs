#[path = "support/cli_artifact_support.rs"]
mod cli_artifact_support;

use cli_artifact_support::{
    compile_source_to_artifact, run_smc_artifact, source_fixture, temp_semcode_artifact,
    verify_artifact,
};

#[test]
fn smc_run_smc_executes_emitted_semcode_artifact() {
    let source = source_fixture("examples/canonical/cli_batch_core/src/main.sm");
    let artifact = temp_semcode_artifact("smc-run-smc-cli", "cli_batch_core");

    compile_source_to_artifact(&source, &artifact);
    verify_artifact(&artifact);
    run_smc_artifact(&artifact);
}
