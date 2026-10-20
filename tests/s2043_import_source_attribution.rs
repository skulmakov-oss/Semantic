//! S2043 CP03: diagnostics found in a composed executable bundle
//! (`Import "helper.sm"`) are attributed to the one original file they lie
//! in, with that file's own byte range, or reported without a location. A
//! bundle position is never presented as a position of the root or of a
//! different helper.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

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

fn smc_in(dir: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_smc"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("run smc");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn project(files: &[(&str, &str)]) -> PathBuf {
    let dir = mk_temp_dir("s2043_cp03");
    for (name, text) in files {
        std::fs::write(dir.join(name), text).expect("write module");
    }
    dir
}

fn line_col(src: &str, offset: usize) -> (usize, usize) {
    let before = &src[..offset];
    let line = before.matches('\n').count() + 1;
    let col = offset - before.rfind('\n').map(|i| i + 1).unwrap_or(0) + 1;
    (line, col)
}

fn json(dir: &Path) -> serde_json::Value {
    let (code, stdout, stderr) = smc_in(dir, &["check", "main.sm", "--format", "json"]);
    assert_eq!(code, 1, "{stdout}{stderr}");
    serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("invalid json {e}: {stdout}"))
}

fn legacy(dir: &Path) -> String {
    let (code, stdout, stderr) =
        smc_in(dir, &["check", "main.sm", "--no-cache", "--color", "never"]);
    assert_eq!(code, 1, "{stdout}{stderr}");
    format!("{stdout}{stderr}")
}

/// Asserts the first diagnostic is attributed to `file` at the exact extent
/// of `token` (first occurrence) inside that file's own text.
fn assert_attributed(dir: &Path, file: &str, token: &str) {
    let text = std::fs::read_to_string(dir.join(file)).unwrap();
    let start = text.find(token).expect("token in file");
    let end = start + token.len();
    let (line, col) = line_col(&text, start);

    let report = json(dir);
    let diag = &report["diagnostics"][0];
    let source = diag["source"]
        .as_u64()
        .unwrap_or_else(|| panic!("{file}: diagnostic not bound to a source: {report}"));
    let path = report["sources"][source as usize]["path"]
        .as_str()
        .unwrap_or("");
    assert!(path.ends_with(file), "{file}: bound to {path}: {report}");
    assert_eq!(
        diag["range"]["start"]["offset"].as_u64(),
        Some(start as u64),
        "{report}"
    );
    assert_eq!(
        diag["range"]["end"]["offset"].as_u64(),
        Some(end as u64),
        "{report}"
    );

    let text_out = legacy(dir);
    assert!(
        text_out.contains(&format!("at line {line}:{col}")),
        "{file}: expected line {line}:{col}: {text_out}"
    );
    let header = text_out.lines().next().unwrap_or_default();
    if file == "main.sm" {
        assert!(
            header.ends_with(&format!("at line {line}:{col}")),
            "root error must not name another file: {text_out}"
        );
    } else {
        assert!(
            header.ends_with(&format!("at line {line}:{col} in {file}")),
            "helper location must name its file: {text_out}"
        );
    }
}

fn assert_unlocated(dir: &Path) {
    let report = json(dir);
    let diag = &report["diagnostics"][0];
    assert!(diag["source"].is_null(), "{report}");
    assert!(diag["range"].is_null(), "{report}");
    let text_out = legacy(dir);
    assert!(text_out.contains("location unavailable"), "{text_out}");
    assert!(!text_out.contains("at line"), "{text_out}");
}

const MAIN_CALLS_HELPER: &str =
    "Import \"helper.sm\"\nfn main() {\n    let x: i32 = helper();\n    let _ = x;\n    return;\n}\n";

#[test]
fn helper_error_is_attributed_to_the_helper_file() {
    let dir = project(&[
        ("main.sm", MAIN_CALLS_HELPER),
        (
            "helper.sm",
            "fn helper() -> i32 {\n    let y: i32 = 1u32;\n    return y;\n}\n",
        ),
    ]);
    assert_attributed(&dir, "helper.sm", "y");
}

#[test]
fn root_error_with_an_import_is_attributed_to_the_root() {
    let dir = project(&[
        (
            "main.sm",
            "Import \"helper.sm\"\nfn main() {\n    let x: i32 = helper();\n    let z: i32 = 2u32;\n    let _ = x;\n    let _ = z;\n    return;\n}\n",
        ),
        ("helper.sm", "fn helper() -> i32 {\n    return 1;\n}\n"),
    ]);
    assert_attributed(&dir, "main.sm", "z");
}

#[test]
fn nested_import_error_is_attributed_to_the_innermost_file() {
    let dir = project(&[
        (
            "main.sm",
            "Import \"a.sm\"\nfn main() {\n    let x: i32 = from_a();\n    let _ = x;\n    return;\n}\n",
        ),
        (
            "a.sm",
            "Import \"b.sm\"\nfn from_a() -> i32 {\n    return from_b();\n}\n",
        ),
        (
            "b.sm",
            "fn from_b() -> i32 {\n    let q: i32 = ghost;\n    return q;\n}\n",
        ),
    ]);
    assert_attributed(&dir, "b.sm", "ghost");
}

#[test]
fn unicode_before_a_helper_error_keeps_byte_exact_ranges() {
    let dir = project(&[
        ("main.sm", MAIN_CALLS_HELPER),
        (
            "helper.sm",
            "fn helper() -> i32 {\n    let s: text = \"\u{e9}\u{1f642}\";\n    let _ = s;\n    let y: i32 = missing;\n    return y;\n}\n",
        ),
    ]);
    assert_attributed(&dir, "helper.sm", "missing");
}

#[test]
fn second_of_two_helpers_is_attributed_correctly() {
    let dir = project(&[
        (
            "main.sm",
            "Import \"one.sm\"\nImport \"two.sm\"\nfn main() {\n    let x: i32 = first() + second();\n    let _ = x;\n    return;\n}\n",
        ),
        ("one.sm", "fn first() -> i32 {\n    return 1;\n}\n"),
        (
            "two.sm",
            "fn second() -> i32 {\n    let w: i32 = 2u32;\n    return w;\n}\n",
        ),
    ]);
    assert_attributed(&dir, "two.sm", "w");
}

#[test]
fn synthesized_selected_import_error_is_reported_without_location() {
    // Selected imports compose rewritten (renamed) helper text, which is not
    // byte-identical to the helper file, so no position can be proven.
    let dir = project(&[
        (
            "main.sm",
            "Import \"helper.sm\" { helper }\nfn main() {\n    let x: i32 = helper();\n    let _ = x;\n    return;\n}\n",
        ),
        (
            "helper.sm",
            "fn helper() -> i32 {\n    let y: i32 = 1u32;\n    return y;\n}\n",
        ),
    ]);
    assert_unlocated(&dir);
}

#[test]
fn unanchored_error_inside_a_bundle_stays_unlocated() {
    let dir = project(&[
        ("main.sm", MAIN_CALLS_HELPER),
        ("helper.sm", "fn helper() -> i32 {\n    return true;\n}\n"),
    ]);
    assert_unlocated(&dir);
}

#[test]
fn subdirectory_and_parent_relative_helpers_are_named_relatively() {
    let root = mk_temp_dir("s2043_cp03_rel");
    let app = root.join("app");
    std::fs::create_dir_all(app.join("sub")).unwrap();
    std::fs::write(
        app.join("main.sm"),
        "Import \"sub/helper.sm\"\nImport \"../outside.sm\"\nfn main() {\n    let x: i32 = helper() + outside();\n    let _ = x;\n    return;\n}\n",
    )
    .unwrap();
    std::fs::write(
        app.join("sub").join("helper.sm"),
        "fn helper() -> i32 {\n    let h: i32 = 1u32;\n    return h;\n}\n",
    )
    .unwrap();
    std::fs::write(
        root.join("outside.sm"),
        "fn outside() -> i32 {\n    return 1;\n}\n",
    )
    .unwrap();
    let text_out = legacy(&app);
    assert!(
        text_out
            .lines()
            .next()
            .unwrap_or_default()
            .ends_with("at line 2:9 in sub/helper.sm"),
        "{text_out}"
    );

    std::fs::write(
        app.join("sub").join("helper.sm"),
        "fn helper() -> i32 {\n    return 1;\n}\n",
    )
    .unwrap();
    std::fs::write(
        root.join("outside.sm"),
        "fn outside() -> i32 {\n    let o: i32 = 1u32;\n    return o;\n}\n",
    )
    .unwrap();
    let text_out = legacy(&app);
    assert!(
        text_out
            .lines()
            .next()
            .unwrap_or_default()
            .ends_with("at line 2:9 in ../outside.sm"),
        "{text_out}"
    );

    // No absolute host path in either surface.
    let host = root.to_string_lossy().replace('\\', "/");
    let (_, json_out, _) = smc_in(&app, &["check", "main.sm", "--format", "json"]);
    for out in [&text_out, &json_out] {
        let normalized = out.replace('\\', "/");
        assert!(!normalized.contains(&host), "host path leaked: {out}");
    }
}

/// `fixture/outside.sm` (type error) imported by `fixture/app/main.sm` as
/// `../outside.sm`; `fixture/app/sub` is an empty directory. No manifest, so
/// every module is rootless (no package identity).
fn external_module_fixture() -> (PathBuf, &'static str) {
    let fixture = mk_temp_dir("s2054_external");
    std::fs::create_dir_all(fixture.join("app").join("sub")).unwrap();
    std::fs::write(
        fixture.join("app").join("main.sm"),
        "Import \"../outside.sm\"\n\nfn main() {\n    let value: i32 = outside();\n    let _ = value;\n    return;\n}\n",
    )
    .unwrap();
    let outside = "fn outside() -> i32 {\n    let value: i32 = 1u32;\n    return value;\n}\n";
    std::fs::write(fixture.join("outside.sm"), outside).unwrap();
    (fixture, outside)
}

/// Asserts the JSON document of `smc check <entry> --format json` run in
/// `cwd` attributes its one diagnostic to `outside.sm` under exactly
/// `expected_path`, with `outside.sm`'s own byte range, and returns stdout.
fn assert_external_module_path(
    cwd: &Path,
    entry: &str,
    expected_path: &str,
    outside: &str,
) -> String {
    let (code, stdout, stderr) = smc_in(cwd, &["check", entry, "--format", "json"]);
    assert_eq!(code, 1, "{stdout}{stderr}");
    let report: serde_json::Value =
        serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("invalid json {e}: {stdout}"));
    assert_eq!(report["schema"], "semantic.diagnostics", "{report}");
    assert_eq!(report["schema_version"], 1, "{report}");
    let diag = &report["diagnostics"][0];
    let source = diag["source"]
        .as_u64()
        .unwrap_or_else(|| panic!("diagnostic not bound to a source: {report}"));
    assert_ne!(source, 0, "attributed to the root: {report}");
    let entry_json = &report["sources"][source as usize];
    assert_eq!(entry_json["path"], expected_path, "{report}");
    assert!(
        entry_json["identity"].is_null(),
        "rootless module: {report}"
    );
    // The let binding's name in outside.sm, as for any let type mismatch.
    let start = outside.find("value").unwrap();
    assert_eq!(
        diag["range"]["start"]["offset"].as_u64(),
        Some(start as u64),
        "{report}"
    );
    assert_eq!(
        diag["range"]["end"]["offset"].as_u64(),
        Some((start + "value".len()) as u64),
        "{report}"
    );
    stdout
}

/// skulmakov-oss/Semantic#2054: a bundle module outside the working
/// directory is named by its relative form, never `null` / `null`.
#[test]
fn external_bundle_module_is_named_relative_to_the_working_directory() {
    let (fixture, outside) = external_module_fixture();
    let app = fixture.join("app");
    let first = assert_external_module_path(&app, "main.sm", "../outside.sm", outside);
    let host = fixture.to_string_lossy().replace('\\', "/");
    assert!(
        !first.replace('\\', "/").contains(&host),
        "host path leaked: {first}"
    );
    let second = assert_external_module_path(&app, "main.sm", "../outside.sm", outside);
    assert_eq!(first, second, "deterministic");
}

/// #2054: the path is relative to the working directory (the schema's
/// `Source.path` contract), not to the root module's directory, when the
/// two differ.
#[test]
fn external_bundle_module_path_is_relative_to_the_working_directory_not_the_root() {
    let (fixture, outside) = external_module_fixture();
    let sub = fixture.join("app").join("sub");
    let out = assert_external_module_path(&sub, "../main.sm", "../../outside.sm", outside);
    let report: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(report["sources"][0]["path"], "../main.sm", "{report}");
}

#[test]
fn diamond_import_error_is_attributed_to_the_shared_module_once() {
    let dir = project(&[
        (
            "main.sm",
            "Import \"a.sm\"\nImport \"b.sm\"\nfn main() {\n    let x: i32 = from_a() + from_b();\n    let _ = x;\n    return;\n}\n",
        ),
        ("a.sm", "Import \"c.sm\"\nfn from_a() -> i32 {\n    return shared();\n}\n"),
        ("b.sm", "Import \"c.sm\"\nfn from_b() -> i32 {\n    return shared();\n}\n"),
        (
            "c.sm",
            "fn shared() -> i32 {\n    let c: i32 = 3u32;\n    return c;\n}\n",
        ),
    ]);
    assert_attributed(&dir, "c.sm", "c");
    let report = json(&dir);
    let c_sources = report["sources"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|s| s["path"].as_str().is_some_and(|p| p.ends_with("c.sm")))
        .count();
    assert_eq!(c_sources, 1, "{report}");
}

#[test]
fn bundle_attribution_is_deterministic() {
    let dir = project(&[
        ("main.sm", MAIN_CALLS_HELPER),
        (
            "helper.sm",
            "fn helper() -> i32 {\n    let y: i32 = 1u32;\n    return y;\n}\n",
        ),
    ]);
    let first = smc_in(&dir, &["check", "main.sm", "--format", "json"]);
    let second = smc_in(&dir, &["check", "main.sm", "--format", "json"]);
    assert_eq!(first, second);
    let first = smc_in(
        &dir,
        &["check", "main.sm", "--no-cache", "--color", "never"],
    );
    let second = smc_in(
        &dir,
        &["check", "main.sm", "--no-cache", "--color", "never"],
    );
    assert_eq!(first, second);
}

#[test]
fn valid_bundle_still_compiles_and_runs() {
    let dir = project(&[
        ("main.sm", MAIN_CALLS_HELPER),
        ("helper.sm", "fn helper() -> i32 {\n    return 7;\n}\n"),
    ]);
    let (code, stdout, stderr) = smc_in(&dir, &["check", "main.sm", "--format", "json"]);
    assert_eq!(code, 0, "{stdout}{stderr}");
    let (code, stdout, stderr) = smc_in(&dir, &["run", "main.sm"]);
    assert_eq!(code, 0, "{stdout}{stderr}");
}
