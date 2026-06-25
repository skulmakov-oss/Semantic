use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
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

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Depth {
    CheckOnly,
    CompileOk,
    VerifyOk,
    RunOk,
}

fn run_depth(rel: &str, depth: Depth) {
    let path = repo_path(rel);
    cli_ok(
        vec!["check".to_string(), path.clone()],
        &format!("smc check for {path}"),
    );
    if matches!(depth, Depth::CheckOnly) {
        return;
    }

    let temp_dir = mk_temp_dir("short_lambda_surface_qualification");
    let out = temp_dir.join(
        Path::new(rel)
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("out.smc")),
    );
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
    if matches!(depth, Depth::CompileOk) {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return;
    }

    cli_ok(
        vec!["verify".to_string(), out_arg.clone()],
        &format!("smc verify for {out_arg}"),
    );
    if matches!(depth, Depth::VerifyOk) {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return;
    }

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
        "expected diagnostic containing '{needle}' for {rel}, got: {err}"
    );
}

#[test]
fn short_lambda_surface_positive_fixtures_are_run_ok() {
    let positive_cases = [
        "examples/qualification/short_lambda_surface/positive_single_arg_lambda/src/main.sm",
        "examples/qualification/short_lambda_surface/positive_lambda_in_let_binding/src/main.sm",
        "examples/qualification/short_lambda_surface/positive_lambda_inside_function/src/main.sm",
        "examples/qualification/short_lambda_surface/positive_pipeline_short_lambda/src/main.sm",
    ];

    for rel in positive_cases {
        run_depth(rel, Depth::RunOk);
    }
}

#[test]
fn short_lambda_surface_negative_fixtures_reject_deterministically() {
    let negative_cases = [
        (
            "examples/qualification/short_lambda_surface/negative_multi_arg_lambda_if_unsupported/src/main.sm",
            "expected ')' after tuple literal",
        ),
        (
            "examples/qualification/short_lambda_surface/negative_captureful_short_lambda/src/main.sm",
            "short lambda v0 is capture-free only; body may not reference non-local 'offset'",
        ),
        (
            "examples/qualification/short_lambda_surface/negative_lambda_missing_context/src/main.sm",
            "contextual Closure(T -> U) type",
        ),
    ];

    for (rel, needle) in negative_cases {
        assert_check_rejects(rel, needle);
    }
}
