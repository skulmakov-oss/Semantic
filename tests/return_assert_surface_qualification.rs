use std::path::PathBuf;

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

fn check_compile_verify_run(rel: &str) {
    let path = repo_path(rel);
    cli_ok(
        vec!["check".to_string(), path.clone()],
        &format!("smc check for {path}"),
    );

    let temp_dir = std::env::temp_dir().join(format!(
        "return_assert_surface_qualification_{}_{}",
        std::process::id(),
        rel.replace(['/', '\\'], "_")
    ));
    std::fs::create_dir_all(&temp_dir).expect("mkdir");
    let out = temp_dir.join("out.smc");
    let out_arg = out.to_string_lossy().replace('\\', "/");

    cli_ok(
        vec![
            "compile".to_string(),
            path.clone(),
            "-o".to_string(),
            out_arg.clone(),
        ],
        &format!("smc compile for {path}"),
    );
    cli_ok(
        vec!["verify".to_string(), out_arg.clone()],
        &format!("smc verify for {out_arg}"),
    );
    cli_ok(
        vec!["run-smc".to_string(), out_arg.clone()],
        &format!("smc run-smc for {out_arg}"),
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

fn assert_check_rejects(rel: &str, needle: &str) {
    let path = repo_path(rel);
    let err = cli_err(
        vec!["check".to_string(), path.clone()],
        &format!("smc check for {path}"),
    );
    assert!(
        err.contains(needle),
        "expected diagnostic '{needle}' for {rel}, got: {err}"
    );
}

fn assert_runtime_trap(rel: &str, needle: &str) {
    let path = repo_path(rel);
    cli_ok(
        vec!["check".to_string(), path.clone()],
        &format!("smc check for {path}"),
    );

    let temp_dir = std::env::temp_dir().join(format!(
        "return_assert_surface_trap_{}_{}",
        std::process::id(),
        rel.replace(['/', '\\'], "_")
    ));
    std::fs::create_dir_all(&temp_dir).expect("mkdir");
    let out = temp_dir.join("out.smc");
    let out_arg = out.to_string_lossy().replace('\\', "/");

    cli_ok(
        vec![
            "compile".to_string(),
            path.clone(),
            "-o".to_string(),
            out_arg.clone(),
        ],
        &format!("smc compile for {path}"),
    );
    cli_ok(
        vec!["verify".to_string(), out_arg.clone()],
        &format!("smc verify for {out_arg}"),
    );

    let err = cli_err(
        vec!["run-smc".to_string(), out_arg.clone()],
        &format!("smc run-smc for {out_arg}"),
    );
    assert!(
        err.contains(needle),
        "expected runtime trap containing '{needle}' for {rel}, got: {err}"
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn return_assert_surface_positive_cases_run_end_to_end() {
    let positive_cases = [
        "examples/qualification/return_assert_surface/positive_explicit_return_value/src/main.sm",
        "examples/qualification/return_assert_surface/positive_branch_return_value/src/main.sm",
        "examples/qualification/return_assert_surface/positive_assert_true/src/main.sm",
        "examples/qualification/return_assert_surface/positive_assert_computed_condition/src/main.sm",
    ];

    for rel in positive_cases {
        check_compile_verify_run(rel);
    }
}

#[test]
fn return_assert_surface_static_negative_cases_reject_deterministically() {
    let negative_cases = [
        (
            "examples/qualification/return_assert_surface/negative_return_type_mismatch/src/main.sm",
            "return type mismatch",
        ),
        (
            "examples/qualification/return_assert_surface/negative_missing_return_value/src/main.sm",
            "return type mismatch",
        ),
        (
            "examples/qualification/return_assert_surface/negative_assert_non_bool/src/main.sm",
            "assert builtin requires bool condition",
        ),
        (
            "examples/qualification/return_assert_surface/negative_branch_return_type_mismatch/src/main.sm",
            "return type mismatch",
        ),
    ];

    for (rel, needle) in negative_cases {
        assert_check_rejects(rel, needle);
    }
}

#[test]
fn return_assert_surface_assert_false_traps_at_runtime() {
    assert_runtime_trap(
        "examples/qualification/return_assert_surface/negative_assert_false_runtime_trap/src/main.sm",
        "assertion failed",
    );
}
