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

    let temp_dir = mk_temp_dir("collections_map_surface_qualification");
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
        err.contains("Error [E0201]"),
        "expected E0201 for {rel}, got: {err}"
    );
    assert!(
        err.contains(needle),
        "expected diagnostic containing '{needle}' for {rel}, got: {err}"
    );
}

#[test]
fn collections_map_surface_positive_sequence_and_map_fixtures_run_end_to_end() {
    let positive_cases = [
        "examples/qualification/collections_map_surface/positive_sequence_basics/src/main.sm",
        "examples/qualification/collections_map_surface/positive_sequence_mutation/src/main.sm",
        "examples/qualification/collections_map_surface/positive_sequence_loop/src/main.sm",
        "examples/qualification/collections_map_surface/positive_map_basics/src/main.sm",
    ];

    for rel in positive_cases {
        run_full_pipeline(rel);
    }
}

#[test]
fn collections_map_surface_negative_fixtures_reject_deterministically() {
    let negative_cases = [
        (
            "examples/qualification/collections_map_surface/negative_sequence_bool_index/src/main.sm",
            "sequence indexing currently requires i32 index",
        ),
        (
            "examples/qualification/collections_map_surface/negative_sequence_value_type_mismatch/src/main.sm",
            "type mismatch in ordered sequence item 1: I32 vs Bool",
        ),
        (
            "examples/qualification/collections_map_surface/negative_map_missing_context/src/main.sm",
            "map_empty() requires a contextual Map(K, V) type",
        ),
        (
            "examples/qualification/collections_map_surface/negative_map_key_type_mismatch/src/main.sm",
            "builtin 'map_set' key type Bool does not match map key type I32",
        ),
        (
            "examples/qualification/collections_map_surface/negative_map_value_type_mismatch/src/main.sm",
            "builtin 'map_set' value type I32 does not match map value type Bool",
        ),
    ];

    for (rel, needle) in negative_cases {
        assert_check_rejects(rel, needle);
    }
}
