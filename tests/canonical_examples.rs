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

use std::sync::atomic::{AtomicUsize, Ordering};

static DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn mk_temp_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "{}_{}_{}_{}",
        prefix,
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos(),
        DIR_COUNTER.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::create_dir_all(&dir).expect("mkdir");
    dir
}

fn check_run_compile_verify(rel: &str) {
    let input = repo_path(rel);
    cli_ok(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for {input}"),
    );
    cli_ok(
        vec!["run".to_string(), input.clone()],
        &format!("smc run for {input}"),
    );

    let dir = mk_temp_dir("smc_canonical_examples");
    let out = dir.join("out.smc");
    let out_arg = out.to_string_lossy().replace('\\', "/");
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
        vec!["verify".to_string(), out_arg],
        &format!("smc verify for {input}"),
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn canonical_positive_examples_check_run_compile_and_verify() {
    for rel in [
        "examples/canonical/cli_batch_core/src/main.sm",
        "examples/canonical/rule_state_decision/src/main.sm",
        "examples/canonical/data_audit_record_iterable/src/main.sm",
        "examples/canonical/wave2_local_helper_import/src/main.sm",
        "examples/canonical/positive_selected_import/src/main.sm",
    ] {
        check_run_compile_verify(rel);
    }
}

#[test]
fn canonical_rule_state_decision_is_the_first_application_completeness_anchor() {
    check_run_compile_verify("examples/canonical/rule_state_decision/src/main.sm");
}

#[test]
fn canonical_data_audit_record_iterable_is_the_second_application_completeness_anchor() {
    check_run_compile_verify("examples/canonical/data_audit_record_iterable/src/main.sm");
}

#[test]
fn canonical_cli_batch_core_is_the_third_application_completeness_anchor() {
    check_run_compile_verify("examples/canonical/cli_batch_core/src/main.sm");
}

#[test]
fn canonical_boundary_example_reports_current_alias_limit() {
    let input = repo_path("examples/canonical/boundary_alias_import/src/main.sm");
    let err = cli_err(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for {input}"),
    );
    assert!(
        err.contains(
            "top-level executable Import currently admits direct local-path helper-module imports plus selected imports in wave2"
        ),
        "expected executable alias boundary diagnostic, got: {err}"
    );
}
