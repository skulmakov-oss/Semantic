use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_path(rel: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(rel)
        .to_string_lossy()
        .replace('\\', "/")
}

fn cli_ok(args: Vec<String>, context: &str) {
    smc_cli::run(args).unwrap_or_else(|err| panic!("{context} failed: {err}"));
}

fn cli_err(args: Vec<String>, context: &str) -> String {
    smc_cli::run(args).expect_err(&format!("{context} unexpectedly passed"))
}

fn mk_temp_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "{}_{}_{}",
        prefix,
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("mkdir");
    dir
}

fn compile_verify_run_smc(rel: &str, out_name: &str) -> Vec<u8> {
    let input = repo_path(rel);
    let dir = mk_temp_dir("smc_pcc1_control_flow_gate");
    let out = dir.join(out_name);
    let out_arg = out.to_string_lossy().replace('\\', "/");

    cli_ok(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for {input}"),
    );
    cli_ok(
        vec!["run".to_string(), input.clone()],
        &format!("smc run for {input}"),
    );
    cli_ok(
        vec![
            "compile".to_string(),
            input.clone(),
            "-o".to_string(),
            out_arg.clone(),
        ],
        &format!("smc compile for {input}"),
    );
    cli_ok(
        vec!["verify".to_string(), out_arg.clone()],
        &format!("smc verify for {out_arg}"),
    );
    cli_ok(
        vec!["run-smc".to_string(), out_arg.clone()],
        &format!("smc run-smc for {out_arg}"),
    );

    let bytes = std::fs::read(&out)
        .unwrap_or_else(|err| panic!("read emitted semcode failed for {out_arg}: {err}"));
    let _ = std::fs::remove_dir_all(&dir);
    bytes
}

// PCC-1 gate mapping:
// - Syntax Hell: parser acceptance of if / while / loop surfaces
// - Type Hell: bool loop conditions and loop-control diagnostics
// - Lowering Hell: stable compile-to-SemCode paths
// - Verifier Hell: emitted SemCode must verify before execution
// - VM Hell: emitted SemCode must run deterministically
// - Practical Hell: public CLI pipeline coverage
// - User Pain / Diagnostics Hell: stable rejection messages

#[test]
fn pcc1_if_else_positive_pipeline() {
    compile_verify_run_smc(
        "tests/fixtures/pcc1_control_flow/positive_if_else.sm",
        "if_else_positive.smc",
    );
}

#[test]
fn pcc1_while_positive_pipeline() {
    compile_verify_run_smc(
        "tests/fixtures/pcc1_control_flow/positive_while.sm",
        "while_positive.smc",
    );
}

#[test]
fn pcc1_loop_break_continue_positive_pipeline() {
    compile_verify_run_smc(
        "tests/fixtures/pcc1_control_flow/positive_loop_break_continue.sm",
        "loop_break_continue_positive.smc",
    );
}

#[test]
fn pcc1_break_outside_loop_rejects() {
    let input = repo_path("tests/fixtures/pcc1_control_flow/negative_break_outside_loop.sm");
    let err = cli_err(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for {input}"),
    );
    assert!(
        err.contains("Error [E0201]"),
        "expected stable control-flow diagnostic code for break-outside-loop rejection, got: {err}"
    );
    assert!(
        err.contains("bare break is allowed only inside while or statement loop"),
        "expected break-outside-loop diagnostic, got: {err}"
    );
}

#[test]
fn pcc1_continue_outside_loop_rejects() {
    let input = repo_path("tests/fixtures/pcc1_control_flow/negative_continue_outside_loop.sm");
    let err = cli_err(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for {input}"),
    );
    assert!(
        err.contains("Error [E0201]"),
        "expected stable control-flow diagnostic code for continue-outside-loop rejection, got: {err}"
    );
    assert!(
        err.contains("continue is allowed only inside while or statement loop"),
        "expected continue-outside-loop diagnostic, got: {err}"
    );
}

#[test]
fn pcc1_while_non_bool_condition_rejects() {
    let input = repo_path("tests/fixtures/pcc1_control_flow/negative_while_non_bool_condition.sm");
    let err = cli_err(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for {input}"),
    );
    assert!(
        err.contains("Error [E0201]"),
        "expected stable control-flow diagnostic code for non-bool while rejection, got: {err}"
    );
    assert!(
        err.contains("while condition must be bool"),
        "expected non-bool while diagnostic, got: {err}"
    );
}

#[test]
fn pcc1_control_flow_repeated_pipeline_is_stable() {
    let input = "tests/fixtures/pcc1_control_flow/positive_loop_break_continue.sm";
    let first = compile_verify_run_smc(input, "repeated_first.smc");
    let second = compile_verify_run_smc(input, "repeated_second.smc");
    assert_eq!(first, second, "emitted SemCode drifted across repeated PCC-1 gate runs");
}

#[test]
fn pcc1_nested_while_break_positive_pipeline() {
    compile_verify_run_smc(
        "tests/fixtures/pcc1_control_flow/positive_nested_while_break.sm",
        "nested_while_break_positive.smc",
    );
}

#[test]
fn pcc1_nested_loop_continue_positive_pipeline() {
    compile_verify_run_smc(
        "tests/fixtures/pcc1_control_flow/positive_nested_loop_continue.sm",
        "nested_loop_continue_positive.smc",
    );
}

#[test]
fn pcc1_nested_loop_break_then_outer_continue_positive_pipeline() {
    compile_verify_run_smc(
        "tests/fixtures/pcc1_control_flow/positive_nested_loop_break_then_outer_continue.sm",
        "nested_loop_break_then_outer_continue_positive.smc",
    );
}

#[test]
fn pcc1_break_after_nested_loop_outside_loop_rejects() {
    let input = repo_path(
        "tests/fixtures/pcc1_control_flow/negative_break_after_nested_loop_outside_loop.sm",
    );
    let err = cli_err(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for {input}"),
    );
    assert!(
        err.contains("Error [E0201]"),
        "expected stable control-flow diagnostic code for break-after-nested-loop rejection, got: {err}"
    );
    assert!(
        err.contains("bare break is allowed only inside while or statement loop"),
        "expected break-after-nested-loop diagnostic, got: {err}"
    );
}

#[test]
fn pcc1_continue_after_nested_loop_outside_loop_rejects() {
    let input = repo_path(
        "tests/fixtures/pcc1_control_flow/negative_continue_after_nested_loop_outside_loop.sm",
    );
    let err = cli_err(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for {input}"),
    );
    assert!(
        err.contains("Error [E0201]"),
        "expected stable control-flow diagnostic code for continue-after-nested-loop rejection, got: {err}"
    );
    assert!(
        err.contains("continue is allowed only inside while or statement loop"),
        "expected continue-after-nested-loop diagnostic, got: {err}"
    );
}

#[test]
fn pcc1_nested_control_flow_repeated_pipeline_is_stable() {
    let input = "tests/fixtures/pcc1_control_flow/positive_nested_loop_break_then_outer_continue.sm";
    let first = compile_verify_run_smc(input, "nested_repeated_first.smc");
    let second = compile_verify_run_smc(input, "nested_repeated_second.smc");
    assert_eq!(first, second, "nested control-flow emitted SemCode drifted across repeated PCC-1 gate runs");
}
