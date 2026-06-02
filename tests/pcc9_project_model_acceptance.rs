use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use smc_cli::{
    admit_package_entry_module, parse_package_manifest_baseline, resolve_package_import_path,
    validate_package_manifest_baseline, PackageDependencySource, PACKAGE_MANIFEST_FILE_NAME,
};

fn repo_path(rel: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(rel)
        .to_string_lossy()
        .replace('\\', "/")
}

fn fixture_path(rel: &str) -> String {
    repo_path(&format!("tests/fixtures/pcc9_project_model/{rel}"))
}

fn read_fixture(rel: &str) -> String {
    std::fs::read_to_string(fixture_path(rel))
        .unwrap_or_else(|err| panic!("read fixture {rel}: {err}"))
}

fn cli_ok(args: Vec<String>, context: &str) {
    smc_cli::run(args).unwrap_or_else(|err| panic!("{context} failed: {err}"));
}

fn cli_stdout(args: Vec<String>, context: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_smc"))
        .args(args)
        .output()
        .unwrap_or_else(|err| panic!("{context} failed to spawn smc: {err}"));
    assert!(
        output.status.success(),
        "{context} failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn cli_output(
    args: Vec<String>,
    cwd: Option<&std::path::Path>,
    context: &str,
) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_smc"));
    command.args(args);
    if let Some(dir) = cwd {
        command.current_dir(dir);
    }
    command
        .output()
        .unwrap_or_else(|err| panic!("{context} failed to spawn smc: {err}"))
}

fn cli_command_output(
    command: &str,
    input: &str,
    cwd: Option<&std::path::Path>,
    context: &str,
) -> std::process::Output {
    cli_output(vec![command.to_string(), input.to_string()], cwd, context)
}

fn cli_check_project_root_ok(dir: &std::path::Path, context: &str) {
    let input = normalize_path(dir);
    cli_ok(vec!["check".to_string(), input], context);
}

fn cli_run_project_root_ok(dir: &std::path::Path, context: &str) {
    let input = normalize_path(dir);
    cli_ok(vec!["run".to_string(), input], context);
}

// Shared output helpers keep the project-root acceptance matrix aligned across command families.
fn cli_command_project_root_output(command: &str, dir: &std::path::Path, context: &str) -> String {
    let input = normalize_path(dir);
    cli_stdout(vec![command.to_string(), input], context)
}

fn cli_command_project_root_output_dot(
    command: &str,
    dir: &std::path::Path,
    context: &str,
) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_smc"))
        .arg(command)
        .arg(".")
        .current_dir(dir)
        .output()
        .unwrap_or_else(|err| panic!("{context} failed to spawn smc: {err}"));
    assert!(
        output.status.success(),
        "{context} failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn cli_command_file_output(command: &str, path: &std::path::Path, context: &str) -> String {
    cli_stdout(vec![command.to_string(), normalize_path(path)], context)
}

fn cli_hash_ast_project_root_output(dir: &std::path::Path, context: &str) -> String {
    cli_command_project_root_output("hash-ast", dir, context)
}

fn cli_hash_ast_project_root_output_dot(dir: &std::path::Path, context: &str) -> String {
    cli_command_project_root_output_dot("hash-ast", dir, context)
}

fn cli_hash_ast_file_output(path: &std::path::Path, context: &str) -> String {
    cli_command_file_output("hash-ast", path, context)
}

fn cli_hash_ir_project_root_output(dir: &std::path::Path, context: &str) -> String {
    cli_command_project_root_output("hash-ir", dir, context)
}

fn cli_hash_ir_project_root_output_dot(dir: &std::path::Path, context: &str) -> String {
    cli_command_project_root_output_dot("hash-ir", dir, context)
}

fn cli_hash_ir_file_output(path: &std::path::Path, context: &str) -> String {
    cli_command_file_output("hash-ir", path, context)
}

fn cli_hash_smc_project_root_output(dir: &std::path::Path, context: &str) -> String {
    cli_command_project_root_output("hash-smc", dir, context)
}

fn cli_hash_smc_project_root_output_dot(dir: &std::path::Path, context: &str) -> String {
    cli_command_project_root_output_dot("hash-smc", dir, context)
}

fn cli_hash_smc_file_output(path: &std::path::Path, context: &str) -> String {
    cli_command_file_output("hash-smc", path, context)
}

fn cli_disasm_artifact_output(path: &std::path::Path, context: &str) -> String {
    cli_stdout(vec!["disasm".to_string(), normalize_path(path)], context)
}

fn cli_compile_project_root_ok(dir: &std::path::Path, out_name: &str, context: &str) -> PathBuf {
    let input = normalize_path(dir);
    let out = dir.join(out_name);
    let out_arg = normalize_path(&out);
    cli_ok(
        vec![
            "compile".to_string(),
            input,
            "-o".to_string(),
            out_arg.clone(),
        ],
        context,
    );
    assert!(out.is_file(), "{context} did not write requested output");
    cli_ok(
        vec!["verify".to_string(), out_arg.clone()],
        &format!("{context} produced unverifiable SemCode"),
    );
    cli_ok(
        vec!["run-smc".to_string(), out_arg],
        &format!("{context} run-smc failed"),
    );
    out
}

fn collect_semcode_artifacts(dir: &std::path::Path, artifacts: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("scan .smc artifacts in {}: {err}", dir.display()))
    {
        let entry =
            entry.unwrap_or_else(|err| panic!("read dir entry in {}: {err}", dir.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_semcode_artifacts(&path, artifacts);
        } else if path.extension().is_some_and(|ext| ext == "smc") {
            artifacts.push(path);
        }
    }
}

fn assert_no_semcode_artifacts(dir: &std::path::Path, context: &str) {
    let mut artifacts = Vec::new();
    collect_semcode_artifacts(dir, &mut artifacts);
    assert!(
        artifacts.is_empty(),
        "{context} unexpectedly created .smc artifacts: {:?}",
        artifacts
    );
}

fn assert_artifact_only_command_rejects_project_root_input(
    command: &str,
    dir: &std::path::Path,
    context: &str,
) {
    let root_input = normalize_path(dir);

    for (label, input, cwd) in [
        ("absolute project root", root_input.as_str(), None),
        ("dot from project root", ".", Some(dir)),
    ] {
        let output = cli_command_output(command, input, cwd, &format!("{context} ({label})"));
        assert!(
            !output.status.success(),
            "{context} ({label}) unexpectedly passed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stdout.trim().is_empty(),
            "{context} ({label}) unexpectedly wrote stdout:\n{stdout}"
        );
        assert!(
            !stderr.trim().is_empty(),
            "{context} ({label}) produced empty stderr"
        );
        assert!(
            !stderr.contains("semantic.toml"),
            "{context} ({label}) fell back to project-root manifest routing:\n{stderr}"
        );
        assert!(
            !stderr.contains("project root"),
            "{context} ({label}) leaked project-root routing markers:\n{stderr}"
        );
        assert!(
            !stderr.contains("resolved entry"),
            "{context} ({label}) leaked resolved-entry routing markers:\n{stderr}"
        );
        assert_no_semcode_artifacts(dir, &format!("{context} ({label})"));
    }
}

fn cli_compile_project_root_err_no_overwrite(
    dir: &std::path::Path,
    out_name: &str,
    context: &str,
) -> String {
    let input = normalize_path(dir);
    let out = dir.join(out_name);
    let out_arg = normalize_path(&out);
    std::fs::write(&out, "sentinel").expect("write sentinel");
    let err = cli_err(
        vec!["compile".to_string(), input, "-o".to_string(), out_arg],
        context,
    );
    let content = std::fs::read_to_string(&out).expect("read out file");
    assert_eq!(
        content, "sentinel",
        "{context} overwrote existing file on compilation failure"
    );
    err
}

fn assert_stale_artifact_protected<F>(
    dir: &std::path::Path,
    out_name: &str,
    mutate: F,
    context: &str,
) where
    F: FnOnce(&std::path::Path),
{
    let out = cli_compile_project_root_ok(dir, out_name, &format!("{context} (initial compile)"));
    let original_bytes = std::fs::read(&out).unwrap_or_else(|err| {
        panic!(
            "{context}: failed to read original bytes of {}: {err}",
            out.display()
        )
    });
    mutate(dir);
    let input = normalize_path(dir);
    let out_arg = normalize_path(&out);
    let _err = cli_err(
        vec![
            "compile".to_string(),
            input,
            "-o".to_string(),
            out_arg.clone(),
        ],
        &format!("{context} (failing compile expected)"),
    );
    assert!(out.is_file(), "{context}: out.smc was deleted");
    let current_bytes = std::fs::read(&out).unwrap_or_else(|err| {
        panic!(
            "{context}: failed to read current bytes of {}: {err}",
            out.display()
        )
    });
    assert_eq!(
        original_bytes, current_bytes,
        "{context}: out.smc was overwritten or modified on compilation failure"
    );
    cli_ok(
        vec!["verify".to_string(), out_arg.clone()],
        &format!("{context}: verification failed for original artifact after failed compile"),
    );
    cli_ok(
        vec!["run-smc".to_string(), out_arg],
        &format!("{context}: run-smc failed for original artifact after failed compile"),
    );
}

fn cli_check_dot_ok(dir: &std::path::Path, context: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_smc"))
        .arg("check")
        .arg(".")
        .current_dir(dir)
        .output()
        .unwrap_or_else(|err| panic!("{context} failed to spawn smc: {err}"));
    assert!(
        output.status.success(),
        "{context} failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn cli_run_dot_ok(dir: &std::path::Path, context: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_smc"))
        .arg("run")
        .arg(".")
        .current_dir(dir)
        .output()
        .unwrap_or_else(|err| panic!("{context} failed to spawn smc: {err}"));
    assert!(
        output.status.success(),
        "{context} failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn cli_compile_dot_ok(dir: &std::path::Path, out_name: &str, context: &str) -> PathBuf {
    let output = Command::new(env!("CARGO_BIN_EXE_smc"))
        .arg("compile")
        .arg(".")
        .arg("-o")
        .arg(out_name)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|err| panic!("{context} failed to spawn smc: {err}"));
    assert!(
        output.status.success(),
        "{context} failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let out = dir.join(out_name);
    assert!(out.is_file(), "{context} did not write requested output");
    cli_ok(
        vec!["verify".to_string(), normalize_path(&out)],
        &format!("{context} produced unverifiable SemCode"),
    );
    let output2 = Command::new(env!("CARGO_BIN_EXE_smc"))
        .arg("run-smc")
        .arg(out_name)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|err| panic!("{context} failed to spawn smc run-smc: {err}"));
    assert!(
        output2.status.success(),
        "{context} run-smc failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
        output2.status.code(),
        String::from_utf8_lossy(&output2.stdout),
        String::from_utf8_lossy(&output2.stderr)
    );
    out
}

fn cli_err(args: Vec<String>, context: &str) -> String {
    smc_cli::run(args).expect_err(context)
}

fn mk_temp_dir(prefix: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "{}_{}_{}",
        prefix,
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&base).expect("mkdir");
    base
}

fn normalize_path(path: &std::path::Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .trim_start_matches("//?/")
        .to_string()
}

fn full_cli_path(file_rel: &str) {
    let input = repo_path(file_rel);
    cli_ok(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for {input}"),
    );
    cli_ok(
        vec!["run".to_string(), input.clone()],
        &format!("smc run for {input}"),
    );

    let dir = mk_temp_dir("pcc9_project_model_acceptance");
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

fn full_project_root_check_path() {
    let dir = mk_temp_dir("pcc9_project_root_acceptance");
    let src_dir = dir.join("src");
    std::fs::create_dir_all(&src_dir).expect("mkdir src");
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package]
name = "app"

[project]
entry = "src/main.sm"
"#,
    )
    .expect("write manifest");
    std::fs::write(src_dir.join("main.sm"), "fn main() { return; }\n").expect("write entry");

    let input = normalize_path(&dir);
    cli_ok(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for project root {input}"),
    );

    let _ = std::fs::remove_dir_all(&dir);
}

fn write_semantic_toml(dir: &std::path::Path, entry: Option<&str>) {
    let manifest = match entry {
        Some(entry) => format!(
            r#"
[package]
name = "app"

[project]
entry = "{entry}"
"#
        ),
        None => r#"
[package]
name = "app"
"#
        .to_string(),
    };
    std::fs::write(dir.join("semantic.toml"), manifest).expect("write semantic.toml");
}

fn write_source(path: &std::path::Path) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir source parent");
    }
    std::fs::write(path, "fn main() { return; }\n").expect("write source");
}

fn write_package_manifest_baseline(dir: &std::path::Path) {
    std::fs::write(
        dir.join(PACKAGE_MANIFEST_FILE_NAME),
        r#"
format 1
package app
manifest_dir .
module_root src
"#,
    )
    .expect("write package manifest");
}

fn full_project_root_package_baseline_check_path() {
    let dir = mk_temp_dir("pcc9_project_root_package_acceptance");
    let src_dir = dir.join("src");
    std::fs::create_dir_all(&src_dir).expect("mkdir src");
    write_package_manifest_baseline(&dir);
    std::fs::write(src_dir.join("main.sm"), "fn main() { return; }\n").expect("write entry");

    let input = normalize_path(&dir);
    cli_ok(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for package baseline project root {input}"),
    );

    let _ = std::fs::remove_dir_all(&dir);
}

fn full_project_root_package_baseline_run_path() {
    let dir = mk_temp_dir("pcc9_project_root_package_run_acceptance");
    write_package_manifest_baseline(&dir);
    write_source(&dir.join("src").join("main.sm"));

    cli_run_project_root_ok(&dir, "smc run for package baseline project root");

    let _ = std::fs::remove_dir_all(&dir);
}

fn full_project_root_package_baseline_hash_ast_path() {
    let dir = mk_temp_dir("pcc9_project_root_package_hash_ast_acceptance");
    write_package_manifest_baseline(&dir);
    let entry = dir.join("src").join("main.sm");
    write_source(&entry);

    let root_hash =
        cli_hash_ast_project_root_output(&dir, "smc hash-ast for package baseline project root");
    let file_hash = cli_hash_ast_file_output(&entry, "smc hash-ast for resolved package entry");
    assert_eq!(
        root_hash, file_hash,
        "project-root hash must match resolved file hash"
    );
    assert_eq!(
        root_hash,
        cli_hash_ast_project_root_output(
            &dir,
            "smc hash-ast for package baseline project root second run"
        ),
        "project-root hash must be deterministic across repeated runs"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

fn full_project_root_package_baseline_compile_path() {
    let dir = mk_temp_dir("pcc9_project_root_package_compile_acceptance");
    write_package_manifest_baseline(&dir);
    write_source(&dir.join("src").join("main.sm"));

    cli_compile_project_root_ok(
        &dir,
        "package-out.smc",
        "smc compile for package project root",
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_artifact_only_commands_still_work_for_real_semcode_artifacts() {
    let dir = mk_temp_dir("pcc9_artifact_only_positive_chain");
    write_package_manifest_baseline(&dir);
    let entry = dir.join("src").join("main.sm");
    write_source(&entry);

    let out = cli_compile_project_root_ok(
        &dir,
        "artifact-chain.smc",
        "smc compile for artifact-only positive chain",
    );
    let disasm = cli_disasm_artifact_output(&out, "smc disasm for compiled artifact");
    assert!(
        !disasm.trim().is_empty(),
        "smc disasm for compiled artifact returned empty output"
    );
    assert_eq!(
        disasm,
        cli_disasm_artifact_output(&out, "smc disasm for compiled artifact second run"),
        "smc disasm for compiled artifact must be deterministic across repeated runs"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_artifact_only_disasm_rejects_project_root_inputs_without_fallback() {
    let dir = mk_temp_dir("pcc9_artifact_only_disasm_project_root");
    write_semantic_toml(&dir, Some("src/main.sm"));
    write_source(&dir.join("src").join("main.sm"));

    assert_artifact_only_command_rejects_project_root_input(
        "disasm",
        &dir,
        "smc disasm must reject project-root inputs",
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_artifact_only_verify_rejects_project_root_inputs_without_fallback() {
    let dir = mk_temp_dir("pcc9_artifact_only_verify_project_root");
    write_semantic_toml(&dir, Some("src/main.sm"));
    write_source(&dir.join("src").join("main.sm"));

    assert_artifact_only_command_rejects_project_root_input(
        "verify",
        &dir,
        "smc verify must reject project-root inputs",
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_artifact_only_run_smc_rejects_project_root_inputs_without_fallback() {
    let dir = mk_temp_dir("pcc9_artifact_only_run_smc_project_root");
    write_semantic_toml(&dir, Some("src/main.sm"));
    write_source(&dir.join("src").join("main.sm"));

    assert_artifact_only_command_rejects_project_root_input(
        "run-smc",
        &dir,
        "smc run-smc must reject project-root inputs",
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_single_file_baseline_passes_full_cli_path() {
    full_cli_path("tests/fixtures/pcc9_project_model/single_file_baseline/main.sm");
}

#[test]
fn pcc9_project_root_baseline_passes_check_entrypoint() {
    full_project_root_check_path();
}

#[test]
fn pcc9_project_root_package_baseline_still_passes_check_entrypoint() {
    full_project_root_package_baseline_check_path();
}

#[test]
fn pcc9_project_root_package_baseline_still_runs_entrypoint() {
    full_project_root_package_baseline_run_path();
}

#[test]
fn pcc9_project_root_package_baseline_still_hashes_entrypoint() {
    full_project_root_package_baseline_hash_ast_path();
}

#[test]
fn pcc9_project_root_package_baseline_still_compiles_entrypoint() {
    full_project_root_package_baseline_compile_path();
}

#[test]
fn pcc9_project_root_semantic_toml_explicit_entry_passes_check() {
    let dir = mk_temp_dir("pcc9_project_root_semantic_explicit_entry");
    write_semantic_toml(&dir, Some("src/main.sm"));
    write_source(&dir.join("src").join("main.sm"));

    cli_check_project_root_ok(&dir, "smc check for semantic.toml explicit entry");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_semantic_toml_explicit_entry_runs() {
    let dir = mk_temp_dir("pcc9_project_root_semantic_explicit_entry_run");
    write_semantic_toml(&dir, Some("src/main.sm"));
    write_source(&dir.join("src").join("main.sm"));

    cli_run_project_root_ok(&dir, "smc run for semantic.toml explicit entry");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_semantic_toml_explicit_entry_hashes() {
    let dir = mk_temp_dir("pcc9_project_root_semantic_explicit_entry_hash");
    write_semantic_toml(&dir, Some("src/main.sm"));
    let entry = dir.join("src").join("main.sm");
    write_source(&entry);

    let root_hash =
        cli_hash_ast_project_root_output(&dir, "smc hash-ast for semantic.toml explicit entry");
    let file_hash = cli_hash_ast_file_output(&entry, "smc hash-ast for explicit resolved entry");
    assert_eq!(
        root_hash, file_hash,
        "project-root hash must match resolved file hash"
    );
    assert_eq!(
        root_hash,
        cli_hash_ast_project_root_output(
            &dir,
            "smc hash-ast for semantic.toml explicit entry second run"
        ),
        "project-root hash must be deterministic across repeated runs"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_semantic_toml_explicit_entry_compiles() {
    let dir = mk_temp_dir("pcc9_project_root_semantic_explicit_entry_compile");
    write_semantic_toml(&dir, Some("src/main.sm"));
    write_source(&dir.join("src").join("main.sm"));

    cli_compile_project_root_ok(&dir, "explicit-out.smc", "smc compile for explicit entry");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_semantic_toml_default_entry_passes_check() {
    let dir = mk_temp_dir("pcc9_project_root_semantic_default_entry");
    write_semantic_toml(&dir, None);
    write_source(&dir.join("src").join("main.sm"));

    cli_check_project_root_ok(&dir, "smc check for semantic.toml default entry");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_semantic_toml_default_entry_runs() {
    let dir = mk_temp_dir("pcc9_project_root_semantic_default_entry_run");
    write_semantic_toml(&dir, None);
    write_source(&dir.join("src").join("main.sm"));

    cli_run_project_root_ok(&dir, "smc run for semantic.toml default entry");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_semantic_toml_default_entry_hashes() {
    let dir = mk_temp_dir("pcc9_project_root_semantic_default_entry_hash");
    write_semantic_toml(&dir, None);
    let entry = dir.join("src").join("main.sm");
    write_source(&entry);

    let root_hash =
        cli_hash_ast_project_root_output(&dir, "smc hash-ast for semantic.toml default entry");
    let file_hash = cli_hash_ast_file_output(&entry, "smc hash-ast for default resolved entry");
    assert_eq!(
        root_hash, file_hash,
        "project-root hash must match resolved file hash"
    );
    assert_eq!(
        root_hash,
        cli_hash_ast_project_root_output(
            &dir,
            "smc hash-ast for semantic.toml default entry second run"
        ),
        "project-root hash must be deterministic across repeated runs"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_semantic_toml_default_entry_compiles() {
    let dir = mk_temp_dir("pcc9_project_root_semantic_default_entry_compile");
    write_semantic_toml(&dir, None);
    write_source(&dir.join("src").join("main.sm"));

    cli_compile_project_root_ok(&dir, "default-out.smc", "smc compile for default entry");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_semantic_toml_nested_entry_passes_check() {
    let dir = mk_temp_dir("pcc9_project_root_semantic_nested_entry");
    write_semantic_toml(&dir, Some("examples/main.sm"));
    write_source(&dir.join("examples").join("main.sm"));

    cli_check_project_root_ok(&dir, "smc check for semantic.toml nested entry");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_semantic_toml_nested_entry_runs() {
    let dir = mk_temp_dir("pcc9_project_root_semantic_nested_entry_run");
    write_semantic_toml(&dir, Some("examples/main.sm"));
    write_source(&dir.join("examples").join("main.sm"));

    cli_run_project_root_ok(&dir, "smc run for semantic.toml nested entry");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_semantic_toml_nested_entry_hashes() {
    let dir = mk_temp_dir("pcc9_project_root_semantic_nested_entry_hash");
    write_semantic_toml(&dir, Some("examples/main.sm"));
    let entry = dir.join("examples").join("main.sm");
    write_source(&entry);

    let root_hash =
        cli_hash_ast_project_root_output(&dir, "smc hash-ast for semantic.toml nested entry");
    let file_hash = cli_hash_ast_file_output(&entry, "smc hash-ast for nested resolved entry");
    assert_eq!(
        root_hash, file_hash,
        "project-root hash must match resolved file hash"
    );
    assert_eq!(
        root_hash,
        cli_hash_ast_project_root_output(
            &dir,
            "smc hash-ast for semantic.toml nested entry second run"
        ),
        "project-root hash must be deterministic across repeated runs"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_semantic_toml_nested_entry_compiles() {
    let dir = mk_temp_dir("pcc9_project_root_semantic_nested_entry_compile");
    write_semantic_toml(&dir, Some("examples/main.sm"));
    write_source(&dir.join("examples").join("main.sm"));

    cli_compile_project_root_ok(&dir, "nested-out.smc", "smc compile for nested entry");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_semantic_toml_is_preferred_over_package_manifest() {
    let dir = mk_temp_dir("pcc9_project_root_semantic_preferred");
    write_semantic_toml(&dir, Some("examples/main.sm"));
    write_package_manifest_baseline(&dir);
    write_source(&dir.join("examples").join("main.sm"));

    cli_check_project_root_ok(
        &dir,
        "smc check prefers semantic.toml over Semantic.package",
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_semantic_toml_is_preferred_over_package_manifest_for_run() {
    let dir = mk_temp_dir("pcc9_project_root_semantic_preferred_run");
    write_semantic_toml(&dir, Some("examples/main.sm"));
    write_package_manifest_baseline(&dir);
    write_source(&dir.join("examples").join("main.sm"));

    cli_run_project_root_ok(&dir, "smc run prefers semantic.toml over Semantic.package");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_semantic_toml_is_preferred_over_package_manifest_for_hash_ast() {
    let dir = mk_temp_dir("pcc9_project_root_semantic_preferred_hash");
    write_semantic_toml(&dir, Some("examples/main.sm"));
    write_package_manifest_baseline(&dir);
    let entry = dir.join("examples").join("main.sm");
    write_source(&entry);

    let root_hash = cli_hash_ast_project_root_output(
        &dir,
        "smc hash-ast prefers semantic.toml over Semantic.package",
    );
    let file_hash = cli_hash_ast_file_output(&entry, "smc hash-ast for preferred resolved entry");
    assert_eq!(
        root_hash, file_hash,
        "project-root hash must match resolved file hash"
    );
    assert_eq!(
        root_hash,
        cli_hash_ast_project_root_output(
            &dir,
            "smc hash-ast prefers semantic.toml over Semantic.package second run"
        ),
        "project-root hash must be deterministic across repeated runs"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_semantic_toml_is_preferred_over_package_manifest_for_compile() {
    let dir = mk_temp_dir("pcc9_project_root_semantic_preferred_compile");
    write_semantic_toml(&dir, Some("examples/main.sm"));
    write_package_manifest_baseline(&dir);
    write_source(&dir.join("examples").join("main.sm"));

    cli_compile_project_root_ok(
        &dir,
        "preferred-out.smc",
        "smc compile prefers semantic.toml over Semantic.package",
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_check_dot_matches_absolute_project_root() {
    let dir = mk_temp_dir("pcc9_project_root_check_dot");
    write_semantic_toml(&dir, Some("src/main.sm"));
    write_source(&dir.join("src").join("main.sm"));

    cli_check_project_root_ok(&dir, "smc check for absolute project root");
    cli_check_dot_ok(&dir, "smc check dot from project root");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_run_dot_matches_absolute_project_root() {
    let dir = mk_temp_dir("pcc9_project_root_run_dot");
    write_semantic_toml(&dir, Some("src/main.sm"));
    write_source(&dir.join("src").join("main.sm"));

    cli_run_project_root_ok(&dir, "smc run for absolute project root");
    cli_run_dot_ok(&dir, "smc run dot from project root");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_hash_ast_dot_matches_absolute_project_root() {
    let dir = mk_temp_dir("pcc9_project_root_hash_ast_dot");
    write_semantic_toml(&dir, Some("src/main.sm"));
    let entry = dir.join("src").join("main.sm");
    write_source(&entry);

    let root_hash =
        cli_hash_ast_project_root_output(&dir, "smc hash-ast for absolute project root");
    let dot_hash = cli_hash_ast_project_root_output_dot(&dir, "smc hash-ast dot from project root");
    let file_hash = cli_hash_ast_file_output(&entry, "smc hash-ast for absolute resolved entry");
    assert_eq!(
        root_hash, file_hash,
        "project-root hash must match resolved file hash"
    );
    assert_eq!(
        dot_hash, file_hash,
        "smc hash-ast . must match resolved file hash"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_compile_dot_writes_requested_output() {
    let dir = mk_temp_dir("pcc9_project_root_compile_dot");
    write_semantic_toml(&dir, Some("src/main.sm"));
    write_source(&dir.join("src").join("main.sm"));

    cli_compile_project_root_ok(
        &dir,
        "absolute-out.smc",
        "smc compile absolute project root",
    );
    cli_compile_dot_ok(&dir, "dot-out.smc", "smc compile dot from project root");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_hash_ast_rejects_invalid_semantic_toml_without_fallback() {
    let dir = mk_temp_dir("pcc9_project_root_hash_ast_invalid_syntax");
    write_package_manifest_baseline(&dir);
    write_source(&dir.join("src").join("main.sm"));
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package
name = "app"
"#,
    )
    .expect("write manifest");

    let input = normalize_path(&dir);
    let err = cli_err(
        vec!["hash-ast".to_string(), input.clone()],
        &format!("smc hash-ast for invalid manifest project root {input}"),
    );
    assert!(err.contains("semantic.toml"), "{err}");
    assert!(err.contains("malformed"), "{err}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_semantic_toml_rejects_invalid_syntax() {
    let dir = mk_temp_dir("pcc9_project_root_invalid_syntax");
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package
name = "app"
"#,
    )
    .expect("write manifest");

    let input = normalize_path(&dir);
    let err = cli_err(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for invalid manifest project root {input}"),
    );
    assert!(err.contains("semantic.toml"), "{err}");
    assert!(err.contains("malformed"), "{err}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_hash_ast_rejects_missing_entry_file_without_fallback() {
    let dir = mk_temp_dir("pcc9_project_root_hash_ast_missing_entry_file");
    write_package_manifest_baseline(&dir);
    write_source(&dir.join("src").join("main.sm"));
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package]
name = "app"

[project]
entry = "examples/missing.sm"
"#,
    )
    .expect("write manifest");

    let input = normalize_path(&dir);
    let err = cli_err(
        vec!["hash-ast".to_string(), input.clone()],
        &format!("smc hash-ast for missing entry file project root {input}"),
    );
    assert!(err.contains("semantic.toml"), "{err}");
    assert!(err.contains("missing file"), "{err}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_run_rejects_invalid_semantic_toml_without_fallback() {
    let dir = mk_temp_dir("pcc9_project_root_run_invalid_syntax");
    write_package_manifest_baseline(&dir);
    write_source(&dir.join("src").join("main.sm"));
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package
name = "app"
"#,
    )
    .expect("write manifest");

    let input = normalize_path(&dir);
    let err = cli_err(
        vec!["run".to_string(), input.clone()],
        &format!("smc run for invalid manifest project root {input}"),
    );
    assert!(err.contains("semantic.toml"), "{err}");
    assert!(err.contains("malformed"), "{err}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_compile_rejects_invalid_semantic_toml_without_fallback() {
    let dir = mk_temp_dir("pcc9_project_root_compile_invalid_syntax");
    write_package_manifest_baseline(&dir);
    write_source(&dir.join("src").join("main.sm"));
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package
name = "app"
"#,
    )
    .expect("write manifest");

    let err = cli_compile_project_root_err_no_overwrite(
        &dir,
        "invalid-out.smc",
        "smc compile for invalid manifest project root",
    );
    assert!(err.contains("semantic.toml"), "{err}");
    assert!(err.contains("malformed"), "{err}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_hash_ast_rejects_path_escape_without_fallback() {
    let dir = mk_temp_dir("pcc9_project_root_hash_ast_path_escape");
    write_package_manifest_baseline(&dir);
    write_source(&dir.join("src").join("main.sm"));
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package]
name = "app"

[project]
entry = "../escape.sm"
"#,
    )
    .expect("write manifest");

    let input = normalize_path(&dir);
    let err = cli_err(
        vec!["hash-ast".to_string(), input.clone()],
        &format!("smc hash-ast for escaped entry project root {input}"),
    );
    assert!(err.contains("semantic.toml"), "{err}");
    assert!(err.contains("must not escape the project root"), "{err}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_semantic_toml_rejects_missing_package_name() {
    let dir = mk_temp_dir("pcc9_project_root_missing_package_name");
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package]

[project]
entry = "src/main.sm"
"#,
    )
    .expect("write manifest");

    let input = normalize_path(&dir);
    let err = cli_err(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for missing package name project root {input}"),
    );
    assert!(err.contains("semantic.toml"), "{err}");
    assert!(err.contains("missing required [package].name"), "{err}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_hash_ast_rejects_missing_package_name_without_fallback() {
    let dir = mk_temp_dir("pcc9_project_root_hash_ast_missing_package_name");
    write_package_manifest_baseline(&dir);
    write_source(&dir.join("src").join("main.sm"));
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package]

[project]
entry = "src/main.sm"
"#,
    )
    .expect("write manifest");

    let input = normalize_path(&dir);
    let err = cli_err(
        vec!["hash-ast".to_string(), input.clone()],
        &format!("smc hash-ast for missing package name project root {input}"),
    );
    assert!(err.contains("semantic.toml"), "{err}");
    assert!(err.contains("missing required [package].name"), "{err}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_run_rejects_missing_entry_file_without_fallback() {
    let dir = mk_temp_dir("pcc9_project_root_run_missing_entry_file");
    write_package_manifest_baseline(&dir);
    write_source(&dir.join("src").join("main.sm"));
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package]
name = "app"

[project]
entry = "examples/missing.sm"
"#,
    )
    .expect("write manifest");

    let input = normalize_path(&dir);
    let err = cli_err(
        vec!["run".to_string(), input.clone()],
        &format!("smc run for missing entry file project root {input}"),
    );
    assert!(err.contains("semantic.toml"), "{err}");
    assert!(err.contains("missing file"), "{err}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_compile_rejects_missing_entry_file_without_fallback() {
    let dir = mk_temp_dir("pcc9_project_root_compile_missing_entry_file");
    write_package_manifest_baseline(&dir);
    write_source(&dir.join("src").join("main.sm"));
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package]
name = "app"

[project]
entry = "examples/missing.sm"
"#,
    )
    .expect("write manifest");

    let err = cli_compile_project_root_err_no_overwrite(
        &dir,
        "missing-out.smc",
        "smc compile for missing entry file project root",
    );
    assert!(err.contains("semantic.toml"), "{err}");
    assert!(err.contains("missing file"), "{err}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_semantic_toml_rejects_empty_package_name() {
    let dir = mk_temp_dir("pcc9_project_root_empty_package_name");
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package]
name = ""

[project]
entry = "src/main.sm"
"#,
    )
    .expect("write manifest");

    let input = normalize_path(&dir);
    let err = cli_err(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for empty package name project root {input}"),
    );
    assert!(err.contains("semantic.toml"), "{err}");
    assert!(err.contains("empty [package].name"), "{err}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_hash_ast_rejects_empty_entry_without_fallback() {
    let dir = mk_temp_dir("pcc9_project_root_hash_ast_empty_entry");
    write_package_manifest_baseline(&dir);
    write_source(&dir.join("src").join("main.sm"));
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package]
name = "app"

[project]
entry = ""
"#,
    )
    .expect("write manifest");

    let input = normalize_path(&dir);
    let err = cli_err(
        vec!["hash-ast".to_string(), input.clone()],
        &format!("smc hash-ast for empty entry project root {input}"),
    );
    assert!(err.contains("semantic.toml"), "{err}");
    assert!(err.contains("empty [project].entry"), "{err}");

    let _ = std::fs::remove_dir_all(&dir);
}

fn assert_hash_ir_project_root_matches_entry(
    dir: &std::path::Path,
    entry: &std::path::Path,
    context: &str,
) {
    let root_hash = cli_hash_ir_project_root_output(dir, &format!("{context} (project root)"));
    let file_hash = cli_hash_ir_file_output(entry, &format!("{context} (resolved file)"));
    assert_eq!(
        root_hash, file_hash,
        "{context}: project-root hash must match resolved file hash"
    );
    assert_eq!(
        root_hash,
        cli_hash_ir_project_root_output(dir, &format!("{context} (second run)")),
        "{context}: project-root hash must be deterministic across repeated runs"
    );
}

fn assert_hash_smc_project_root_matches_entry(
    dir: &std::path::Path,
    entry: &std::path::Path,
    context: &str,
) {
    let root_hash = cli_hash_smc_project_root_output(dir, &format!("{context} (project root)"));
    let file_hash = cli_hash_smc_file_output(entry, &format!("{context} (resolved file)"));
    assert_eq!(
        root_hash, file_hash,
        "{context}: project-root hash must match resolved file hash"
    );
    assert_eq!(
        root_hash,
        cli_hash_smc_project_root_output(dir, &format!("{context} (second run)")),
        "{context}: project-root hash must be deterministic across repeated runs"
    );
}

#[test]
fn pcc9_project_root_hash_ir_semantic_toml_explicit_entry() {
    let dir = mk_temp_dir("pcc9_project_root_hash_ir_explicit_entry");
    write_semantic_toml(&dir, Some("src/main.sm"));
    let entry = dir.join("src").join("main.sm");
    write_source(&entry);

    assert_hash_ir_project_root_matches_entry(
        &dir,
        &entry,
        "smc hash-ir with semantic.toml explicit entry",
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_hash_ir_semantic_toml_default_entry() {
    let dir = mk_temp_dir("pcc9_project_root_hash_ir_default_entry");
    write_semantic_toml(&dir, None);
    let entry = dir.join("src").join("main.sm");
    write_source(&entry);

    assert_hash_ir_project_root_matches_entry(
        &dir,
        &entry,
        "smc hash-ir with semantic.toml default entry",
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_hash_ir_semantic_toml_nested_entry() {
    let dir = mk_temp_dir("pcc9_project_root_hash_ir_nested_entry");
    write_semantic_toml(&dir, Some("examples/main.sm"));
    let entry = dir.join("examples").join("main.sm");
    write_source(&entry);

    assert_hash_ir_project_root_matches_entry(
        &dir,
        &entry,
        "smc hash-ir with semantic.toml nested entry",
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_hash_ir_prefers_semantic_toml_over_package_manifest() {
    let dir = mk_temp_dir("pcc9_project_root_hash_ir_preferred");
    write_semantic_toml(&dir, Some("examples/main.sm"));
    write_package_manifest_baseline(&dir);
    let semantic_entry = dir.join("examples").join("main.sm");
    let package_entry = dir.join("src").join("main.sm");
    std::fs::create_dir_all(semantic_entry.parent().unwrap()).expect("mkdir semantic entry");
    std::fs::create_dir_all(package_entry.parent().unwrap()).expect("mkdir package entry");
    std::fs::write(
        &semantic_entry,
        "fn main() { return; }\nfn toml_main() { return; }\n",
    )
    .expect("write semantic entry");
    std::fs::write(
        &package_entry,
        "fn main() { return; }\nfn pkg_main() { return; }\n",
    )
    .expect("write package entry");

    assert_hash_ir_project_root_matches_entry(
        &dir,
        &semantic_entry,
        "smc hash-ir prefers semantic.toml over Semantic.package",
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_hash_ir_package_baseline_fallback() {
    let dir = mk_temp_dir("pcc9_project_root_hash_ir_package_baseline");
    write_package_manifest_baseline(&dir);
    let entry = dir.join("src").join("main.sm");
    write_source(&entry);

    assert_hash_ir_project_root_matches_entry(
        &dir,
        &entry,
        "smc hash-ir fallback to Semantic.package",
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_hash_ir_dot_works_from_root() {
    let dir = mk_temp_dir("pcc9_project_root_hash_ir_dot");
    write_semantic_toml(&dir, Some("src/main.sm"));
    let entry = dir.join("src").join("main.sm");
    write_source(&entry);

    let root_hash = cli_hash_ir_project_root_output(&dir, "smc hash-ir for absolute project root");
    let dot_hash = cli_hash_ir_project_root_output_dot(&dir, "smc hash-ir dot from project root");
    let file_hash = cli_hash_ir_file_output(&entry, "smc hash-ir for resolved entry");
    assert_eq!(
        root_hash, file_hash,
        "project-root hash must match resolved file hash"
    );
    assert_eq!(
        dot_hash, file_hash,
        "smc hash-ir . must match resolved file hash"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_hash_ir_rejects_invalid_semantic_toml_without_fallback() {
    let dir = mk_temp_dir("pcc9_project_root_hash_ir_invalid_syntax");
    write_package_manifest_baseline(&dir);
    write_source(&dir.join("src").join("main.sm"));
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package
name = "app"
"#,
    )
    .expect("write manifest");

    let input = normalize_path(&dir);
    let err = cli_err(
        vec!["hash-ir".to_string(), input.clone()],
        &format!("smc hash-ir for invalid manifest project root {input}"),
    );
    assert!(err.contains("semantic.toml"), "{err}");
    assert!(err.contains("malformed"), "{err}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_hash_ir_rejects_missing_entry_file_without_fallback() {
    let dir = mk_temp_dir("pcc9_project_root_hash_ir_missing_entry_file");
    write_package_manifest_baseline(&dir);
    write_source(&dir.join("src").join("main.sm"));
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package]
name = "app"

[project]
entry = "examples/missing.sm"
"#,
    )
    .expect("write manifest");

    let input = normalize_path(&dir);
    let err = cli_err(
        vec!["hash-ir".to_string(), input.clone()],
        &format!("smc hash-ir for missing entry file project root {input}"),
    );
    assert!(err.contains("semantic.toml"), "{err}");
    assert!(err.contains("missing file"), "{err}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_hash_ir_rejects_path_escape_without_fallback() {
    let dir = mk_temp_dir("pcc9_project_root_hash_ir_path_escape");
    write_package_manifest_baseline(&dir);
    write_source(&dir.join("src").join("main.sm"));
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package]
name = "app"

[project]
entry = "../escape.sm"
"#,
    )
    .expect("write manifest");

    let input = normalize_path(&dir);
    let err = cli_err(
        vec!["hash-ir".to_string(), input.clone()],
        &format!("smc hash-ir for escaped entry project root {input}"),
    );
    assert!(err.contains("semantic.toml"), "{err}");
    assert!(err.contains("must not escape the project root"), "{err}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_hash_smc_semantic_toml_explicit_entry() {
    let dir = mk_temp_dir("pcc9_project_root_hash_smc_explicit_entry");
    write_semantic_toml(&dir, Some("src/main.sm"));
    let entry = dir.join("src").join("main.sm");
    write_source(&entry);

    assert_hash_smc_project_root_matches_entry(
        &dir,
        &entry,
        "smc hash-smc with semantic.toml explicit entry",
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_hash_smc_semantic_toml_default_entry() {
    let dir = mk_temp_dir("pcc9_project_root_hash_smc_default_entry");
    write_semantic_toml(&dir, None);
    let entry = dir.join("src").join("main.sm");
    write_source(&entry);

    assert_hash_smc_project_root_matches_entry(
        &dir,
        &entry,
        "smc hash-smc with semantic.toml default entry",
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_hash_smc_semantic_toml_nested_entry() {
    let dir = mk_temp_dir("pcc9_project_root_hash_smc_nested_entry");
    write_semantic_toml(&dir, Some("examples/main.sm"));
    let entry = dir.join("examples").join("main.sm");
    write_source(&entry);

    assert_hash_smc_project_root_matches_entry(
        &dir,
        &entry,
        "smc hash-smc with semantic.toml nested entry",
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_hash_smc_prefers_semantic_toml_over_package_manifest() {
    let dir = mk_temp_dir("pcc9_project_root_hash_smc_preferred");
    write_semantic_toml(&dir, Some("examples/main.sm"));
    write_package_manifest_baseline(&dir);
    let semantic_entry = dir.join("examples").join("main.sm");
    let package_entry = dir.join("src").join("main.sm");
    std::fs::create_dir_all(semantic_entry.parent().unwrap()).expect("mkdir semantic entry");
    std::fs::create_dir_all(package_entry.parent().unwrap()).expect("mkdir package entry");
    std::fs::write(
        &semantic_entry,
        "fn main() { return; }\nfn toml_main() { return; }\n",
    )
    .expect("write semantic entry");
    std::fs::write(
        &package_entry,
        "fn main() { return; }\nfn pkg_main() { return; }\n",
    )
    .expect("write package entry");

    assert_hash_smc_project_root_matches_entry(
        &dir,
        &semantic_entry,
        "smc hash-smc prefers semantic.toml over Semantic.package",
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_hash_smc_package_baseline_fallback() {
    let dir = mk_temp_dir("pcc9_project_root_hash_smc_package_baseline");
    write_package_manifest_baseline(&dir);
    let entry = dir.join("src").join("main.sm");
    write_source(&entry);

    assert_hash_smc_project_root_matches_entry(
        &dir,
        &entry,
        "smc hash-smc fallback to Semantic.package",
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_hash_smc_dot_works_from_root() {
    let dir = mk_temp_dir("pcc9_project_root_hash_smc_dot");
    write_semantic_toml(&dir, Some("src/main.sm"));
    let entry = dir.join("src").join("main.sm");
    write_source(&entry);

    let root_hash =
        cli_hash_smc_project_root_output(&dir, "smc hash-smc for absolute project root");
    let dot_hash = cli_hash_smc_project_root_output_dot(&dir, "smc hash-smc dot from project root");
    let file_hash = cli_hash_smc_file_output(&entry, "smc hash-smc for resolved entry");
    assert_eq!(
        root_hash, file_hash,
        "project-root hash must match resolved file hash"
    );
    assert_eq!(
        dot_hash, file_hash,
        "smc hash-smc . must match resolved file hash"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_hash_smc_rejects_invalid_semantic_toml_without_fallback() {
    let dir = mk_temp_dir("pcc9_project_root_hash_smc_invalid_syntax");
    write_package_manifest_baseline(&dir);
    write_source(&dir.join("src").join("main.sm"));
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package
name = "app"
"#,
    )
    .expect("write manifest");

    let input = normalize_path(&dir);
    let err = cli_err(
        vec!["hash-smc".to_string(), input.clone()],
        &format!("smc hash-smc for invalid manifest project root {input}"),
    );
    assert!(err.contains("semantic.toml"), "{err}");
    assert!(err.contains("malformed"), "{err}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_hash_smc_rejects_missing_entry_file_without_fallback() {
    let dir = mk_temp_dir("pcc9_project_root_hash_smc_missing_entry_file");
    write_package_manifest_baseline(&dir);
    write_source(&dir.join("src").join("main.sm"));
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package]
name = "app"

[project]
entry = "examples/missing.sm"
"#,
    )
    .expect("write manifest");

    let input = normalize_path(&dir);
    let err = cli_err(
        vec!["hash-smc".to_string(), input.clone()],
        &format!("smc hash-smc for missing entry file project root {input}"),
    );
    assert!(err.contains("semantic.toml"), "{err}");
    assert!(err.contains("missing file"), "{err}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_hash_smc_rejects_path_escape_without_fallback() {
    let dir = mk_temp_dir("pcc9_project_root_hash_smc_path_escape");
    write_package_manifest_baseline(&dir);
    write_source(&dir.join("src").join("main.sm"));
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package]
name = "app"

[project]
entry = "../escape.sm"
"#,
    )
    .expect("write manifest");

    let input = normalize_path(&dir);
    let err = cli_err(
        vec!["hash-smc".to_string(), input.clone()],
        &format!("smc hash-smc for escaped entry project root {input}"),
    );
    assert!(err.contains("semantic.toml"), "{err}");
    assert!(err.contains("must not escape the project root"), "{err}");

    let _ = std::fs::remove_dir_all(&dir);
}
#[test]
fn pcc9_project_root_run_rejects_path_escape_without_fallback() {
    let dir = mk_temp_dir("pcc9_project_root_run_path_escape");
    write_package_manifest_baseline(&dir);
    write_source(&dir.join("src").join("main.sm"));
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package]
name = "app"

[project]
entry = "../escape.sm"
"#,
    )
    .expect("write manifest");

    let input = normalize_path(&dir);
    let err = cli_err(
        vec!["run".to_string(), input.clone()],
        &format!("smc run for escaped entry project root {input}"),
    );
    assert!(err.contains("semantic.toml"), "{err}");
    assert!(err.contains("must not escape the project root"), "{err}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_compile_rejects_path_escape_without_fallback() {
    let dir = mk_temp_dir("pcc9_project_root_compile_path_escape");
    write_package_manifest_baseline(&dir);
    write_source(&dir.join("src").join("main.sm"));
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package]
name = "app"

[project]
entry = "../escape.sm"
"#,
    )
    .expect("write manifest");

    let err = cli_compile_project_root_err_no_overwrite(
        &dir,
        "escape-out.smc",
        "smc compile for escaped entry project root",
    );
    assert!(err.contains("semantic.toml"), "{err}");
    assert!(err.contains("must not escape the project root"), "{err}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_semantic_toml_rejects_empty_entry() {
    let dir = mk_temp_dir("pcc9_project_root_empty_entry");
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package]
name = "app"

[project]
entry = ""
"#,
    )
    .expect("write manifest");

    let input = normalize_path(&dir);
    let err = cli_err(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for empty entry project root {input}"),
    );
    assert!(err.contains("semantic.toml"), "{err}");
    assert!(err.contains("empty [project].entry"), "{err}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_semantic_toml_rejects_missing_entry_file() {
    let dir = mk_temp_dir("pcc9_project_root_missing_entry_file");
    std::fs::create_dir_all(dir.join("src")).expect("mkdir src");
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package]
name = "app"

[project]
entry = "src/missing.sm"
"#,
    )
    .expect("write manifest");

    let input = normalize_path(&dir);
    let err = cli_err(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for missing entry file project root {input}"),
    );
    assert!(err.contains("semantic.toml"), "{err}");
    assert!(err.contains("missing file"), "{err}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_project_root_semantic_toml_rejects_path_escape() {
    let dir = mk_temp_dir("pcc9_project_root_path_escape");
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package]
name = "app"

[project]
entry = "../escape.sm"
"#,
    )
    .expect("write manifest");

    let input = normalize_path(&dir);
    let err = cli_err(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for escaped entry project root {input}"),
    );
    assert!(err.contains("semantic.toml"), "{err}");
    assert!(err.contains("must not escape the project root"), "{err}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_package_manifest_baseline_parses_and_validates() {
    let source = read_fixture("package_manifest_minimal/Semantic.package");
    let manifest = parse_package_manifest_baseline(&source).expect("parse minimal manifest");
    assert_eq!(manifest.package.name, "app");
    assert_eq!(manifest.package.root.manifest_dir, ".");
    assert_eq!(manifest.package.root.module_root, "src");
    assert!(manifest.dependencies.is_empty());
    validate_package_manifest_baseline(&manifest).expect("validate minimal manifest");
}

#[test]
fn pcc9_package_manifest_local_dependency_inventory_is_deterministic() {
    let source = read_fixture("package_manifest_local_dep/Semantic.package");
    let manifest = parse_package_manifest_baseline(&source).expect("parse manifest with dep");
    validate_package_manifest_baseline(&manifest).expect("validate manifest with dep");
    assert_eq!(manifest.package.name, "app");
    assert_eq!(manifest.dependencies.len(), 1);
    assert_eq!(manifest.dependencies[0].alias, "math");
    assert_eq!(manifest.dependencies[0].package_name, "math");
    match &manifest.dependencies[0].source {
        PackageDependencySource::LocalPath { path } => {
            assert_eq!(path, "../math");
        }
    }
}

#[test]
fn pcc9_package_manifest_entry_admission_accepts_entry_under_module_root() {
    let dir = mk_temp_dir("pcc9_entry_admission");
    let src_dir = dir.join("src");
    std::fs::create_dir_all(src_dir.join("nested")).expect("mkdir src");
    std::fs::write(
        dir.join(PACKAGE_MANIFEST_FILE_NAME),
        read_fixture("package_manifest_entry_admission/Semantic.package"),
    )
    .expect("write manifest");
    let entry = src_dir.join("nested").join("main.sm");
    std::fs::write(&entry, "fn main() { return; }").expect("write entry");

    let admitted = admit_package_entry_module(&entry)
        .expect("admit entry")
        .expect("manifest must exist");
    assert_eq!(admitted.package_name, "app");
    assert!(admitted.manifest_path.ends_with("/Semantic.package"));
    assert_eq!(admitted.module_path, "nested/main.sm");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_package_manifest_import_resolution_resolves_local_dependency_alias() {
    let dir = mk_temp_dir("pcc9_import_resolution");
    let app_src = dir.join("app").join("src");
    let math_src = dir.join("math").join("src");
    std::fs::create_dir_all(&app_src).expect("mkdir app src");
    std::fs::create_dir_all(&math_src).expect("mkdir math src");

    std::fs::write(
        dir.join("app").join(PACKAGE_MANIFEST_FILE_NAME),
        read_fixture("package_manifest_import_resolution/Semantic.package"),
    )
    .expect("write app manifest");
    std::fs::write(
        dir.join("math").join(PACKAGE_MANIFEST_FILE_NAME),
        r#"
format 1
package math
manifest_dir .
module_root src
"#,
    )
    .expect("write math manifest");

    let importer = app_src.join("main.sm");
    let dep = math_src.join("core.sm");
    std::fs::write(
        &importer,
        "Import \"math::core.sm\"\nfn main() { return; }\n",
    )
    .expect("write importer");
    std::fs::write(&dep, "fn core() { return; }\n").expect("write dep");

    let resolved = resolve_package_import_path(&importer, "math::core.sm").expect("resolve");
    assert_eq!(normalize_path(&resolved), normalize_path(&dep));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_stale_artifact_protected_when_semantic_toml_becomes_malformed() {
    let dir = mk_temp_dir("pcc9_stale_malformed");
    write_semantic_toml(&dir, Some("src/main.sm"));
    write_source(&dir.join("src").join("main.sm"));

    assert_stale_artifact_protected(
        &dir,
        "out.smc",
        |d| {
            std::fs::write(
                d.join("semantic.toml"),
                r#"
[package
name = "app"
"#,
            )
            .expect("write malformed toml");
        },
        "malformed semantic.toml protection",
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_stale_artifact_protected_when_package_name_missing() {
    let dir = mk_temp_dir("pcc9_stale_missing_package_name");
    write_semantic_toml(&dir, Some("src/main.sm"));
    write_source(&dir.join("src").join("main.sm"));

    assert_stale_artifact_protected(
        &dir,
        "out.smc",
        |d| {
            std::fs::write(
                d.join("semantic.toml"),
                r#"
[package]

[project]
entry = "src/main.sm"
"#,
            )
            .expect("write missing package name toml");
        },
        "missing package name protection",
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_stale_artifact_protected_when_project_entry_empty() {
    let dir = mk_temp_dir("pcc9_stale_empty_entry");
    write_semantic_toml(&dir, Some("src/main.sm"));
    write_source(&dir.join("src").join("main.sm"));

    assert_stale_artifact_protected(
        &dir,
        "out.smc",
        |d| {
            std::fs::write(
                d.join("semantic.toml"),
                r#"
[package]
name = "app"

[project]
entry = ""
"#,
            )
            .expect("write empty entry toml");
        },
        "empty entry protection",
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_stale_artifact_protected_when_project_entry_missing_file() {
    let dir = mk_temp_dir("pcc9_stale_missing_file");
    write_semantic_toml(&dir, Some("src/main.sm"));
    write_source(&dir.join("src").join("main.sm"));

    assert_stale_artifact_protected(
        &dir,
        "out.smc",
        |d| {
            std::fs::write(
                d.join("semantic.toml"),
                r#"
[package]
name = "app"

[project]
entry = "src/missing.sm"
"#,
            )
            .expect("write missing entry file toml");
        },
        "missing entry file protection",
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_stale_artifact_protected_when_project_entry_path_escapes() {
    let dir = mk_temp_dir("pcc9_stale_path_escape");
    write_semantic_toml(&dir, Some("src/main.sm"));
    write_source(&dir.join("src").join("main.sm"));

    assert_stale_artifact_protected(
        &dir,
        "out.smc",
        |d| {
            std::fs::write(
                d.join("semantic.toml"),
                r#"
[package]
name = "app"

[project]
entry = "../escape.sm"
"#,
            )
            .expect("write path escape toml");
        },
        "path escape protection",
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_stale_artifact_protected_when_semantic_toml_preferred_over_package_manifest_and_semantic_toml_invalid(
) {
    let dir = mk_temp_dir("pcc9_stale_preferred_invalid");
    write_semantic_toml(&dir, Some("examples/main.sm"));
    write_package_manifest_baseline(&dir);
    write_source(&dir.join("examples").join("main.sm"));
    write_source(&dir.join("src").join("main.sm"));

    assert_stale_artifact_protected(
        &dir,
        "out.smc",
        |d| {
            std::fs::write(
                d.join("semantic.toml"),
                r#"
[package]
name = "app"

[project]
entry = "../escape.sm"
"#,
            )
            .expect("write invalid toml");
        },
        "semantic.toml priority invalid protection",
    );

    let _ = std::fs::remove_dir_all(&dir);
}

fn cli_dump_ast_ok(input_path: &std::path::Path, context: &str) -> String {
    let input = normalize_path(input_path);
    let output = Command::new(env!("CARGO_BIN_EXE_smc"))
        .arg("dump-ast")
        .arg(input)
        .output()
        .unwrap_or_else(|err| panic!("{context} failed to spawn smc: {err}"));
    assert!(
        output.status.success(),
        "{context} failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn cli_dump_ast_dot_ok(dir: &std::path::Path, context: &str) -> String {
    cli_command_project_root_output_dot("dump-ast", dir, context)
}

fn cli_dump_ast_err(input_path: &std::path::Path, context: &str) -> String {
    let input = normalize_path(input_path);
    let output = Command::new(env!("CARGO_BIN_EXE_smc"))
        .arg("dump-ast")
        .arg(input)
        .output()
        .unwrap_or_else(|err| panic!("{context} failed to spawn smc: {err}"));
    assert!(
        !output.status.success(),
        "{context} expected failure but got success\nstdout:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    String::from_utf8_lossy(&output.stderr).to_string()
}

#[test]
fn pcc9_dump_ast_semantic_toml_explicit_entry() {
    let dir = mk_temp_dir("pcc9_dump_ast_explicit");
    write_semantic_toml(&dir, Some("src/main.sm"));
    let entry_file = dir.join("src").join("main.sm");
    std::fs::create_dir_all(entry_file.parent().unwrap()).expect("mkdir");
    std::fs::write(&entry_file, "fn explicit_main() { return; }\n").expect("write file");

    let ast = cli_dump_ast_ok(&dir, "dump-ast with semantic.toml explicit entry");
    assert!(
        ast.contains("explicit_main"),
        "AST should contain explicit_main: {ast}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_dump_ast_semantic_toml_default_entry() {
    let dir = mk_temp_dir("pcc9_dump_ast_default");
    write_semantic_toml(&dir, None);
    let entry_file = dir.join("src").join("main.sm");
    std::fs::create_dir_all(entry_file.parent().unwrap()).expect("mkdir");
    std::fs::write(&entry_file, "fn default_main() { return; }\n").expect("write file");

    let ast = cli_dump_ast_ok(&dir, "dump-ast with semantic.toml default entry");
    assert!(
        ast.contains("default_main"),
        "AST should contain default_main: {ast}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_dump_ast_semantic_toml_nested_entry() {
    let dir = mk_temp_dir("pcc9_dump_ast_nested");
    write_semantic_toml(&dir, Some("examples/main.sm"));
    let entry_file = dir.join("examples").join("main.sm");
    std::fs::create_dir_all(entry_file.parent().unwrap()).expect("mkdir");
    std::fs::write(&entry_file, "fn nested_main() { return; }\n").expect("write file");

    let ast = cli_dump_ast_ok(&dir, "dump-ast with semantic.toml nested entry");
    assert!(
        ast.contains("nested_main"),
        "AST should contain nested_main: {ast}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_dump_ast_prefers_semantic_toml_over_package_manifest() {
    let dir = mk_temp_dir("pcc9_dump_ast_preferred");
    write_semantic_toml(&dir, Some("examples/main.sm"));
    write_package_manifest_baseline(&dir);
    let toml_entry = dir.join("examples").join("main.sm");
    std::fs::create_dir_all(toml_entry.parent().unwrap()).expect("mkdir");
    std::fs::write(&toml_entry, "fn toml_main() { return; }\n").expect("write file");

    let pkg_entry = dir.join("src").join("main.sm");
    std::fs::create_dir_all(pkg_entry.parent().unwrap()).expect("mkdir");
    std::fs::write(&pkg_entry, "fn pkg_main() { return; }\n").expect("write file");

    let ast = cli_dump_ast_ok(&dir, "dump-ast prefers semantic.toml over Semantic.package");
    assert!(
        ast.contains("toml_main"),
        "AST should contain toml_main: {ast}"
    );
    assert!(
        !ast.contains("pkg_main"),
        "AST should not contain pkg_main: {ast}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_dump_ast_package_baseline_fallback() {
    let dir = mk_temp_dir("pcc9_dump_ast_package_baseline");
    write_package_manifest_baseline(&dir);
    let entry_file = dir.join("src").join("main.sm");
    std::fs::create_dir_all(entry_file.parent().unwrap()).expect("mkdir");
    std::fs::write(&entry_file, "fn pkg_fallback() { return; }\n").expect("write file");

    let ast = cli_dump_ast_ok(&dir, "dump-ast fallback to Semantic.package");
    assert!(
        ast.contains("pkg_fallback"),
        "AST should contain pkg_fallback: {ast}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_dump_ast_dot_works_from_root() {
    let dir = mk_temp_dir("pcc9_dump_ast_dot");
    write_semantic_toml(&dir, Some("src/main.sm"));
    let entry_file = dir.join("src").join("main.sm");
    std::fs::create_dir_all(entry_file.parent().unwrap()).expect("mkdir");
    std::fs::write(&entry_file, "fn dot_main() { return; }\n").expect("write file");

    let ast = cli_dump_ast_dot_ok(&dir, "dump-ast . from project root");
    assert!(
        ast.contains("dot_main"),
        "AST should contain dot_main: {ast}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_dump_ast_invalid_semantic_toml_fails_without_fallback() {
    let dir = mk_temp_dir("pcc9_dump_ast_invalid_toml");
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package
name = "app"
"#,
    )
    .expect("write malformed toml");
    write_package_manifest_baseline(&dir);
    let entry_file = dir.join("src").join("main.sm");
    std::fs::create_dir_all(entry_file.parent().unwrap()).expect("mkdir");
    std::fs::write(&entry_file, "fn pkg_main() { return; }\n").expect("write file");

    let err = cli_dump_ast_err(&dir, "dump-ast with invalid semantic.toml");
    assert!(!err.is_empty(), "expected error output");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_dump_ast_missing_entry_file_fails_without_fallback() {
    let dir = mk_temp_dir("pcc9_dump_ast_missing_file");
    write_semantic_toml(&dir, Some("src/missing.sm"));

    let err = cli_dump_ast_err(&dir, "dump-ast with missing entry file");
    assert!(!err.is_empty(), "expected error output");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_dump_ast_path_escape_fails_without_fallback() {
    let dir = mk_temp_dir("pcc9_dump_ast_path_escape");
    write_semantic_toml(&dir, Some("../escape.sm"));

    let err = cli_dump_ast_err(&dir, "dump-ast with path escape entry");
    assert!(!err.is_empty(), "expected error output");

    let _ = std::fs::remove_dir_all(&dir);
}

fn cli_dump_ir_ok(input_path: &std::path::Path, context: &str) -> String {
    let input = normalize_path(input_path);
    let output = Command::new(env!("CARGO_BIN_EXE_smc"))
        .arg("dump-ir")
        .arg(input)
        .output()
        .unwrap_or_else(|err| panic!("{context} failed to spawn smc: {err}"));
    assert!(
        output.status.success(),
        "{context} failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn cli_dump_ir_dot_ok(dir: &std::path::Path, context: &str) -> String {
    cli_command_project_root_output_dot("dump-ir", dir, context)
}

fn cli_dump_ir_err(input_path: &std::path::Path, context: &str) -> String {
    let input = normalize_path(input_path);
    let output = Command::new(env!("CARGO_BIN_EXE_smc"))
        .arg("dump-ir")
        .arg(input)
        .output()
        .unwrap_or_else(|err| panic!("{context} failed to spawn smc: {err}"));
    assert!(
        !output.status.success(),
        "{context} expected failure but got success\nstdout:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    String::from_utf8_lossy(&output.stderr).to_string()
}

#[test]
fn pcc9_dump_ir_semantic_toml_explicit_entry() {
    let dir = mk_temp_dir("pcc9_dump_ir_explicit");
    write_semantic_toml(&dir, Some("src/main.sm"));
    let entry_file = dir.join("src").join("main.sm");
    std::fs::create_dir_all(entry_file.parent().unwrap()).expect("mkdir");
    std::fs::write(
        &entry_file,
        "fn main() { return; }\nfn explicit_main() { return; }\n",
    )
    .expect("write file");

    let ir = cli_dump_ir_ok(&dir, "dump-ir with semantic.toml explicit entry");
    assert!(
        ir.contains("explicit_main"),
        "IR should contain explicit_main: {ir}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_dump_ir_semantic_toml_default_entry() {
    let dir = mk_temp_dir("pcc9_dump_ir_default");
    write_semantic_toml(&dir, None);
    let entry_file = dir.join("src").join("main.sm");
    std::fs::create_dir_all(entry_file.parent().unwrap()).expect("mkdir");
    std::fs::write(
        &entry_file,
        "fn main() { return; }\nfn default_main() { return; }\n",
    )
    .expect("write file");

    let ir = cli_dump_ir_ok(&dir, "dump-ir with semantic.toml default entry");
    assert!(
        ir.contains("default_main"),
        "IR should contain default_main: {ir}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_dump_ir_semantic_toml_nested_entry() {
    let dir = mk_temp_dir("pcc9_dump_ir_nested");
    write_semantic_toml(&dir, Some("examples/main.sm"));
    let entry_file = dir.join("examples").join("main.sm");
    std::fs::create_dir_all(entry_file.parent().unwrap()).expect("mkdir");
    std::fs::write(
        &entry_file,
        "fn main() { return; }\nfn nested_main() { return; }\n",
    )
    .expect("write file");

    let ir = cli_dump_ir_ok(&dir, "dump-ir with semantic.toml nested entry");
    assert!(
        ir.contains("nested_main"),
        "IR should contain nested_main: {ir}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_dump_ir_prefers_semantic_toml_over_package_manifest() {
    let dir = mk_temp_dir("pcc9_dump_ir_preferred");
    write_semantic_toml(&dir, Some("examples/main.sm"));
    write_package_manifest_baseline(&dir);
    let toml_entry = dir.join("examples").join("main.sm");
    std::fs::create_dir_all(toml_entry.parent().unwrap()).expect("mkdir");
    std::fs::write(
        &toml_entry,
        "fn main() { return; }\nfn toml_main() { return; }\n",
    )
    .expect("write file");

    let pkg_entry = dir.join("src").join("main.sm");
    std::fs::create_dir_all(pkg_entry.parent().unwrap()).expect("mkdir");
    std::fs::write(
        &pkg_entry,
        "fn main() { return; }\nfn pkg_main() { return; }\n",
    )
    .expect("write file");

    let ir = cli_dump_ir_ok(&dir, "dump-ir prefers semantic.toml over Semantic.package");
    assert!(
        ir.contains("toml_main"),
        "IR should contain toml_main: {ir}"
    );
    assert!(
        !ir.contains("pkg_main"),
        "IR should not contain pkg_main: {ir}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_dump_ir_package_baseline_fallback() {
    let dir = mk_temp_dir("pcc9_dump_ir_package_baseline");
    write_package_manifest_baseline(&dir);
    let entry_file = dir.join("src").join("main.sm");
    std::fs::create_dir_all(entry_file.parent().unwrap()).expect("mkdir");
    std::fs::write(
        &entry_file,
        "fn main() { return; }\nfn pkg_fallback() { return; }\n",
    )
    .expect("write file");

    let ir = cli_dump_ir_ok(&dir, "dump-ir fallback to Semantic.package");
    assert!(
        ir.contains("pkg_fallback"),
        "IR should contain pkg_fallback: {ir}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_dump_ir_dot_works_from_root() {
    let dir = mk_temp_dir("pcc9_dump_ir_dot");
    write_semantic_toml(&dir, Some("src/main.sm"));
    let entry_file = dir.join("src").join("main.sm");
    std::fs::create_dir_all(entry_file.parent().unwrap()).expect("mkdir");
    std::fs::write(
        &entry_file,
        "fn main() { return; }\nfn dot_main() { return; }\n",
    )
    .expect("write file");

    let ir = cli_dump_ir_dot_ok(&dir, "dump-ir . from project root");
    assert!(ir.contains("dot_main"), "IR should contain dot_main: {ir}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_dump_ir_invalid_semantic_toml_fails_without_fallback() {
    let dir = mk_temp_dir("pcc9_dump_ir_invalid_toml");
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package
name = "app"
"#,
    )
    .expect("write malformed toml");
    write_package_manifest_baseline(&dir);
    let entry_file = dir.join("src").join("main.sm");
    std::fs::create_dir_all(entry_file.parent().unwrap()).expect("mkdir");
    std::fs::write(
        &entry_file,
        "fn main() { return; }\nfn pkg_main() { return; }\n",
    )
    .expect("write file");

    let err = cli_dump_ir_err(&dir, "dump-ir with invalid semantic.toml");
    assert!(!err.is_empty(), "expected error output");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_dump_ir_missing_entry_file_fails_without_fallback() {
    let dir = mk_temp_dir("pcc9_dump_ir_missing_file");
    write_semantic_toml(&dir, Some("src/missing.sm"));

    let err = cli_dump_ir_err(&dir, "dump-ir with missing entry file");
    assert!(!err.is_empty(), "expected error output");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_dump_ir_path_escape_fails_without_fallback() {
    let dir = mk_temp_dir("pcc9_dump_ir_path_escape");
    write_semantic_toml(&dir, Some("../escape.sm"));

    let err = cli_dump_ir_err(&dir, "dump-ir with path escape entry");
    assert!(!err.is_empty(), "expected error output");

    let _ = std::fs::remove_dir_all(&dir);
}

fn cli_dump_bytecode_ok(input_path: &std::path::Path, context: &str) -> String {
    let input = normalize_path(input_path);
    let output = Command::new(env!("CARGO_BIN_EXE_smc"))
        .arg("dump-bytecode")
        .arg(input)
        .output()
        .unwrap_or_else(|err| panic!("{context} failed to spawn smc: {err}"));
    assert!(
        output.status.success(),
        "{context} failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn cli_dump_bytecode_dot_ok(dir: &std::path::Path, context: &str) -> String {
    cli_command_project_root_output_dot("dump-bytecode", dir, context)
}

fn cli_dump_bytecode_err(input_path: &std::path::Path, context: &str) -> String {
    let input = normalize_path(input_path);
    let output = Command::new(env!("CARGO_BIN_EXE_smc"))
        .arg("dump-bytecode")
        .arg(input)
        .output()
        .unwrap_or_else(|err| panic!("{context} failed to spawn smc: {err}"));
    assert!(
        !output.status.success(),
        "{context} expected failure but got success\nstdout:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    String::from_utf8_lossy(&output.stderr).to_string()
}

#[test]
fn pcc9_dump_bytecode_semantic_toml_explicit_entry() {
    let dir = mk_temp_dir("pcc9_dump_bytecode_explicit");
    write_semantic_toml(&dir, Some("src/main.sm"));
    let entry_file = dir.join("src").join("main.sm");
    std::fs::create_dir_all(entry_file.parent().unwrap()).expect("mkdir");
    std::fs::write(&entry_file, "fn main() { return; }\n").expect("write file");

    let bytes = cli_dump_bytecode_ok(&dir, "dump-bytecode with semantic.toml explicit entry");
    assert!(
        bytes.contains("0000:"),
        "Bytecode should contain offset 0000: {bytes}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_dump_bytecode_semantic_toml_default_entry() {
    let dir = mk_temp_dir("pcc9_dump_bytecode_default");
    write_semantic_toml(&dir, None);
    let entry_file = dir.join("src").join("main.sm");
    std::fs::create_dir_all(entry_file.parent().unwrap()).expect("mkdir");
    std::fs::write(&entry_file, "fn main() { return; }\n").expect("write file");

    let bytes = cli_dump_bytecode_ok(&dir, "dump-bytecode with semantic.toml default entry");
    assert!(
        bytes.contains("0000:"),
        "Bytecode should contain offset 0000: {bytes}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_dump_bytecode_semantic_toml_nested_entry() {
    let dir = mk_temp_dir("pcc9_dump_bytecode_nested");
    write_semantic_toml(&dir, Some("examples/main.sm"));
    let entry_file = dir.join("examples").join("main.sm");
    std::fs::create_dir_all(entry_file.parent().unwrap()).expect("mkdir");
    std::fs::write(&entry_file, "fn main() { return; }\n").expect("write file");

    let bytes = cli_dump_bytecode_ok(&dir, "dump-bytecode with semantic.toml nested entry");
    assert!(
        bytes.contains("0000:"),
        "Bytecode should contain offset 0000: {bytes}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_dump_bytecode_prefers_semantic_toml_over_package_manifest() {
    let dir = mk_temp_dir("pcc9_dump_bytecode_preferred");
    write_semantic_toml(&dir, Some("examples/main.sm"));
    write_package_manifest_baseline(&dir);
    let toml_entry = dir.join("examples").join("main.sm");
    std::fs::create_dir_all(toml_entry.parent().unwrap()).expect("mkdir");
    std::fs::write(&toml_entry, "fn main() { return; }\n").expect("write file");

    let pkg_entry = dir.join("src").join("main.sm");
    std::fs::create_dir_all(pkg_entry.parent().unwrap()).expect("mkdir");
    std::fs::write(&pkg_entry, "fn main() { return; }\n").expect("write file");

    let bytes = cli_dump_bytecode_ok(
        &dir,
        "dump-bytecode prefers semantic.toml over Semantic.package",
    );
    assert!(
        bytes.contains("0000:"),
        "Bytecode should contain offset 0000: {bytes}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_dump_bytecode_package_baseline_fallback() {
    let dir = mk_temp_dir("pcc9_dump_bytecode_package_baseline");
    write_package_manifest_baseline(&dir);
    let entry_file = dir.join("src").join("main.sm");
    std::fs::create_dir_all(entry_file.parent().unwrap()).expect("mkdir");
    std::fs::write(&entry_file, "fn main() { return; }\n").expect("write file");

    let bytes = cli_dump_bytecode_ok(&dir, "dump-bytecode fallback to Semantic.package");
    assert!(
        bytes.contains("0000:"),
        "Bytecode should contain offset 0000: {bytes}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_dump_bytecode_dot_works_from_root() {
    let dir = mk_temp_dir("pcc9_dump_bytecode_dot");
    write_semantic_toml(&dir, Some("src/main.sm"));
    let entry_file = dir.join("src").join("main.sm");
    std::fs::create_dir_all(entry_file.parent().unwrap()).expect("mkdir");
    std::fs::write(&entry_file, "fn main() { return; }\n").expect("write file");

    let bytes = cli_dump_bytecode_dot_ok(&dir, "dump-bytecode . from project root");
    assert!(
        bytes.contains("0000:"),
        "Bytecode should contain offset 0000: {bytes}"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_dump_bytecode_invalid_semantic_toml_fails_without_fallback() {
    let dir = mk_temp_dir("pcc9_dump_bytecode_invalid_toml");
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package
name = "app"
"#,
    )
    .expect("write malformed toml");
    write_package_manifest_baseline(&dir);
    let entry_file = dir.join("src").join("main.sm");
    std::fs::create_dir_all(entry_file.parent().unwrap()).expect("mkdir");
    std::fs::write(&entry_file, "fn main() { return; }\n").expect("write file");

    let err = cli_dump_bytecode_err(&dir, "dump-bytecode with invalid semantic.toml");
    assert!(!err.is_empty(), "expected error output");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_dump_bytecode_missing_entry_file_fails_without_fallback() {
    let dir = mk_temp_dir("pcc9_dump_bytecode_missing_file");
    write_semantic_toml(&dir, Some("src/missing.sm"));

    let err = cli_dump_bytecode_err(&dir, "dump-bytecode with missing entry file");
    assert!(!err.is_empty(), "expected error output");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_dump_bytecode_path_escape_fails_without_fallback() {
    let dir = mk_temp_dir("pcc9_dump_bytecode_path_escape");
    write_semantic_toml(&dir, Some("../escape.sm"));

    let err = cli_dump_bytecode_err(&dir, "dump-bytecode with path escape entry");
    assert!(!err.is_empty(), "expected error output");

    let _ = std::fs::remove_dir_all(&dir);
}
