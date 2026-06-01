use std::path::PathBuf;
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

fn full_project_root_package_baseline_check_path() {
    let dir = mk_temp_dir("pcc9_project_root_package_acceptance");
    let src_dir = dir.join("src");
    std::fs::create_dir_all(&src_dir).expect("mkdir src");
    std::fs::write(
        dir.join(PACKAGE_MANIFEST_FILE_NAME),
        r#"
format 1
package app
manifest_dir .
module_root src
"#,
    )
    .expect("write manifest");
    std::fs::write(src_dir.join("main.sm"), "fn main() { return; }\n").expect("write entry");

    let input = normalize_path(&dir);
    cli_ok(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for package baseline project root {input}"),
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
