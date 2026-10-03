//! SSF-09 #1580 editor baseline qualification suite.
//!
//! Covers the #1580 acceptance criteria end to end against the real `smc`
//! binary and the in-process `smc lsp` server:
//!
//! - AC1 versioned machine schema: byte-exact goldens, determinism.
//! - AC2 code/severity/source/range preservation, including honest absence.
//! - AC3 formatter determinism, idempotence, semantic (token) preservation.
//! - AC4 CLI/LSP parity: the LSP never disagrees with `smc check`.
//! - AC5 project-root awareness: package identity, nested module paths.
//! - AC6 no hidden network/telemetry surface.
//! - AC7 a working protocol path (LSP over stdio), with negative fixtures.
//!
//! Goldens are re-blessed with `SM_UPDATE_SSF09_GOLDENS=1`.

use serde_json::{json, Value};
use smc_cli::lsp::{serve, utf16_position, LspExit};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const FIXTURES: &str = "tests/fixtures/ssf09_editor_baseline";
const GOLDENS: &str = "tests/golden_snapshots/ssf09_editor_baseline";

const ROOTLESS: [&str; 9] = [
    "ok",
    "syntax_error",
    "lex_error",
    "type_error",
    "no_surface_claim",
    "ambiguous_surface",
    "utf16_syntax_error",
    "logos_warnings",
    "logos_recovered",
];

fn fixture_dir(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(FIXTURES)
        .join(name)
}

fn smc(cwd: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_smc"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("run smc");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8(out.stdout).expect("utf8 stdout"),
        String::from_utf8(out.stderr).expect("utf8 stderr"),
    )
}

fn check(cwd: &Path, entry: &str, format: &str) -> (i32, String) {
    let (code, stdout, stderr) = smc(cwd, &["check", entry, "--format", format]);
    assert!(
        stderr.is_empty(),
        "canonical check wrote to stderr: {stderr}"
    );
    (code, stdout)
}

fn check_json(cwd: &Path, entry: &str) -> (i32, Value) {
    let (code, stdout) = check(cwd, entry, "json");
    (
        code,
        serde_json::from_str(&stdout).expect("schema output is JSON"),
    )
}

/// Replaces the machine-specific absolute fixture root so goldens are
/// portable. Structural fields never contain it; only producer message
/// text for module-load failures does (a documented limitation).
fn portable(text: &str) -> String {
    let root = fixture_dir("");
    let root = root.canonicalize().unwrap_or(root);
    text.replace(&root.to_string_lossy().replace('\\', "/"), "<FIXTURES>/")
        .replace("<FIXTURES>//", "<FIXTURES>/")
}

fn assert_golden(name: &str, actual: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(GOLDENS)
        .join(name);
    let actual = portable(actual);
    if std::env::var("SM_UPDATE_SSF09_GOLDENS").is_ok_and(|v| v == "1") {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, &actual).unwrap();
        return;
    }
    let expected = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read golden {}: {e}", path.display()))
        .replace("\r\n", "\n");
    assert_eq!(actual, expected, "golden {name} drifted");
}

// ---------------------------------------------------------------- AC1

#[test]
fn schema_goldens_rootless() {
    let dir = fixture_dir("rootless");
    for name in ROOTLESS {
        let (code, json) = check(&dir, &format!("{name}.sm"), "json");
        let expected_code = if name == "ok" || name == "logos_warnings" {
            0
        } else {
            1
        };
        assert_eq!(code, expected_code, "{name}: exit code");
        assert_golden(&format!("{name}.json"), &json);
    }
}

#[test]
fn schema_goldens_projects() {
    for (project, expected_code) in [("project_ok", 0), ("project_broken", 1)] {
        let dir = fixture_dir(project);
        let (code, json) = check(&dir, "src/main.sm", "json");
        assert_eq!(code, expected_code, "{project}: exit code");
        assert_golden(&format!("{project}.json"), &json);
    }
}

#[test]
fn human_goldens() {
    let dir = fixture_dir("rootless");
    for name in ROOTLESS {
        let (_, human) = check(&dir, &format!("{name}.sm"), "human");
        assert!(
            !human.contains('\u{1b}'),
            "{name}: ANSI escape in human output"
        );
        assert_golden(&format!("{name}.txt"), &human);
    }
    for project in ["project_ok", "project_broken"] {
        let (_, human) = check(&fixture_dir(project), "src/main.sm", "human");
        assert_golden(&format!("{project}.txt"), &human);
    }
}

#[test]
fn schema_output_is_byte_deterministic() {
    let dir = fixture_dir("project_ok");
    let first = check(&dir, "src/main.sm", "json").1;
    for _ in 0..3 {
        assert_eq!(check(&dir, "src/main.sm", "json").1, first);
    }
}

#[test]
fn schema_header_and_top_level_shape_are_stable() {
    let (_, json) = check_json(&fixture_dir("rootless"), "syntax_error.sm");
    let keys: Vec<&str> = json
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    let expected: BTreeSet<&str> = [
        "schema",
        "schema_version",
        "status",
        "sources",
        "diagnostics",
        "summary",
        "failure",
    ]
    .into_iter()
    .collect();
    assert_eq!(keys.into_iter().collect::<BTreeSet<_>>(), expected);
    assert_eq!(json["schema"], "semantic.diagnostics");
    assert_eq!(json["schema_version"], 1);
    let diag = &json["diagnostics"][0];
    let dkeys: BTreeSet<&str> = diag
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    let dexpected: BTreeSet<&str> = [
        "code", "severity", "family", "message", "source", "range", "related", "notes", "fix",
        "cause",
    ]
    .into_iter()
    .collect();
    assert_eq!(dkeys, dexpected);
}

#[test]
fn host_failure_is_a_schema_document_not_a_diagnostic() {
    let (code, json) = check_json(&fixture_dir("rootless"), "does_not_exist.sm");
    assert_eq!(code, 1);
    assert_eq!(json["status"], "error");
    assert_eq!(json["diagnostics"], json!([]));
    assert!(json["failure"]["message"]
        .as_str()
        .unwrap()
        .contains("does_not_exist.sm"));
}

#[test]
fn format_flag_rejects_unknown_values_and_mixed_legacy_flags() {
    let dir = fixture_dir("rootless");
    let (code, _, stderr) = smc(&dir, &["check", "ok.sm", "--format", "xml"]);
    assert_eq!(code, 1);
    assert!(stderr.contains("unknown --format value 'xml'"), "{stderr}");
    let (code, _, stderr) = smc(&dir, &["check", "ok.sm", "--format", "json", "--no-cache"]);
    assert_eq!(code, 1);
    assert!(
        stderr.contains("not supported together with --format"),
        "{stderr}"
    );
}

// ---------------------------------------------------------------- AC2

fn only_diag(json: &Value) -> &Value {
    let diags = json["diagnostics"].as_array().unwrap();
    assert_eq!(diags.len(), 1, "{json:#}");
    &diags[0]
}

fn range_text<'a>(source: &'a str, diag: &Value) -> &'a str {
    let start = diag["range"]["start"]["offset"].as_u64().unwrap() as usize;
    let end = diag["range"]["end"]["offset"].as_u64().unwrap() as usize;
    &source[start..end]
}

#[test]
fn frontend_syntax_error_carries_code_severity_source_and_token_range() {
    let dir = fixture_dir("rootless");
    let (_, json) = check_json(&dir, "syntax_error.sm");
    let d = only_diag(&json);
    assert_eq!(d["code"], "E0005");
    assert_eq!(d["severity"], "error");
    assert_eq!(d["family"], "frontend");
    assert_eq!(d["source"], 0);
    assert_eq!(json["sources"][0]["path"], "syntax_error.sm");
    let text = fs::read_to_string(dir.join("syntax_error.sm")).unwrap();
    assert_eq!(range_text(&text, d), "=");
    assert_eq!(d["range"]["start"]["line"], 2);
    assert_eq!(d["range"]["start"]["column"], 9);
}

#[test]
fn lexer_error_keeps_lexer_code_and_exact_range() {
    let dir = fixture_dir("rootless");
    let (_, json) = check_json(&dir, "lex_error.sm");
    let d = only_diag(&json);
    assert_eq!(d["code"], "E0004");
    assert_eq!(d["message"], "unterminated string literal");
    let text = fs::read_to_string(dir.join("lex_error.sm")).unwrap();
    assert_eq!(range_text(&text, d), "\"open");
}

#[test]
fn unprovable_ranges_are_absent_never_fabricated() {
    let dir = fixture_dir("rootless");
    for (file, code) in [
        ("type_error.sm", "E0201"),
        ("no_surface_claim.sm", "E0008"),
        ("ambiguous_surface.sm", "E0007"),
    ] {
        let (_, json) = check_json(&dir, file);
        let d = only_diag(&json);
        assert_eq!(d["code"], code, "{file}");
        assert_eq!(d["source"], 0, "{file}: the checked source is still known");
        assert_eq!(
            d["range"],
            Value::Null,
            "{file}: no producer range authority"
        );
    }
}

#[test]
fn retired_generic_code_is_never_emitted() {
    let dir = fixture_dir("rootless");
    for name in ROOTLESS {
        let (_, json) = check(&dir, &format!("{name}.sm"), "json");
        assert!(!json.contains("\"E0000\""), "{name} emitted retired E0000");
    }
}

#[test]
fn utf8_multibyte_prefix_does_not_skew_byte_ranges() {
    let dir = fixture_dir("rootless");
    let (_, json) = check_json(&dir, "utf16_syntax_error.sm");
    let d = only_diag(&json);
    let text = fs::read_to_string(dir.join("utf16_syntax_error.sm")).unwrap();
    assert_eq!(range_text(&text, d), "=");
    // Column counts Unicode scalar values: 4 + `let s: text = ` (14) +
    // the 6-scalar literal + `; let ` (6) -> column 31.
    assert_eq!(d["range"]["start"]["column"], 31);
}

#[test]
fn crlf_source_ranges_stay_exact() {
    let dir = tempdir("crlf");
    fs::write(dir.join("crlf.sm"), "fn main() {\r\n    let = 1;\r\n}\r\n").unwrap();
    let (_, json) = check_json(&dir, "crlf.sm");
    let d = only_diag(&json);
    let text = fs::read_to_string(dir.join("crlf.sm")).unwrap();
    assert_eq!(range_text(&text, d), "=");
    assert_eq!(d["range"]["start"]["line"], 2);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn logos_grammar_errors_keep_their_own_codes_and_recovered_errors() {
    // A Logos root is checked through the project mechanism: sm-sema's
    // E0239 module-load failure wraps the Logos parser's own structured
    // error (its own E02xx code, bare message, token range), and every
    // further recovered error is a related location - never a rendered
    // text block.
    let dir = fixture_dir("rootless");
    let (_, json) = check_json(&dir, "logos_recovered.sm");
    let d = only_diag(&json);
    assert_eq!(d["code"], "E0239");
    assert_eq!(d["message"], "failed to parse module");
    assert_eq!(d["cause"]["kind"], "diagnostic");
    let cause = &d["cause"]["diagnostics"][0];
    assert_eq!(cause["code"], "E0211");
    assert_eq!(cause["family"], "frontend");
    assert_eq!(cause["message"], "expected ':'");
    let related = cause["related"].as_array().unwrap();
    assert_eq!(related.len(), 1, "{cause:#}");
    assert_eq!(related[0]["source"], 0);
    assert_eq!(related[0]["message"], "E0211: expected ':'");
    assert_eq!(related[0]["range"]["start"]["line"], 2);
    for text in [&d["message"], &cause["message"], &related[0]["message"]] {
        let text = text.as_str().unwrap();
        assert!(!text.contains("-->") && !text.contains("<input>"), "{text}");
    }
}

#[test]
fn canonical_json_contains_no_absolute_fixture_path_anywhere() {
    let root = fixture_dir("");
    let root = root.canonicalize().unwrap_or(root);
    let root = root.to_string_lossy().replace('\\', "/");
    for project in ["project_ok", "project_broken"] {
        let (_, text) = check(&fixture_dir(project), "src/main.sm", "json");
        assert!(
            !text.contains(&root),
            "{project} leaked a host path: {text}"
        );
    }
    let dir = fixture_dir("rootless");
    for name in ROOTLESS {
        let (_, text) = check(&dir, &format!("{name}.sm"), "json");
        assert!(!text.contains(&root), "{name} leaked a host path");
    }
}

#[test]
fn schema_never_serializes_absolute_host_paths_in_structural_fields() {
    let root = fixture_dir("");
    let root = root.canonicalize().unwrap_or(root);
    let root = root.to_string_lossy().replace('\\', "/");
    for project in ["project_ok", "project_broken"] {
        let (_, json) = check_json(&fixture_dir(project), "src/main.sm");
        for source in json["sources"].as_array().unwrap() {
            let path = source["path"].as_str().unwrap();
            assert!(!path.starts_with('/') && !path.contains(':'), "{path}");
            assert!(!path.contains(&root));
            let module = source["identity"]["module"].as_str().unwrap();
            assert!(!module.starts_with('/') && !module.contains(&root));
        }
    }
}

#[test]
fn every_producer_code_is_catalogued_and_explainable() {
    // Registry completeness (Decision B): every code a producer can emit
    // into the canonical carrier has a catalog entry and `smc explain` text.
    let mut codes: Vec<String> = sm_front::diagnostic_authority::FRONTEND_DIAGNOSTIC_CODES
        .iter()
        .map(|c| c.to_string())
        .collect();
    codes.extend(
        sm_verify::diagnostic_authority::ALL_VERIFICATION_CODES
            .iter()
            .map(|c| c.code_token().to_string()),
    );
    codes.extend(
        sm_vm::diagnostic_admission::RUNTIME_DIAGNOSTIC_CODES
            .iter()
            .map(|c| c.to_string()),
    );
    codes.push("E0009".to_string());
    // Logos grammar codes are literals at the parser's error sites.
    let parser = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("crates/sm-front/src/parser.rs"),
    )
    .unwrap();
    let mut rest = parser.as_str();
    while let Some(i) = rest.find("\"E0") {
        let candidate = &rest[i + 1..i + 6];
        if candidate[1..].bytes().all(|b| b.is_ascii_digit()) && rest[i + 6..].starts_with('"') {
            codes.push(candidate.to_string());
        }
        rest = &rest[i + 1..];
    }
    assert!(codes.len() > 60, "registry scan found too few codes");
    // `smc explain` answers exactly from the diagnostic catalog, so an
    // explain text proves the catalog entry.
    let mirror =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/ERROR_CODES.md"))
            .unwrap();
    for code in codes {
        assert!(
            smc_cli::CliPipeline::explain(&code).is_some(),
            "{code} is emitted but `smc explain {code}` has no catalog text"
        );
        assert!(
            mirror.contains(&format!("- `{code}`:")),
            "{code} is missing from docs/ERROR_CODES.md"
        );
    }
}

// ---------------------------------------------------------------- AC5

#[test]
fn nested_module_identity_keeps_module_root_relative_path() {
    // M11 regression: the canonical identity of `src/util/helper.sm` is the
    // module-root-relative `util/helper.sm`, never the basename `helper.sm`.
    let (_, json) = check_json(&fixture_dir("project_ok"), "src/main.sm");
    let sources = json["sources"].as_array().unwrap();
    assert_eq!(sources.len(), 2, "{json:#}");
    assert_eq!(
        sources[0]["identity"],
        json!({"package": "demo", "module": "main.sm"})
    );
    assert_eq!(
        sources[1]["identity"],
        json!({"package": "demo", "module": "util/helper.sm"})
    );
    assert_ne!(sources[1]["identity"]["module"], "helper.sm");
    assert_eq!(sources[1]["path"], "src/util/helper.sm");
    let diags = json["diagnostics"].as_array().unwrap();
    assert!(!diags.is_empty());
    assert!(diags.iter().all(|d| d["source"] == 1));
}

#[test]
fn project_root_directory_resolves_like_its_entry() {
    let dir = fixture_dir("project_ok");
    let tmp = tempdir("project_root");
    copy_tree(&dir, &tmp);
    fs::write(
        tmp.join("semantic.toml"),
        "[project]\nentry = \"src/main.sm\"\n",
    )
    .unwrap();
    let (_, via_root) = check_json(&tmp, ".");
    let (_, via_entry) = check_json(&tmp, "src/main.sm");
    assert_eq!(via_root["diagnostics"], via_entry["diagnostics"]);
    assert_eq!(
        via_root["sources"][1]["identity"],
        via_entry["sources"][1]["identity"]
    );
    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn imported_module_failure_is_attributed_to_that_module() {
    let (_, json) = check_json(&fixture_dir("project_broken"), "src/main.sm");
    let d = only_diag(&json);
    assert_eq!(d["code"], "E0239");
    let index = d["source"].as_u64().unwrap() as usize;
    assert_eq!(
        json["sources"][index]["identity"]["module"],
        "util/helper.sm"
    );
    let text =
        fs::read_to_string(fixture_dir("project_broken").join("src/util/helper.sm")).unwrap();
    assert_eq!(range_text(&text, d), "x");
}

#[test]
fn rootless_source_has_no_fabricated_identity() {
    let (_, json) = check_json(&fixture_dir("rootless"), "syntax_error.sm");
    assert_eq!(json["sources"][0]["identity"], Value::Null);
}

// ---------------------------------------------------------------- LSP harness

fn frame(value: &Value) -> Vec<u8> {
    let body = serde_json::to_vec(value).unwrap();
    let mut out = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
    out.extend(body);
    out
}

fn run_lsp_raw(input: Vec<u8>) -> (LspExit, Vec<Value>) {
    let mut output = Vec::new();
    let exit = serve(std::io::Cursor::new(input), &mut output);
    let mut messages = Vec::new();
    let mut rest = output.as_slice();
    while !rest.is_empty() {
        let header_end = rest.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
        let header = std::str::from_utf8(&rest[..header_end]).unwrap();
        let length: usize = header
            .strip_prefix("Content-Length: ")
            .unwrap()
            .parse()
            .unwrap();
        let body = &rest[header_end + 4..header_end + 4 + length];
        messages.push(serde_json::from_slice(body).unwrap());
        rest = &rest[header_end + 4 + length..];
    }
    (exit, messages)
}

fn run_lsp(messages: &[Value]) -> (LspExit, Vec<Value>) {
    run_lsp_raw(messages.iter().flat_map(frame).collect())
}

fn file_uri(path: &Path) -> String {
    smc_cli::lsp::path_to_uri(&path.canonicalize().unwrap())
}

fn init(root: Option<&Path>) -> Vec<Value> {
    let root_uri = root.map(file_uri);
    vec![
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize",
               "params": {"rootUri": root_uri, "capabilities": {}}}),
        json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}),
    ]
}

fn open(uri: &str, version: i64, text: &str) -> Value {
    json!({"jsonrpc": "2.0", "method": "textDocument/didOpen",
           "params": {"textDocument": {"uri": uri, "languageId": "semantic",
                                       "version": version, "text": text}}})
}

fn change(uri: &str, version: i64, text: &str) -> Value {
    json!({"jsonrpc": "2.0", "method": "textDocument/didChange",
           "params": {"textDocument": {"uri": uri, "version": version},
                      "contentChanges": [{"text": text}]}})
}

fn shutdown_exit(id: i64) -> Vec<Value> {
    vec![
        json!({"jsonrpc": "2.0", "id": id, "method": "shutdown"}),
        json!({"jsonrpc": "2.0", "method": "exit"}),
    ]
}

fn publishes(messages: &[Value]) -> Vec<&Value> {
    messages
        .iter()
        .filter(|m| m["method"] == "textDocument/publishDiagnostics")
        .map(|m| &m["params"])
        .collect()
}

fn last_publish_for<'a>(messages: &'a [Value], uri: &str) -> &'a Value {
    publishes(messages)
        .into_iter()
        .rev()
        .find(|p| p["uri"] == uri)
        .unwrap_or_else(|| panic!("no publish for {uri}"))
}

fn response(messages: &[Value], id: i64) -> &Value {
    messages
        .iter()
        .find(|m| m["id"] == id && m.get("method").is_none())
        .unwrap_or_else(|| panic!("no response for id {id}: {messages:#?}"))
}

// ---------------------------------------------------------------- AC4

/// Every canonical diagnostic of `smc check --format json` for `entry`
/// appears in the LSP publications with the identical code, severity,
/// message and family, at the UTF-16 conversion of the identical byte
/// range, on the URI of its own source - and the LSP publishes nothing
/// else.
fn assert_cli_lsp_parity(dir: &Path, entry: &str) {
    let path = dir.join(entry);
    let (_, cli) = check_json(dir, entry);
    let uri = file_uri(&path);
    let text = fs::read_to_string(&path).unwrap();
    let mut msgs = init(Some(dir));
    msgs.push(open(&uri, 1, &text));
    msgs.extend(shutdown_exit(9));
    let (exit, out) = run_lsp(&msgs);
    assert_eq!(exit, LspExit::Clean);

    let mut expected = Vec::new();
    for d in cli["diagnostics"].as_array().unwrap() {
        let (target, range) = match d["source"].as_u64() {
            Some(index) => {
                let source = &cli["sources"][index as usize];
                let source_path = dir.join(source["path"].as_str().unwrap());
                let source_text = fs::read_to_string(&source_path).unwrap();
                let target = if index == 0 {
                    uri.clone()
                } else {
                    file_uri(&source_path)
                };
                let range = if d["range"].is_null() {
                    None
                } else {
                    let s = d["range"]["start"]["offset"].as_u64().unwrap() as usize;
                    let e = d["range"]["end"]["offset"].as_u64().unwrap() as usize;
                    Some((
                        utf16_position(&source_text, s),
                        utf16_position(&source_text, e),
                    ))
                };
                (target, range)
            }
            None => (uri.clone(), None),
        };
        expected.push((
            target,
            d["code"].as_str().unwrap().to_string(),
            d["message"].as_str().unwrap().to_string(),
            if d["severity"] == "error" { 1 } else { 2 },
            d["family"].as_str().unwrap().to_string(),
            range,
        ));
    }

    let mut actual = Vec::new();
    for p in publishes(&out) {
        for d in p["diagnostics"].as_array().unwrap() {
            let range = if d["data"]["rangeAbsent"] == true {
                None
            } else {
                let r = &d["range"];
                Some((
                    (
                        r["start"]["line"].as_u64().unwrap() as usize,
                        r["start"]["character"].as_u64().unwrap() as usize,
                    ),
                    (
                        r["end"]["line"].as_u64().unwrap() as usize,
                        r["end"]["character"].as_u64().unwrap() as usize,
                    ),
                ))
            };
            actual.push((
                p["uri"].as_str().unwrap().to_string(),
                d["code"].as_str().unwrap().to_string(),
                d["message"].as_str().unwrap().to_string(),
                d["severity"].as_i64().unwrap(),
                d["data"]["family"].as_str().unwrap().to_string(),
                range,
            ));
        }
    }
    assert_eq!(actual, expected, "CLI/LSP parity for {entry}");
}

#[test]
fn cli_lsp_parity_rootless_matrix() {
    let dir = fixture_dir("rootless");
    for name in ROOTLESS {
        assert_cli_lsp_parity(&dir, &format!("{name}.sm"));
    }
}

#[test]
fn cli_lsp_parity_project_matrix() {
    for project in ["project_ok", "project_broken"] {
        assert_cli_lsp_parity(&fixture_dir(project), "src/main.sm");
        assert_cli_lsp_parity(&fixture_dir(project), "src/util/helper.sm");
    }
}

#[test]
fn lsp_routes_imported_module_diagnostics_to_that_module_uri() {
    // M12b regression: a diagnostic of another source in the same check is
    // published on that source's URI, never dropped.
    let dir = fixture_dir("project_broken");
    let main = dir.join("src/main.sm");
    let helper_uri = file_uri(&dir.join("src/util/helper.sm"));
    let mut msgs = init(Some(&dir));
    msgs.push(open(
        &file_uri(&main),
        1,
        &fs::read_to_string(&main).unwrap(),
    ));
    msgs.extend(shutdown_exit(9));
    let (_, out) = run_lsp(&msgs);
    let helper = last_publish_for(&out, &helper_uri);
    assert_eq!(helper["diagnostics"][0]["code"], "E0239");
    assert_eq!(
        last_publish_for(&out, &file_uri(&main))["diagnostics"],
        json!([])
    );
}

#[test]
fn lsp_uses_open_document_overlay_not_disk() {
    // M12a regression: an unsaved editor buffer is what gets checked.
    let dir = fixture_dir("rootless");
    let uri = file_uri(&dir.join("ok.sm"));
    let mut msgs = init(Some(&dir));
    msgs.push(open(&uri, 1, "fn main() {\n    let = 1;\n}\n"));
    msgs.extend(shutdown_exit(9));
    let (_, out) = run_lsp(&msgs);
    let publish = last_publish_for(&out, &uri);
    assert_eq!(publish["version"], 1);
    assert_eq!(publish["diagnostics"][0]["code"], "E0005");
    assert_eq!(
        publish["diagnostics"][0]["range"],
        json!({"start": {"line": 1, "character": 8}, "end": {"line": 1, "character": 9}})
    );
}

#[test]
fn lsp_overlay_of_imported_module_changes_importer_result() {
    let dir = fixture_dir("project_ok");
    let main = dir.join("src/main.sm");
    let helper = dir.join("src/util/helper.sm");
    let mut msgs = init(Some(&dir));
    msgs.push(open(
        &file_uri(&main),
        1,
        &fs::read_to_string(&main).unwrap(),
    ));
    msgs.push(open(
        &file_uri(&helper),
        1,
        "Entity A:\n    state x: quad\nLaw \"L\" [priority x]:\n    When N ->\n        Pulse.emit(\"x\")\n",
    ));
    msgs.extend(shutdown_exit(9));
    let (_, out) = run_lsp(&msgs);
    let helper_publish = last_publish_for(&out, &file_uri(&helper));
    assert_eq!(helper_publish["version"], 1);
    let codes: Vec<&str> = helper_publish["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["code"].as_str().unwrap())
        .collect();
    assert_eq!(codes, vec!["E0239"], "deduplicated across both roots");
}

#[test]
fn lsp_utf16_positions_count_code_units() {
    let dir = fixture_dir("rootless");
    let path = dir.join("utf16_syntax_error.sm");
    let uri = file_uri(&path);
    let mut msgs = init(Some(&dir));
    msgs.push(open(&uri, 1, &fs::read_to_string(&path).unwrap()));
    msgs.extend(shutdown_exit(9));
    let (_, out) = run_lsp(&msgs);
    // `    let s: text = "日本語🎉"; let ` : 4+14+1+3+2(🎉 is two units)+1+6
    assert_eq!(
        last_publish_for(&out, &uri)["diagnostics"][0]["range"],
        json!({"start": {"line": 1, "character": 31}, "end": {"line": 1, "character": 32}})
    );
}

#[test]
fn lsp_projects_recovered_errors_as_related_information() {
    let dir = fixture_dir("rootless");
    let path = dir.join("logos_recovered.sm");
    let uri = file_uri(&path);
    let mut msgs = init(Some(&dir));
    msgs.push(open(&uri, 1, &fs::read_to_string(&path).unwrap()));
    msgs.extend(shutdown_exit(9));
    let (_, out) = run_lsp(&msgs);
    let d = &last_publish_for(&out, &uri)["diagnostics"][0];
    assert_eq!(d["data"]["cause"][0]["code"], "E0211");
    let related = d["relatedInformation"]
        .as_array()
        .expect("relatedInformation");
    assert_eq!(related.len(), 2);
    assert_eq!(related[0]["message"], "caused by E0211: expected ':'");
    assert_eq!(related[1]["message"], "E0211: expected ':'");
    assert_eq!(related[1]["location"]["uri"], uri);
    assert_eq!(related[1]["location"]["range"]["start"]["line"], 1);
}

#[test]
fn lsp_absent_range_is_marked_not_claimed() {
    let dir = fixture_dir("rootless");
    let path = dir.join("type_error.sm");
    let uri = file_uri(&path);
    let mut msgs = init(Some(&dir));
    msgs.push(open(&uri, 1, &fs::read_to_string(&path).unwrap()));
    msgs.extend(shutdown_exit(9));
    let (_, out) = run_lsp(&msgs);
    let d = &last_publish_for(&out, &uri)["diagnostics"][0];
    assert_eq!(d["code"], "E0201");
    assert_eq!(d["data"]["rangeAbsent"], true);
    assert_eq!(d["data"]["sourceBound"], true);
}

#[test]
fn lsp_non_file_uri_is_checked_as_standalone_text() {
    let uri = "untitled:Untitled-1";
    let mut msgs = init(None);
    msgs.push(open(uri, 3, "fn main() {\n    let = 1;\n}\n"));
    msgs.extend(shutdown_exit(9));
    let (_, out) = run_lsp(&msgs);
    let publish = last_publish_for(&out, uri);
    assert_eq!(publish["version"], 3);
    assert_eq!(publish["diagnostics"][0]["code"], "E0005");
}

#[test]
fn lsp_unsaved_file_uri_is_checked_standalone() {
    let dir = fixture_dir("rootless");
    let uri = smc_cli::lsp::path_to_uri(&dir.canonicalize().unwrap().join("not_saved_yet.sm"));
    let mut msgs = init(Some(&dir));
    msgs.push(open(&uri, 1, "fn main() {\n    return;\n}\n"));
    msgs.extend(shutdown_exit(9));
    let (_, out) = run_lsp(&msgs);
    assert_eq!(last_publish_for(&out, &uri)["diagnostics"], json!([]));
}

#[test]
fn lsp_transcript_is_byte_deterministic() {
    let dir = fixture_dir("project_ok");
    let main = dir.join("src/main.sm");
    let mut msgs = init(Some(&dir));
    msgs.push(open(
        &file_uri(&main),
        1,
        &fs::read_to_string(&main).unwrap(),
    ));
    msgs.push(change(&file_uri(&main), 2, "Law \"Root\" [priority 1]:\n"));
    msgs.extend(shutdown_exit(9));
    let input: Vec<u8> = msgs.iter().flat_map(frame).collect();
    let mut first = Vec::new();
    serve(std::io::Cursor::new(input.clone()), &mut first);
    let mut second = Vec::new();
    serve(std::io::Cursor::new(input), &mut second);
    assert_eq!(first, second);
}

// ---------------------------------------------------------------- stale / cancel / lifecycle

#[test]
fn lsp_stale_did_change_is_rejected_and_changes_nothing() {
    // M9 regression.
    let dir = fixture_dir("rootless");
    let uri = file_uri(&dir.join("ok.sm"));
    let mut msgs = init(Some(&dir));
    msgs.push(open(&uri, 5, "fn main() {\n    return;\n}\n"));
    msgs.push(change(&uri, 5, "fn main() {\n    let = 1;\n}\n"));
    msgs.push(change(&uri, 4, "fn main() {\n    let = 1;\n}\n"));
    msgs.extend(shutdown_exit(9));
    let (_, out) = run_lsp(&msgs);
    let ours: Vec<&Value> = publishes(&out)
        .into_iter()
        .filter(|p| p["uri"] == uri)
        .collect();
    assert_eq!(ours.len(), 1, "stale changes must not republish");
    assert_eq!(ours[0]["diagnostics"], json!([]));
    let logs: Vec<&Value> = out
        .iter()
        .filter(|m| m["method"] == "window/logMessage")
        .collect();
    assert_eq!(logs.len(), 2);
    assert!(logs[0]["params"]["message"]
        .as_str()
        .unwrap()
        .contains("stale"));
}

#[test]
fn lsp_newer_change_republishes_with_its_version() {
    let dir = fixture_dir("rootless");
    let uri = file_uri(&dir.join("ok.sm"));
    let mut msgs = init(Some(&dir));
    msgs.push(open(&uri, 1, "fn main() {\n    return;\n}\n"));
    msgs.push(change(&uri, 2, "fn main() {\n    let = 1;\n}\n"));
    msgs.push(change(&uri, 3, "fn main() {\n    return;\n}\n"));
    msgs.extend(shutdown_exit(9));
    let (_, out) = run_lsp(&msgs);
    let versions: Vec<(i64, usize)> = publishes(&out)
        .into_iter()
        .filter(|p| p["uri"] == uri)
        .map(|p| {
            (
                p["version"].as_i64().unwrap(),
                p["diagnostics"].as_array().unwrap().len(),
            )
        })
        .collect();
    assert_eq!(versions, vec![(1, 0), (2, 1), (3, 0)]);
}

#[test]
fn lsp_incremental_change_is_rejected() {
    let dir = fixture_dir("rootless");
    let uri = file_uri(&dir.join("ok.sm"));
    let mut msgs = init(Some(&dir));
    msgs.push(open(&uri, 1, "fn main() {\n    return;\n}\n"));
    msgs.push(json!({"jsonrpc": "2.0", "method": "textDocument/didChange",
        "params": {"textDocument": {"uri": uri, "version": 2},
                   "contentChanges": [{"range": {"start": {"line": 0, "character": 0},
                                                 "end": {"line": 0, "character": 0}},
                                       "text": "x"}]}}));
    msgs.extend(shutdown_exit(9));
    let (_, out) = run_lsp(&msgs);
    assert_eq!(publishes(&out).len(), 1);
    assert!(out.iter().any(|m| m["method"] == "window/logMessage"
        && m["params"]["message"]
            .as_str()
            .unwrap()
            .contains("full-document")));
}

#[test]
fn lsp_cancel_request_for_answered_request_is_a_noop() {
    let mut msgs = init(None);
    msgs.push(json!({"jsonrpc": "2.0", "method": "$/cancelRequest", "params": {"id": 1}}));
    msgs.push(json!({"jsonrpc": "2.0", "method": "$/cancelRequest", "params": {"id": 77}}));
    msgs.extend(shutdown_exit(9));
    let (exit, out) = run_lsp(&msgs);
    assert_eq!(exit, LspExit::Clean);
    assert_eq!(
        out.len(),
        2,
        "only initialize and shutdown responses: {out:#?}"
    );
}

#[test]
fn lsp_did_close_clears_published_diagnostics() {
    let dir = fixture_dir("rootless");
    let uri = file_uri(&dir.join("ok.sm"));
    let mut msgs = init(Some(&dir));
    msgs.push(open(&uri, 1, "fn main() {\n    let = 1;\n}\n"));
    msgs.push(json!({"jsonrpc": "2.0", "method": "textDocument/didClose",
                     "params": {"textDocument": {"uri": uri}}}));
    msgs.extend(shutdown_exit(9));
    let (_, out) = run_lsp(&msgs);
    let last = last_publish_for(&out, &uri);
    assert_eq!(last["diagnostics"], json!([]));
    assert!(last.get("version").is_none());
}

#[test]
fn lsp_workspace_folder_change_republishes_deterministically() {
    let dir = fixture_dir("project_ok");
    let main = dir.join("src/main.sm");
    let uri = file_uri(&main);
    let mut msgs = init(None);
    msgs.push(open(&uri, 1, &fs::read_to_string(&main).unwrap()));
    msgs.push(
        json!({"jsonrpc": "2.0", "method": "workspace/didChangeWorkspaceFolders",
        "params": {"event": {"added": [{"uri": file_uri(&dir), "name": "demo"}], "removed": []}}}),
    );
    msgs.push(
        json!({"jsonrpc": "2.0", "method": "workspace/didChangeWorkspaceFolders",
        "params": {"event": {"added": [], "removed": [{"uri": file_uri(&dir), "name": "demo"}]}}}),
    );
    msgs.extend(shutdown_exit(9));
    let (_, out) = run_lsp(&msgs);
    let helper_uri = file_uri(&dir.join("src/util/helper.sm"));
    let helper: Vec<&Value> = publishes(&out)
        .into_iter()
        .filter(|p| p["uri"] == helper_uri)
        .collect();
    assert_eq!(helper.len(), 3);
    // Identity is package authority, not LSP root: every republish agrees.
    assert_eq!(helper[0], helper[1]);
    assert_eq!(helper[1], helper[2]);
}

#[test]
fn lsp_exit_without_shutdown_is_unclean() {
    let mut msgs = init(None);
    msgs.push(json!({"jsonrpc": "2.0", "method": "exit"}));
    assert_eq!(run_lsp(&msgs).0, LspExit::Unclean);
    assert_eq!(run_lsp(&init(None)).0, LspExit::Unclean, "EOF without exit");
}

#[test]
fn lsp_requests_after_shutdown_are_invalid() {
    let mut msgs = init(None);
    msgs.push(json!({"jsonrpc": "2.0", "id": 2, "method": "shutdown"}));
    msgs.push(
        json!({"jsonrpc": "2.0", "id": 3, "method": "textDocument/formatting",
                     "params": {"textDocument": {"uri": "untitled:x"}}}),
    );
    msgs.push(json!({"jsonrpc": "2.0", "method": "exit"}));
    let (exit, out) = run_lsp(&msgs);
    assert_eq!(exit, LspExit::Clean);
    assert_eq!(response(&out, 3)["error"]["code"], -32600);
}

#[test]
fn lsp_deferred_features_are_not_advertised_and_deterministically_rejected() {
    let mut msgs = init(None);
    msgs.push(open("untitled:d", 1, "fn main() {\n    return;\n}\n"));
    let position = json!({"textDocument": {"uri": "untitled:d"},
                          "position": {"line": 0, "character": 3}});
    for (id, method) in [
        (2, "textDocument/hover"),
        (3, "textDocument/definition"),
        (4, "textDocument/documentSymbol"),
        (5, "textDocument/codeAction"),
        (6, "textDocument/completion"),
    ] {
        msgs.push(json!({"jsonrpc": "2.0", "id": id, "method": method, "params": position}));
    }
    msgs.extend(shutdown_exit(9));
    let (_, out) = run_lsp(&msgs);
    let caps = &response(&out, 1)["result"]["capabilities"];
    for key in [
        "hoverProvider",
        "definitionProvider",
        "documentSymbolProvider",
        "codeActionProvider",
        "completionProvider",
    ] {
        assert!(caps.get(key).is_none(), "{key} must not be advertised");
    }
    for id in 2..=6 {
        assert_eq!(response(&out, id)["error"]["code"], -32601);
    }
}

// ---------------------------------------------------------------- malformed protocol

#[test]
fn lsp_request_before_initialize_is_rejected() {
    let (_, out) = run_lsp(&[
        json!({"jsonrpc": "2.0", "id": 7, "method": "textDocument/formatting",
               "params": {"textDocument": {"uri": "untitled:x"}}}),
        json!({"jsonrpc": "2.0", "method": "exit"}),
    ]);
    assert_eq!(response(&out, 7)["error"]["code"], -32002);
}

#[test]
fn lsp_malformed_messages_get_json_rpc_errors_and_session_continues() {
    let mut input = Vec::new();
    for m in init(None) {
        input.extend(frame(&m));
    }
    let bad_bodies: [&[u8]; 4] = [
        b"{not json",
        b"[1,2,3]",
        br#"{"id": 5, "method": "shutdown"}"#,
        br#"{"jsonrpc": "2.0", "id": {"x": 1}, "method": "shutdown"}"#,
    ];
    for body in bad_bodies {
        input.extend(format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes());
        input.extend_from_slice(body);
    }
    input.extend(frame(
        &json!({"jsonrpc": "2.0", "id": 8, "method": "no/such"}),
    ));
    for m in shutdown_exit(9) {
        input.extend(frame(&m));
    }
    let (exit, out) = run_lsp_raw(input);
    assert_eq!(exit, LspExit::Clean);
    let codes: Vec<i64> = out
        .iter()
        .filter_map(|m| m["error"]["code"].as_i64())
        .collect();
    assert_eq!(codes, vec![-32700, -32600, -32600, -32600, -32601]);
    assert_eq!(response(&out, 9)["result"], Value::Null);
}

#[test]
fn lsp_invalid_utf8_body_is_a_parse_error() {
    let mut input = Vec::new();
    for m in init(None) {
        input.extend(frame(&m));
    }
    input.extend(b"Content-Length: 3\r\n\r\n\xff\xfe\xfd");
    for m in shutdown_exit(9) {
        input.extend(frame(&m));
    }
    let (exit, out) = run_lsp_raw(input);
    assert_eq!(exit, LspExit::Clean);
    assert!(out.iter().any(|m| m["error"]["code"] == -32700));
}

#[test]
fn lsp_framing_errors_end_the_session_deterministically() {
    let cases: [(&[u8], &str); 5] = [
        (b"Content-Type: x\r\n\r\n{}", "missing Content-Length"),
        (b"Content-Length: abc\r\n\r\n{}", "invalid Content-Length"),
        (b"Content-Length: 10\r\n\r\n{}", "inside message body"),
        (b"Content-Length: 2\n\n{}", "not CRLF-terminated"),
        (b"Content-Length: 99999999999\r\n\r\n", "exceeds"),
    ];
    for (input, expected) in cases {
        match run_lsp_raw(input.to_vec()).0 {
            LspExit::Framing(message) => assert!(message.contains(expected), "{message}"),
            other => panic!("expected framing error for {expected}, got {other:?}"),
        }
    }
}

// ---------------------------------------------------------------- AC3 formatter

fn all_fixture_sources() -> Vec<PathBuf> {
    let mut out = Vec::new();
    for root in [FIXTURES, "examples/canonical"] {
        collect_sm(&Path::new(env!("CARGO_MANIFEST_DIR")).join(root), &mut out);
    }
    out.sort();
    out
}

fn collect_sm(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_sm(&path, out);
        } else if path.extension().is_some_and(|e| e == "sm") {
            out.push(path);
        }
    }
}

#[test]
fn formatter_is_idempotent_and_deterministic_on_all_fixtures() {
    let sources = all_fixture_sources();
    assert!(sources.len() >= 10, "fixture corpus unexpectedly small");
    for path in sources {
        let text = fs::read_to_string(&path).unwrap();
        let once = smc_cli::format_source_text(&text);
        assert_eq!(
            smc_cli::format_source_text(&once),
            once,
            "{}",
            path.display()
        );
        assert_eq!(
            smc_cli::format_source_text(&text),
            once,
            "{}",
            path.display()
        );
    }
}

#[test]
fn formatter_preserves_check_result_of_messy_sources() {
    // Semantic preservation evidence: the canonical check result of a
    // source is identical before and after canonical formatting.
    let dir = tempdir("fmt_semantics");
    let cases = [
        "fn main() {   \r\n    let x: i32 = 1;\t\r\n    return;\r\n}\r\n\r\n\r\n",
        "Entity A:  \n    state x: quad\t\nLaw \"L\" [priority 1]:\n    When N ->   \n        Pulse.emit(\"x\")\n\n",
        "fn main() {  \n    let = 1;\n}",
    ];
    for (i, text) in cases.iter().enumerate() {
        let file = format!("case{i}.sm");
        fs::write(dir.join(&file), text).unwrap();
        let (_, before) = check_json(&dir, &file);
        let (code, _, _) = smc(&dir, &["fmt", &file]);
        assert_eq!(code, 0, "fmt {file}");
        let formatted = fs::read_to_string(dir.join(&file)).unwrap();
        assert_ne!(&formatted, text);
        let (_, after) = check_json(&dir, &file);
        let strip = |v: &Value| {
            v["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .map(|d| {
                    (
                        d["code"].clone(),
                        d["message"].clone(),
                        d["range"]["start"]["line"].clone(),
                    )
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(strip(&before), strip(&after), "{file}");
        assert_eq!(before["status"], after["status"]);
    }
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn lsp_formatting_bridge_equals_cli_formatter_including_refusals() {
    // M10 regression: the LSP returns exactly the canonical formatter's
    // result, and refuses exactly when it refuses.
    let messy = "fn main() {   \n    return;\t\n}\n\n";
    let unsafe_text = "fn main() {  \n    let s: text = \"a\rb\";\n}\n";
    let uri_a = "untitled:a";
    let uri_b = "untitled:b";
    let uri_c = "untitled:c";
    let mut msgs = init(None);
    msgs.push(open(uri_a, 1, messy));
    msgs.push(open(uri_b, 1, unsafe_text));
    msgs.push(open(uri_c, 1, "fn main() {\n    return;\n}\n"));
    for (id, uri) in [(2, uri_a), (3, uri_b), (4, uri_c)] {
        msgs.push(
            json!({"jsonrpc": "2.0", "id": id, "method": "textDocument/formatting",
                         "params": {"textDocument": {"uri": uri},
                                    "options": {"tabSize": 4, "insertSpaces": true}}}),
        );
    }
    msgs.extend(shutdown_exit(9));
    let (_, out) = run_lsp(&msgs);
    let edits = &response(&out, 2)["result"];
    assert_eq!(edits[0]["newText"], smc_cli::format_source_text(messy));
    assert_eq!(
        edits[0]["range"],
        json!({"start": {"line": 0, "character": 0}, "end": {"line": 4, "character": 0}})
    );
    assert!(smc_cli::format_source_checked(unsafe_text).is_err());
    assert_eq!(response(&out, 3)["error"]["code"], -32803);
    assert_eq!(response(&out, 4)["result"], json!([]));

    let dir = tempdir("fmt_refusal_parity");
    fs::write(dir.join("u.sm"), unsafe_text).unwrap();
    let (code, _, stderr) = smc(&dir, &["fmt", "--check", "u.sm"]);
    assert_eq!(code, 1);
    assert!(stderr.contains("refusing to format"), "{stderr}");
    let _ = fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------- AC6

const NETWORK_TOKENS: [&str; 10] = [
    "std::net",
    "TcpStream",
    "TcpListener",
    "UdpSocket",
    "reqwest",
    "hyper",
    "ureq",
    "http://",
    "https://",
    "telemetry::",
];

#[test]
fn diagnostic_and_editor_surfaces_have_no_network_or_telemetry_code() {
    let files = [
        "crates/sm-diagnostic/src/lib.rs",
        "crates/sm-front/src/diagnostic_authority.rs",
        "crates/sm-verify/src/diagnostic_authority.rs",
        "crates/sm-vm/src/diagnostic_admission.rs",
        "crates/smc-cli/src/canonical_check.rs",
        "crates/smc-cli/src/diagnostic_schema.rs",
        "crates/smc-cli/src/lsp.rs",
        "crates/smc-cli/src/formatter.rs",
    ];
    for file in files {
        let text = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(file)).unwrap();
        for token in NETWORK_TOKENS {
            assert!(
                !text.contains(token),
                "{file} contains network/telemetry token {token}"
            );
        }
    }
}

#[test]
fn editor_host_crates_declare_no_network_dependencies() {
    for manifest in [
        "crates/smc-cli/Cargo.toml",
        "crates/sm-diagnostic/Cargo.toml",
    ] {
        let text =
            fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(manifest)).unwrap();
        for dep in [
            "reqwest",
            "hyper",
            "ureq",
            "curl",
            "tokio",
            "async-std",
            "sentry",
            "opentelemetry",
        ] {
            assert!(
                !text.lines().any(|l| l.trim_start().starts_with(dep)),
                "{manifest} declares network-capable dependency {dep}"
            );
        }
    }
}

#[test]
fn lsp_writes_only_protocol_frames_to_its_output() {
    let dir = fixture_dir("project_ok");
    let main = dir.join("src/main.sm");
    let mut msgs = init(Some(&dir));
    msgs.push(open(
        &file_uri(&main),
        1,
        &fs::read_to_string(&main).unwrap(),
    ));
    msgs.extend(shutdown_exit(9));
    let input: Vec<u8> = msgs.iter().flat_map(frame).collect();
    let mut output = Vec::new();
    serve(std::io::Cursor::new(input), &mut output);
    // Every byte of output belongs to a well-formed frame.
    let mut rest = output.as_slice();
    while !rest.is_empty() {
        assert!(rest.starts_with(b"Content-Length: "));
        let header_end = rest.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
        let length: usize = std::str::from_utf8(&rest[16..header_end])
            .unwrap()
            .parse()
            .unwrap();
        rest = &rest[header_end + 4 + length..];
    }
}

// ---------------------------------------------------------------- AC7 binary path

#[test]
fn smc_lsp_binary_speaks_the_protocol_over_stdio() {
    use std::io::Write as _;
    use std::process::Stdio;
    let dir = fixture_dir("rootless");
    let path = dir.join("syntax_error.sm");
    let uri = file_uri(&path);
    let mut msgs = init(Some(&dir));
    msgs.push(open(&uri, 1, &fs::read_to_string(&path).unwrap()));
    msgs.extend(shutdown_exit(9));
    let mut child = Command::new(env!("CARGO_BIN_EXE_smc"))
        .args(["lsp", "--stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn smc lsp");
    let input: Vec<u8> = msgs.iter().flat_map(frame).collect();
    child.stdin.take().unwrap().write_all(&input).unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stderr.is_empty());
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("\"code\":\"E0005\""), "{text}");
    assert!(text.contains("\"serverInfo\""), "{text}");
}

#[test]
fn smc_lsp_rejects_unknown_flags() {
    let (code, _, stderr) = smc(Path::new(env!("CARGO_MANIFEST_DIR")), &["lsp", "--tcp"]);
    assert_eq!(code, 1);
    assert!(stderr.contains("unknown flag '--tcp'"), "{stderr}");
}

// ---------------------------------------------------------------- helpers

fn tempdir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "ssf09_{tag}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn copy_tree(from: &Path, to: &Path) {
    for entry in fs::read_dir(from).unwrap().flatten() {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            fs::create_dir_all(&target).unwrap();
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

// ---------------------------------------------------------------- R3 repairs

/// The same project checked from two different absolute checkout roots.
const PORTABILITY_CASES: [(&str, &str, &str); 5] = [
    ("cycle", "src/a.sm", "E0238"),
    ("missing_import", "src/main.sm", "E0239"),
    ("select_missing", "src/main.sm", "E0244"),
    ("bundle_missing", "main.sm", "E0009"),
    ("rootless", "missing_import.sm", "E0239"),
];

#[test]
fn canonical_json_is_byte_identical_across_checkout_roots() {
    // R3 blocker #1: canonical machine output must not depend on where the
    // checkout lives - no absolute host path, no OS-specific I/O wording.
    let fixture = fixture_dir("portability");
    let shallow = tempdir("portable_a");
    let deep = tempdir("portable_b")
        .join("much")
        .join("deeper")
        .join("checkout");
    fs::create_dir_all(&deep).unwrap();
    copy_tree(&fixture, &shallow);
    copy_tree(&fixture, &deep);
    for (case, entry, code) in PORTABILITY_CASES {
        let (_, a) = check(&shallow.join(case), entry, "json");
        let (_, b) = check(&deep.join(case), entry, "json");
        assert_eq!(
            a, b,
            "{case}: canonical JSON differs between checkout roots"
        );
        let parsed: Value = serde_json::from_str(&a).unwrap();
        assert_eq!(parsed["diagnostics"][0]["code"], code, "{case}: {a}");
        for root in [&shallow, &deep] {
            let root = root.canonicalize().unwrap();
            let root = root.to_string_lossy().replace('\\', "/");
            assert!(!a.contains(&root), "{case}: host path leaked: {a}");
        }
        assert!(!a.contains("os error"), "{case}: OS error text leaked: {a}");
        assert!(
            !a.contains("No such file"),
            "{case}: OS error text leaked: {a}"
        );
    }
    let _ = fs::remove_dir_all(&shallow);
    let _ = fs::remove_dir_all(deep.ancestors().nth(3).unwrap());
}

#[test]
fn portable_messages_name_modules_by_project_relative_path() {
    let dir = fixture_dir("portability");
    let cases = [
        (
            "cycle",
            "src/a.sm",
            "cyclic import detected: a.sm -> b.sm -> a.sm",
        ),
        (
            "missing_import",
            "src/main.sm",
            "failed to read import 'gone.sm': module file does not exist or cannot be resolved",
        ),
        (
            "select_missing",
            "src/main.sm",
            "selected import symbol 'Zed' not found in 'b.sm'",
        ),
        (
            "bundle_missing",
            "main.sm",
            "module 'helper.sm': module file does not exist or cannot be resolved",
        ),
    ];
    for (case, entry, message) in cases {
        let (_, json) = check_json(&dir.join(case), entry);
        assert_eq!(json["diagnostics"][0]["message"], message, "{case}");
    }
    // A select failure is bound to the importing module's own source.
    let (_, json) = check_json(&dir.join("select_missing"), "src/main.sm");
    assert_eq!(json["diagnostics"][0]["source"], 0);
}

#[test]
fn legacy_check_output_keeps_host_detail() {
    // Outside the canonical path the legacy rendering is unchanged.
    let dir = fixture_dir("portability").join("missing_import");
    let (code, _, stderr) = smc(&dir, &["check", "src/main.sm"]);
    assert_eq!(code, 1);
    let root = dir.canonicalize().unwrap();
    assert!(
        stderr.contains(&*root.to_string_lossy().replace('\\', "/"))
            || stderr.contains(&*root.to_string_lossy()),
        "{stderr}"
    );
}

/// Drives the real `smc lsp` binary over stdio with `messages`.
fn run_binary_lsp(messages: &[Value]) -> (Option<i32>, Vec<Value>) {
    use std::io::Write as _;
    use std::process::Stdio;
    let mut child = Command::new(env!("CARGO_BIN_EXE_smc"))
        .args(["lsp", "--stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn smc lsp");
    let input: Vec<u8> = messages.iter().flat_map(frame).collect();
    child.stdin.take().unwrap().write_all(&input).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(
        out.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let mut values = Vec::new();
    let mut rest = out.stdout.as_slice();
    while !rest.is_empty() {
        let header_end = rest.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
        let length: usize = std::str::from_utf8(&rest[16..header_end])
            .unwrap()
            .parse()
            .unwrap();
        values
            .push(serde_json::from_slice(&rest[header_end + 4..header_end + 4 + length]).unwrap());
        rest = &rest[header_end + 4 + length..];
    }
    (out.status.code(), values)
}

fn codes_of(publish: &Value) -> Vec<String> {
    publish["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["code"].as_str().unwrap_or("<none>").to_string())
        .collect()
}

#[test]
fn lsp_unsaved_rustlike_helper_is_seen_by_its_importer_over_stdio() {
    // R3 blocker #2: RustLike executable helpers are read through the same
    // overlay-aware seam as every other project source.
    let dir = fixture_dir("rustlike_helper");
    let main = dir.join("main.sm");
    let helper = dir.join("helper.sm");
    let main_uri = file_uri(&main);
    let helper_uri = file_uri(&helper);
    let broken_helper = "fn helper() -> i32 {\n    return true;\n}\n";
    let close = |uri: &str| {
        json!({"jsonrpc": "2.0", "method": "textDocument/didClose",
               "params": {"textDocument": {"uri": uri}}})
    };
    let mut msgs = init(Some(&dir));
    msgs.push(open(&main_uri, 1, &fs::read_to_string(&main).unwrap()));
    msgs.push(open(&helper_uri, 1, broken_helper));
    msgs.push(close(&helper_uri));
    msgs.push(open(&helper_uri, 2, broken_helper));
    msgs.extend(shutdown_exit(9));
    let (code, out) = run_binary_lsp(&msgs);
    assert_eq!(code, Some(0));
    let main_publishes: Vec<Vec<String>> = publishes(&out)
        .into_iter()
        .filter(|p| p["uri"] == main_uri)
        .map(codes_of)
        .collect();
    assert_eq!(
        main_publishes,
        vec![
            vec![],                    // saved helper: clean
            vec!["E0201".to_string()], // unsaved helper error seen by importer
            vec![],                    // helper closed: back to disk truth
            vec!["E0201".to_string()], // reopened: overlay restored
        ],
        "{out:#?}"
    );
    let with_error = publishes(&out)
        .into_iter()
        .find(|p| p["uri"] == main_uri && !codes_of(p).is_empty())
        .unwrap();
    assert!(with_error["diagnostics"][0]["message"]
        .as_str()
        .unwrap()
        .contains("return type mismatch"));
}

#[test]
fn lsp_rustlike_overlay_matches_cli_on_the_same_effective_sources() {
    // Parity against the same effective source state: the LSP result with
    // an unsaved helper equals the CLI result once that text is on disk.
    let broken_helper = "fn helper() -> i32 {\n    return true;\n}\n";
    let dir = tempdir("rl_parity");
    copy_tree(&fixture_dir("rustlike_helper"), &dir);
    let mut msgs = init(Some(&dir));
    let main_uri = file_uri(&dir.join("main.sm"));
    msgs.push(open(
        &main_uri,
        1,
        &fs::read_to_string(dir.join("main.sm")).unwrap(),
    ));
    msgs.push(open(&file_uri(&dir.join("helper.sm")), 1, broken_helper));
    msgs.extend(shutdown_exit(9));
    let (_, out) = run_lsp(&msgs);
    let lsp = last_publish_for(&out, &main_uri)["diagnostics"][0].clone();
    fs::write(dir.join("helper.sm"), broken_helper).unwrap();
    let (_, cli) = check_json(&dir, "main.sm");
    let cli = &cli["diagnostics"][0];
    assert_eq!(lsp["code"], cli["code"]);
    assert_eq!(lsp["message"], cli["message"]);
    assert_eq!(lsp["data"]["family"], cli["family"]);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn lsp_rejects_oversized_header_line_before_buffering_it() {
    // Copilot #1: one unterminated header line far beyond the limit. The
    // framing error must arrive after reading at most one maximal line, not
    // after buffering the whole oversized line.
    let prefix = b"Content-Length: 2\r\nX-Pad: ";
    let mut input = prefix.to_vec();
    input.extend(std::iter::repeat_n(b'a', 10 * 1024 * 1024));
    let mut cursor = std::io::Cursor::new(input);
    let error = smc_cli::lsp::read_message(&mut cursor).expect_err("must be rejected");
    assert!(
        error.contains(&format!(
            "header line exceeds {}",
            smc_cli::lsp::MAX_HEADER_LINE_BYTES
        )),
        "{error}"
    );
    let consumed = cursor.position() as usize;
    assert!(
        consumed <= prefix.len() + smc_cli::lsp::MAX_HEADER_LINE_BYTES + 1,
        "read {consumed} bytes before rejecting an oversized header line"
    );
    // The session-level result is the same deterministic framing error.
    let mut input = prefix.to_vec();
    input.extend(std::iter::repeat_n(b'a', 64 * 1024));
    assert!(matches!(run_lsp_raw(input).0, LspExit::Framing(_)));
}

#[test]
fn lsp_rejects_too_many_header_bytes() {
    let mut input = Vec::new();
    for i in 0..200 {
        input.extend(format!("X-Header-{i}: {}\r\n", "v".repeat(60)).into_bytes());
    }
    input.extend(b"Content-Length: 2\r\n\r\n{}");
    match run_lsp_raw(input).0 {
        LspExit::Framing(message) => assert!(
            message.contains(&format!(
                "headers exceed {}",
                smc_cli::lsp::MAX_HEADER_BYTES
            )),
            "{message}"
        ),
        other => panic!("expected framing error, got {other:?}"),
    }
}

#[test]
fn lsp_accepts_normal_headers_within_limits() {
    let body = serde_json::to_vec(&json!({"jsonrpc": "2.0", "id": 1, "method": "initialize",
                                          "params": {"capabilities": {}}}))
    .unwrap();
    let mut input = format!(
        "Content-Length: {}\r\nContent-Type: application/vscode-jsonrpc; charset=utf-8\r\n\r\n",
        body.len()
    )
    .into_bytes();
    input.extend(body);
    for m in shutdown_exit(9) {
        input.extend(frame(&m));
    }
    let (exit, out) = run_lsp_raw(input);
    assert_eq!(exit, LspExit::Clean);
    assert!(response(&out, 1)["result"]["capabilities"].is_object());
}

fn mutation_runner(args: &[&str]) -> (i32, String) {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/ssf09_mutation_campaign.py");
    for python in ["python3", "python"] {
        if let Ok(out) = Command::new(python).arg(&script).args(args).output() {
            return (
                out.status.code().unwrap_or(-1),
                format!(
                    "{}{}",
                    String::from_utf8_lossy(&out.stdout),
                    String::from_utf8_lossy(&out.stderr)
                ),
            );
        }
    }
    panic!("no python interpreter (python3/python) available for the mutation runner test");
}

#[test]
fn mutation_campaign_selection_fails_closed() {
    // Copilot #2: an unknown selector never yields a vacuous success.
    let (code, out) = mutation_runner(&["M13"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("unknown mutant id(s): M13"), "{out}");
    let (code, out) = mutation_runner(&["M1", "M13"]);
    assert_eq!(code, 2, "M1 must not run alone when M13 is unknown: {out}");
    assert!(!out.contains("KILLED"), "{out}");
    let (code, out) = mutation_runner(&["--dry-run", "M13"]);
    assert_eq!(code, 2, "{out}");
    // The full, known set still resolves, with every anchor present once.
    let (code, out) = mutation_runner(&["--dry-run"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("13 mutant(s) selected"), "{out}");
    let (code, out) = mutation_runner(&["--dry-run", "M1"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("1 mutant(s) selected: M1"), "{out}");
}

#[test]
#[cfg(windows)]
fn lsp_windows_uri_roundtrip_and_no_verbatim_prefix_leak() {
    let dir = fixture_dir("project_ok");
    let main = dir.join("src/main.sm");
    let uri = smc_cli::lsp::path_to_uri(&main.canonicalize().unwrap());
    assert!(
        !uri.contains("%3F") && !uri.contains("?"),
        "LSP URI contains verbatim question mark: {uri}"
    );
    assert!(
        uri.starts_with("file:///"),
        "LSP URI does not start with standard file:/// : {uri}"
    );
    let mut msgs = init(Some(&dir));
    msgs.push(open(&uri, 1, &fs::read_to_string(&main).unwrap()));
    msgs.extend(shutdown_exit(9));
    let (_, out) = run_lsp(&msgs);
    let publish = last_publish_for(&out, &uri);
    assert_eq!(publish["diagnostics"], json!([]));
}
