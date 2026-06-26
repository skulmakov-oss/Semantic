use std::path::PathBuf;

fn run_err(args: &[&str]) -> String {
    smc_cli::run(args.iter().map(|s| s.to_string()).collect())
        .expect_err("expected command-shape failure")
}

fn assert_usage_error(args: &[&str], expected_usage_fragment: &str) {
    let err = run_err(args);
    assert!(
        err.contains(expected_usage_fragment),
        "expected usage fragment '{expected_usage_fragment}' in error:\n{err}"
    );
}

fn assert_unknown_flag_error(args: &[&str]) {
    let err = run_err(args);
    assert!(
        err.contains("unknown flag '--bogus'"),
        "expected unknown-flag error in:\n{err}"
    );
}

#[test]
fn smc_commands_reject_missing_arguments_with_usage() {
    assert_usage_error(&["check"], "usage: smc check");
    assert_usage_error(&["compile"], "usage: smc compile");
    assert_usage_error(&["run"], "usage: smc run");
    assert_usage_error(&["verify"], "usage: smc verify");
    assert_usage_error(&["run-smc"], "usage: smc run-smc");
}

#[test]
fn smc_commands_reject_leading_unknown_flags() {
    assert_unknown_flag_error(&["check", "--bogus"]);
    assert_unknown_flag_error(&["compile", "--bogus"]);
    assert_unknown_flag_error(&["run", "--bogus"]);
    assert_unknown_flag_error(&["verify", "--bogus"]);
    assert_unknown_flag_error(&["run-smc", "--bogus"]);
}

#[test]
fn smc_check_keeps_real_missing_path_behavior_distinct() {
    let missing = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("cli_command_shape_diagnostics")
        .join("definitely_missing_source.sm");

    if let Some(parent) = missing.parent() {
        assert!(
            !parent.exists(),
            "test path unexpectedly exists: {}",
            parent.display()
        );
    }

    let err = run_err(&["check", &missing.to_string_lossy()]);
    assert!(
        err.contains("failed to resolve")
            || err.contains("failed to read")
            || err.contains("package module admission error"),
        "expected a path/admission error, got:\n{err}"
    );
    assert!(
        !err.contains("unknown flag"),
        "missing path should not be mistaken for a CLI flag:\n{err}"
    );
}
