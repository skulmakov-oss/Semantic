#[path = "support/cli_artifact_support.rs"]
mod cli_artifact_support;

use cli_artifact_support::{
    compile_source_to_artifact, temp_semcode_artifact, temp_source_file, verify_artifact,
};

use std::process::{Command, Output};

fn svm_output(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_svm"))
        .args(args)
        .output()
        .expect("run svm")
}

fn assert_success(output: &Output, context: &str) {
    assert!(
        output.status.success(),
        "{context} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).is_empty(),
        "{context} wrote stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cli_quickstart_svm_entrypoints_work_for_verified_zero_effect_semcode() {
    let source = temp_source_file(
        "cli_quickstart_svm_smoke",
        "program.sm",
        r#"
fn main() {
    return;
}
"#,
    );
    let artifact = temp_semcode_artifact("cli_quickstart_svm_smoke", "program");
    let source_arg = source.cli_arg();
    let artifact_arg = artifact.cli_arg();

    compile_source_to_artifact(&source, &artifact);
    verify_artifact(&artifact);

    let svm_run = svm_output(&["run", &artifact_arg]);
    assert_success(
        &svm_run,
        "svm run <input.smc> for verified zero-effect SemCode",
    );

    let svm_disasm = svm_output(&["disasm", &artifact_arg]);
    assert_success(
        &svm_disasm,
        "svm disasm <input.smc> for verified zero-effect SemCode",
    );
    let stdout = String::from_utf8_lossy(&svm_disasm.stdout);
    assert!(
        stdout.contains("fn main:"),
        "disassembly did not expose the expected main function marker: {stdout}"
    );

    let _ = std::fs::remove_file(&source_arg);
    let _ = std::fs::remove_dir_all(artifact.root_dir());
}
