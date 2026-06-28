use std::path::PathBuf;
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

fn check_compile_verify_run(root_rel: &str) {
    let root = repo_path(root_rel);
    cli_ok(
        vec!["check".to_string(), root.clone()],
        &format!("smc check for {root}"),
    );

    let temp_dir = std::env::temp_dir().join(format!(
        "project_manifest_surface_qualification_{}_{}",
        std::process::id(),
        root_rel.replace(['/', '\\'], "_")
    ));
    std::fs::create_dir_all(&temp_dir).expect("mkdir");
    let out = temp_dir.join("out.smc");
    let out_arg = out.to_string_lossy().replace('\\', "/");

    cli_ok(
        vec![
            "compile".to_string(),
            root.clone(),
            "-o".to_string(),
            out_arg.clone(),
        ],
        &format!("smc compile for {root}"),
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

fn assert_check_rejects(root_rel: &str, needle: &str) {
    let root = repo_path(root_rel);
    let err = cli_err(
        vec!["check".to_string(), root.clone()],
        &format!("smc check for {root}"),
    );
    assert!(
        err.contains(needle),
        "expected diagnostic containing '{needle}' for {root_rel}, got: {err}"
    );
}

fn assert_missing_project_manifest_rejects() {
    let dir = mk_temp_dir("project_manifest_surface_missing_manifest");
    std::fs::create_dir_all(dir.join("src")).expect("mkdir src");
    std::fs::write(dir.join("src/main.sm"), "fn main() { return; }\n").expect("write source");

    let err = cli_err(
        vec![
            "check".to_string(),
            dir.to_string_lossy().replace('\\', "/"),
        ],
        "smc check for missing project manifest",
    );
    assert!(
        err.contains("must contain 'semantic.toml' or 'Semantic.package'"),
        "expected missing manifest diagnostic, got: {err}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn project_manifest_surface_positive_roots_run_end_to_end() {
    let positive_roots = [
        "examples/qualification/project_manifest_surface/positive_semantic_toml_minimal",
        "examples/qualification/project_manifest_surface/positive_semantic_toml_nested_entry",
        "examples/qualification/project_manifest_surface/positive_package_baseline_local_import",
    ];

    for root in positive_roots {
        check_compile_verify_run(root);
    }
}

#[test]
fn project_manifest_surface_negative_manifest_cases_reject_deterministically() {
    let negative_cases = [
        (
            "examples/qualification/project_manifest_surface/negative_missing_package_name",
            "missing required [package].name",
        ),
        (
            "examples/qualification/project_manifest_surface/negative_semantic_toml_entry_missing",
            "resolves to missing file",
        ),
        (
            "examples/qualification/project_manifest_surface/negative_semantic_toml_escape_entry",
            "must not escape the project root",
        ),
        (
            "examples/qualification/project_manifest_surface/negative_package_missing_directive",
            "package manifest requires an explicit package directive",
        ),
        (
            "examples/qualification/project_manifest_surface/negative_package_duplicate_directive",
            "duplicate package manifest directive 'package'",
        ),
    ];

    for (root, needle) in negative_cases {
        assert_check_rejects(root, needle);
    }
}

#[test]
fn project_manifest_surface_missing_project_manifest_is_rejected() {
    assert_missing_project_manifest_rejects();
}
