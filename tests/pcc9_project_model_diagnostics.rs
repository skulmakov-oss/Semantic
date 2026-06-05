use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use smc_cli::{
    admit_package_entry_module, parse_package_manifest_baseline, resolve_package_import_path,
    validate_package_manifest_baseline, PackageImportResolutionCode, PackageManifestParseCode,
    PackageManifestValidationCode, PackageModuleAdmissionCode, PACKAGE_MANIFEST_FILE_NAME,
};

fn repo_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn fixture_path(rel: &str) -> PathBuf {
    repo_path(&format!(
        "tests/fixtures/pcc9_project_model_diagnostics/{rel}"
    ))
}

fn fixture_text(rel: &str) -> String {
    std::fs::read_to_string(fixture_path(rel))
        .unwrap_or_else(|err| panic!("read fixture {rel}: {err}"))
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

fn assert_manifest_parse_error(rel: &str, code: PackageManifestParseCode, needle: &str) {
    let source = fixture_text(rel);
    let err = parse_package_manifest_baseline(&source).expect_err("must reject");
    assert_eq!(err.code, code, "unexpected parse code for {rel}: {err:?}");
    assert!(
        err.message.contains(needle),
        "expected parse error containing '{needle}' for {rel}, got: {}",
        err.message
    );
}

fn assert_manifest_validate_error(rel: &str, code: PackageManifestValidationCode, needle: &str) {
    let source = fixture_text(rel);
    let manifest = parse_package_manifest_baseline(&source).expect("parse");
    let err = validate_package_manifest_baseline(&manifest).expect_err("must reject");
    assert_eq!(
        err.code, code,
        "unexpected validation code for {rel}: {err:?}"
    );
    assert!(
        err.message.contains(needle),
        "expected validation error containing '{needle}' for {rel}, got: {}",
        err.message
    );
}

fn assert_entry_admission_error(
    rel: &str,
    entry_rel: &str,
    code: PackageModuleAdmissionCode,
    needle: &str,
) {
    let dir = mk_temp_dir("pcc9_entry_admission_error");
    std::fs::create_dir_all(dir.join("src")).expect("mkdir src");
    let manifest_path = dir.join(PACKAGE_MANIFEST_FILE_NAME);
    std::fs::write(&manifest_path, fixture_text(rel)).expect("write manifest");

    let entry = dir.join(entry_rel);
    if let Some(parent) = entry.parent() {
        std::fs::create_dir_all(parent).expect("mkdir entry parent");
    }
    std::fs::write(&entry, "fn main() { return; }").expect("write entry");

    let err = admit_package_entry_module(&entry).expect_err("must reject");
    assert_eq!(
        err.code, code,
        "unexpected entry admission code for {rel}: {err:?}"
    );
    assert!(
        err.message.contains(needle),
        "expected entry admission error containing '{needle}' for {rel}, got: {}",
        err.message
    );

    let _ = std::fs::remove_dir_all(&dir);
}

fn assert_import_resolution_error(
    rel: &str,
    import: &str,
    code: PackageImportResolutionCode,
    needle: &str,
) {
    let dir = mk_temp_dir("pcc9_import_resolution_error");
    let app_dir = dir.join("app").join("src");
    let dep_dir = dir.join("dep").join("src");
    std::fs::create_dir_all(&app_dir).expect("mkdir app src");
    std::fs::create_dir_all(&dep_dir).expect("mkdir dep src");

    std::fs::write(
        dir.join("app").join(PACKAGE_MANIFEST_FILE_NAME),
        fixture_text(rel),
    )
    .expect("write app manifest");
    std::fs::write(
        dir.join("dep").join(PACKAGE_MANIFEST_FILE_NAME),
        r#"
format 1
package dep
manifest_dir .
module_root src
"#,
    )
    .expect("write dep manifest");

    let importer = app_dir.join("main.sm");
    std::fs::write(
        &importer,
        format!("Import \"{import}\"\nfn main() {{ return; }}\n"),
    )
    .expect("write importer");

    let err = resolve_package_import_path(&importer, import).expect_err("must reject");
    assert_eq!(
        err.code, code,
        "unexpected import resolution code for {rel}: {err:?}"
    );
    assert!(
        err.message.contains(needle),
        "expected import resolution error containing '{needle}' for {rel}, got: {}",
        err.message
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn pcc9_manifest_malformed_rejects_parse() {
    assert_manifest_parse_error(
        "negative_manifest_malformed/Semantic.package",
        PackageManifestParseCode::InvalidDirective,
        "unterminated quoted value",
    );
}

#[test]
fn pcc9_manifest_missing_package_rejects_parse() {
    assert_manifest_parse_error(
        "negative_manifest_missing_package/Semantic.package",
        PackageManifestParseCode::MissingPackageDirective,
        "explicit package directive",
    );
}

#[test]
fn pcc9_manifest_invalid_format_rejects_validation() {
    assert_manifest_validate_error(
        "negative_manifest_invalid_format/Semantic.package",
        PackageManifestValidationCode::UnsupportedFormatVersion,
        "unsupported package manifest format version",
    );
}

#[test]
fn pcc9_manifest_invalid_dep_shape_rejects_parse() {
    assert_manifest_parse_error(
        "negative_manifest_invalid_dep_shape/Semantic.package",
        PackageManifestParseCode::InvalidDirective,
        "dep directive must be: dep <alias> <package_name> <local_path>",
    );
}

#[test]
fn pcc9_manifest_missing_manifest_dir_rejects_parse() {
    assert_manifest_parse_error(
        "negative_manifest_missing_manifest_dir/Semantic.package",
        PackageManifestParseCode::MissingManifestDirDirective,
        "explicit manifest_dir directive",
    );
}

#[test]
fn pcc9_manifest_missing_module_root_rejects_parse() {
    assert_manifest_parse_error(
        "negative_manifest_missing_module_root/Semantic.package",
        PackageManifestParseCode::MissingModuleRootDirective,
        "explicit module_root directive",
    );
}

#[test]
fn pcc9_entry_admission_rejects_path_escape() {
    assert_entry_admission_error(
        "negative_entry_path_escape/Semantic.package",
        "main.sm",
        PackageModuleAdmissionCode::EntryOutsideModuleRoot,
        "outside admitted package module_root",
    );
}

#[test]
fn pcc9_import_resolution_rejects_unresolved_dependency_alias() {
    assert_import_resolution_error(
        "negative_import_unresolved_dep/Semantic.package",
        "math::core.sm",
        PackageImportResolutionCode::UnknownDependencyAlias,
        "does not declare dependency alias",
    );
}
