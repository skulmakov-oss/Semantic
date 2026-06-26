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

fn run_full_pipeline(rel: &str) {
    let path = repo_path(rel);
    cli_ok(
        vec!["check".to_string(), path.clone()],
        &format!("smc check for {path}"),
    );

    let temp_dir = mk_temp_dir("record_copy_with_surface_qualification");
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
        err.contains("record copy-with"),
        "expected record copy-with diagnostic for {rel}, got: {err}"
    );
    assert!(
        err.contains(needle),
        "expected diagnostic containing '{needle}' for {rel}, got: {err}"
    );
}

#[test]
fn record_copy_with_surface_positive_fixtures_run_end_to_end() {
    let positive_cases = [
        "examples/qualification/record_copy_with_surface/positive_single_field_update/src/main.sm",
        "examples/qualification/record_copy_with_surface/positive_multi_field_update/src/main.sm",
        "examples/qualification/record_copy_with_surface/positive_copy_with_in_function/src/main.sm",
    ];

    for rel in positive_cases {
        run_full_pipeline(rel);
    }
}

#[test]
fn record_copy_with_surface_negative_fixtures_reject_deterministically() {
    let negative_cases = [
        (
            "examples/qualification/record_copy_with_surface/negative_unknown_field_update/src/main.sm",
            "record copy-with 'Pair' has no field named 'middle'",
        ),
        (
            "examples/qualification/record_copy_with_surface/negative_update_type_mismatch/src/main.sm",
            "type mismatch in record copy-with 'Pair.left'",
        ),
        (
            "examples/qualification/record_copy_with_surface/negative_copy_with_non_record_base/src/main.sm",
            "record copy-with requires record base before 'with', got I32",
        ),
        (
            "examples/qualification/record_copy_with_surface/negative_duplicate_update_field/src/main.sm",
            "record copy-with 'Pair' cannot repeat field 'left'",
        ),
    ];

    for (rel, needle) in negative_cases {
        assert_check_rejects(rel, needle);
    }
}
