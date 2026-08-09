use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_path(rel: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(rel)
        .to_string_lossy()
        .replace('\\', "/")
}

fn cli_ok(command: &str, rel: &str) {
    let path = repo_path(rel);
    smc_cli::run(vec![command.to_string(), path.clone()])
        .unwrap_or_else(|err| panic!("smc {command} failed for {path}: {err}"));
}

fn cli_err(command: &str, rel: &str) -> String {
    let path = repo_path(rel);
    smc_cli::run(vec![command.to_string(), path.clone()])
        .expect_err(&format!("smc {command} unexpectedly passed for {path}"))
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

fn compile_bytes(rel: &str) -> Vec<u8> {
    let input = repo_path(rel);
    let dir = mk_temp_dir("smc_exec_compile_determinism");
    let out = dir.join("out.smc");
    let out_arg = out.to_string_lossy().replace('\\', "/");
    smc_cli::run(vec![
        "compile".to_string(),
        input.clone(),
        "-o".to_string(),
        out_arg.clone(),
    ])
    .unwrap_or_else(|err| panic!("smc compile failed for {input}: {err}"));
    let bytes = std::fs::read(&out).expect("compiled bytes");
    let _ = std::fs::remove_dir_all(&dir);
    bytes
}

#[test]
fn executable_module_entry_wave2_local_helper_import_checks_and_runs() {
    let rel =
        "examples/qualification/executable_module_entry/wave2_local_helper_import/src/main.sm";
    cli_ok("check", rel);
    cli_ok("run", rel);
}

#[test]
fn executable_module_entry_selected_import_checks_and_runs() {
    let rel = "examples/qualification/executable_module_entry/positive_selected_import/src/main.sm";
    cli_ok("check", rel);
    cli_ok("run", rel);
}

#[test]
fn executable_module_entry_selected_import_aliases_relieve_helper_collisions() {
    let rel = "examples/qualification/executable_module_entry/positive_selected_import_alias_collision/src/main.sm";
    cli_ok("check", rel);
    cli_ok("run", rel);
}

#[test]
fn executable_module_entry_repeated_direct_import_is_deduped() {
    let rel =
        "examples/qualification/executable_module_entry/positive_repeated_direct_import/src/main.sm";
    cli_ok("check", rel);
    cli_ok("run", rel);
}

#[test]
fn executable_module_entry_repeated_transitive_helper_import_is_deduped() {
    let rel = "examples/qualification/executable_module_entry/positive_repeated_transitive_import/src/main.sm";
    cli_ok("check", rel);
    cli_ok("run", rel);
}

#[test]
fn executable_module_entry_helper_graph_compile_is_deterministic() {
    let rel = "examples/qualification/executable_module_entry/positive_repeated_transitive_import/src/main.sm";
    let first = compile_bytes(rel);
    let second = compile_bytes(rel);
    assert_eq!(
        first, second,
        "helper-module compile output must stay deterministic across repeated builds"
    );
}

#[test]
fn executable_module_entry_negative_out_of_scope_import_forms_report_wave2_boundary() {
    let wave2_boundary =
        "top-level executable Import admits direct local-path and package-qualified helper modules plus selected local imports";
    let cases = [
        "examples/qualification/executable_module_entry/negative_alias_import/src/main.sm",
        "examples/qualification/executable_module_entry/negative_wildcard_import/src/main.sm",
        "examples/qualification/executable_module_entry/negative_reexport_import/src/main.sm",
    ];

    for rel in cases {
        let err = cli_err("check", rel);
        assert!(
            err.contains(wave2_boundary),
            "expected wave2 executable import boundary diagnostic for {rel}, got: {err}"
        );
    }
}

#[test]
fn executable_module_entry_package_qualified_import_requires_declared_alias() {
    let rel =
        "examples/qualification/executable_module_entry/negative_package_qualified_import/src/main.sm";
    let err = cli_err("check", rel);
    assert!(
        err.contains("does not declare dependency alias 'math'"),
        "expected package alias diagnostic, got: {err}"
    );
}

#[test]
fn executable_module_entry_negative_graph_and_namespace_cases_report_explicit_failures() {
    let cases = [
        (
            "examples/qualification/executable_module_entry/negative_cycle_bare_import/src/main.sm",
            "cyclic executable helper import detected:",
        ),
        (
            "examples/qualification/executable_module_entry/negative_duplicate_symbol_collision/src/main.sm",
            "duplicate function 'score'",
        ),
        (
            "examples/qualification/executable_module_entry/negative_namespace_collision/src/main.sm",
            "duplicate function 'score'",
        ),
        (
            "examples/qualification/executable_module_entry/negative_transitive_duplicate_symbol_collision/src/main.sm",
            "duplicate function 'score'",
        ),
    ];

    for (rel, needle) in cases {
        let err = cli_err("check", rel);
        assert!(
            err.contains(needle),
            "expected diagnostic '{needle}' for {rel}, got: {err}"
        );
    }
}

#[test]
fn executable_module_entry_multi_hop_cycle_reports_deterministic_chain() {
    let rel =
        "examples/qualification/executable_module_entry/negative_multi_hop_cycle_bare_import/src/main.sm";
    let err = cli_err("check", rel);
    assert!(
        err.contains("cyclic executable helper import detected:"),
        "expected cycle diagnostic, got: {err}"
    );
    for needle in ["a.sm", "b.sm", "c.sm"] {
        assert!(
            err.contains(needle),
            "expected cycle diagnostic to mention {needle}, got: {err}"
        );
    }
}
