//! SSF-11 (#1582) canonical application conformance corpus.
//!
//! Replays every case in `examples/qualification/ssf11/corpus.json` through
//! the real public `smc` binary and checks only observable outcomes: exit
//! status, diagnostic codes, verifier/runtime identifiers, stdout bytes, and
//! files written inside a per-case sandbox. Absolute paths, temp directory
//! names, and timing are never compared, so the same corpus can later be
//! replayed against a Bootstrap toolchain.

use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const CORPUS: &str = "examples/qualification/ssf11/corpus.json";
const FAMILIES: [&str; 12] = [
    "F01", "F02", "F03", "F04", "F05", "F06", "F07", "F08", "F09", "F10", "F11", "F12",
];
const DISPOSITIONS: [&str; 6] = [
    "PASS-REUSED",
    "PASS-NEW",
    "EXCLUDED-JUSTIFIED",
    "RETURN-TO-OWNER",
    "BLOCKED",
    "HISTORICAL-NON-QUALIFYING",
];

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn corpus() -> Value {
    let text = std::fs::read_to_string(repo().join(CORPUS)).expect("read corpus");
    serde_json::from_str(&text).expect("corpus is valid JSON")
}

fn cases(corpus: &Value) -> &Vec<Value> {
    corpus["cases"].as_array().expect("cases array")
}

fn str_list(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .map(|item| item.as_str().expect("string item").to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// Per-case scratch area: `outer/` holds the sandbox root `outer/root/`, so a
/// parent-traversal escape would land in `outer/` and can be detected.
struct Scratch {
    outer: PathBuf,
    root: PathBuf,
    artifact: PathBuf,
}

impl Scratch {
    fn new(case_id: &str) -> Self {
        let outer = std::env::temp_dir().join(format!(
            "ssf11_{}_{}_{}",
            case_id.replace('.', "_"),
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = std::fs::remove_dir_all(&outer);
        let root = outer.join("root");
        std::fs::create_dir_all(&root).expect("create sandbox root");
        let artifact = outer.join("artifact.smc");
        Self {
            outer,
            root,
            artifact,
        }
    }

    fn reset_root(&self, files: &Value) {
        let _ = std::fs::remove_dir_all(&self.root);
        std::fs::create_dir_all(&self.root).expect("recreate sandbox root");
        if let Some(files) = files.as_object() {
            for (name, contents) in files {
                std::fs::write(self.root.join(name), contents.as_str().expect("file text"))
                    .expect("write sandbox file");
            }
        }
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.outer);
    }
}

struct Observed {
    exit: i32,
    stdout: String,
    stderr: String,
}

fn normalize(text: &str, scratch: &Scratch) -> String {
    let repo_root = repo().to_string_lossy().replace('\\', "/");
    let outer = scratch.outer.to_string_lossy().replace('\\', "/");
    text.replace("\r\n", "\n")
        .replace('\\', "/")
        .replace(&outer, "<scratch>")
        .replace(&repo_root, "<repo>")
}

fn run_step(case: &Value, step: &Value, scratch: &Scratch) -> Observed {
    let entry = case["entry_path"]
        .as_str()
        .map(|rel| repo().join(rel).to_string_lossy().into_owned());
    let artifact = scratch.artifact.to_string_lossy().into_owned();
    let args: Vec<String> = str_list(&step["args"])
        .into_iter()
        .map(|arg| match arg.as_str() {
            "{entry}" => entry.clone().expect("case has entry_path"),
            "{artifact}" => artifact.clone(),
            _ => arg,
        })
        .collect();
    let cwd = if step["cwd"].as_str() == Some("sandbox") {
        scratch.root.clone()
    } else {
        repo()
    };
    let output = Command::new(env!("CARGO_BIN_EXE_smc"))
        .args(&args)
        .current_dir(&cwd)
        .output()
        .expect("spawn smc");
    Observed {
        exit: output.status.code().unwrap_or(-1),
        stdout: normalize(&String::from_utf8_lossy(&output.stdout), scratch),
        stderr: normalize(&String::from_utf8_lossy(&output.stderr), scratch),
    }
}

fn diagnostic_codes(stdout: &str) -> Vec<String> {
    let json: Value = serde_json::from_str(stdout).expect("--format json emits JSON");
    json["diagnostics"]
        .as_array()
        .expect("diagnostics array")
        .iter()
        .map(|d| d["code"].as_str().expect("diagnostic code").to_string())
        .collect()
}

fn sandbox_listing(root: &Path) -> BTreeSet<String> {
    std::fs::read_dir(root)
        .expect("list sandbox")
        .map(|entry| {
            entry
                .expect("entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

fn check_step(case_id: &str, index: usize, step: &Value, seen: &Observed, scratch: &Scratch) {
    let ctx = format!(
        "{case_id} step {index} {:?}\n--- stdout ---\n{}\n--- stderr ---\n{}",
        str_list(&step["args"]),
        seen.stdout,
        seen.stderr
    );
    let expected_exit = step["expected_exit"].as_i64().expect("expected_exit") as i32;
    assert_eq!(seen.exit, expected_exit, "exit status mismatch: {ctx}");
    if let Some(exact) = step["stdout_exact"].as_str() {
        assert_eq!(seen.stdout, exact, "stdout mismatch: {ctx}");
    }
    if let Some(pattern) = step["stdout_regex"].as_str() {
        assert!(
            simple_sha_shape(pattern, &seen.stdout),
            "stdout shape: {ctx}"
        );
    }
    for needle in str_list(&step["stdout_contains"]) {
        assert!(
            seen.stdout.contains(&needle),
            "stdout lacks {needle:?}: {ctx}"
        );
    }
    for needle in str_list(&step["stderr_contains"]) {
        assert!(
            seen.stderr.contains(&needle),
            "stderr lacks {needle:?}: {ctx}"
        );
    }
    let codes = str_list(&step["diagnostic_codes"]);
    if !codes.is_empty() {
        assert_eq!(diagnostic_codes(&seen.stdout), codes, "codes: {ctx}");
    }
    if step["artifact_absent"].as_bool() == Some(true) {
        assert!(!scratch.artifact.exists(), "artifact must not exist: {ctx}");
    }
    if let Some(files) = step["files_exact"].as_object() {
        for (name, expected) in files {
            let actual = std::fs::read(scratch.root.join(name))
                .unwrap_or_else(|e| panic!("missing sandbox file {name}: {e}: {ctx}"));
            assert_eq!(
                actual,
                expected.as_str().expect("file text").as_bytes(),
                "bytes of {name}: {ctx}"
            );
        }
    }
    let only = str_list(&step["files_only"]);
    if !only.is_empty() {
        let expected: BTreeSet<String> = only.into_iter().collect();
        assert_eq!(sandbox_listing(&scratch.root), expected, "sandbox: {ctx}");
    }
    for name in str_list(&step["outside_absent"]) {
        assert!(
            !scratch.outer.join(&name).exists(),
            "{name} escaped root: {ctx}"
        );
    }
}

/// The only regex the corpus uses is the SSF-10 digest shape; keep the check
/// local instead of adding a regex dependency.
fn simple_sha_shape(pattern: &str, text: &str) -> bool {
    assert_eq!(
        pattern, "^sha256:[0-9a-f]{64}\n$",
        "unsupported stdout_regex"
    );
    let Some(hex) = text
        .strip_prefix("sha256:")
        .and_then(|t| t.strip_suffix('\n'))
    else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

#[test]
fn every_corpus_case_replays_through_the_public_cli() {
    let corpus = corpus();
    let mut executed = 0;
    for case in cases(&corpus) {
        let case_id = case["case_id"].as_str().expect("case_id");
        let scratch = Scratch::new(case_id);
        for (index, step) in case["steps"].as_array().expect("steps").iter().enumerate() {
            scratch.reset_root(&case["sandbox_files"]);
            let seen = run_step(case, step, &scratch);
            check_step(case_id, index, step, &seen, &scratch);
            if step["replay_identical"].as_bool() == Some(true) {
                scratch.reset_root(&case["sandbox_files"]);
                let replay = run_step(case, step, &scratch);
                assert_eq!(
                    (seen.exit, &seen.stdout, &seen.stderr),
                    (replay.exit, &replay.stdout, &replay.stderr),
                    "{case_id} step {index} is not replay-deterministic"
                );
                check_step(case_id, index, step, &replay, &scratch);
            }
            executed += 1;
        }
    }
    assert!(executed > 0, "corpus executed no steps");
}

#[test]
fn corpus_covers_all_twelve_families_with_honest_dispositions() {
    let corpus = corpus();
    assert_eq!(corpus["schema"], "semantic.foundation.ssf11.corpus");
    assert_eq!(corpus["schema_version"], 1);
    let families = corpus["families"].as_array().expect("families");
    let ids: Vec<&str> = families
        .iter()
        .map(|f| f["family_id"].as_str().expect("family_id"))
        .collect();
    assert_eq!(ids, FAMILIES, "families must be exactly F01..F12 in order");
    for family in families {
        let id = family["family_id"].as_str().unwrap();
        let disposition = family["disposition"].as_str().expect("disposition");
        assert!(DISPOSITIONS.contains(&disposition), "{id}: {disposition}");
        let family_cases: Vec<&Value> = cases(&corpus)
            .iter()
            .filter(|c| c["family_id"] == id)
            .collect();
        assert!(!family_cases.is_empty(), "{id} has no corpus case");
        let required_kind = match family["evidence_kind"].as_str() {
            Some("rejection") => "negative",
            Some("execution") => "positive",
            Some("historical") => "historical",
            other => panic!("{id}: unknown evidence_kind {other:?}"),
        };
        if disposition.starts_with("PASS") {
            assert!(
                family_cases.iter().any(|c| c["kind"] == required_kind),
                "{id} claims PASS without a {required_kind} case"
            );
        }
        if disposition == "EXCLUDED-JUSTIFIED" {
            assert!(
                family_cases.iter().any(|c| c["kind"] == "negative"),
                "{id}: an exclusion needs deterministic boundary evidence"
            );
        }
        if disposition == "HISTORICAL-NON-QUALIFYING" {
            assert!(
                family_cases.iter().all(|c| c["kind"] == "historical"
                    && c["bootstrap_comparable"] == false
                    && c["qualification_role"] == "historical-non-qualifying"),
                "{id}: historical evidence must not be presented as qualifying"
            );
        }
    }
}

#[test]
fn corpus_cases_are_unique_and_reference_existing_paths() {
    let corpus = corpus();
    let mut seen = BTreeSet::new();
    for case in cases(&corpus) {
        let case_id = case["case_id"].as_str().expect("case_id");
        assert!(
            seen.insert(case_id.to_string()),
            "duplicate case_id {case_id}"
        );
        assert!(case_id.starts_with("ssf11.f"), "{case_id}: id prefix");
        for field in ["kind", "profile", "maturity", "qualification_role"] {
            assert!(case[field].is_string(), "{case_id}: missing {field}");
        }
        assert!(
            ["positive", "negative", "historical"].contains(&case["kind"].as_str().unwrap()),
            "{case_id}: unknown kind"
        );
        if let Some(entry) = case["entry_path"].as_str() {
            assert!(repo().join(entry).exists(), "{case_id}: missing {entry}");
        }
        for reference in str_list(&case["contract_refs"]) {
            assert!(
                repo().join(&reference).exists(),
                "{case_id}: missing ref {reference}"
            );
        }
        if case["kind"] == "positive" {
            assert!(
                case["maturity"] != "Roadmap" && case["maturity"] != "Experimental",
                "{case_id}: a Roadmap/Experimental surface cannot be a positive case"
            );
        }
    }
}

/// Every Bootstrap-comparable case must carry enough metadata to be replayed
/// against another implementation: stable id, entry, commands, expected exit,
/// and an observable expectation that is not a Rust internal.
#[test]
fn bootstrap_comparable_cases_are_replayable() {
    let corpus = corpus();
    for case in cases(&corpus) {
        if case["bootstrap_comparable"] != true {
            continue;
        }
        let case_id = case["case_id"].as_str().unwrap();
        assert!(case["entry_path"].is_string(), "{case_id}: entry_path");
        let observable = case["expected_observable"].as_str().unwrap_or_default();
        assert!(
            !observable.trim().is_empty(),
            "{case_id}: expected_observable"
        );
        let steps = case["steps"].as_array().expect("steps");
        assert!(!steps.is_empty(), "{case_id}: no steps");
        for step in steps {
            assert!(step["expected_exit"].is_i64(), "{case_id}: expected_exit");
            let args = str_list(&step["args"]);
            assert!(!args.is_empty(), "{case_id}: empty command");
            for arg in &args {
                assert!(
                    !arg.contains('/') || arg.starts_with("../") || arg.starts_with("{"),
                    "{case_id}: absolute or repository-specific argument {arg}"
                );
            }
        }
        if case["kind"] == "negative" {
            assert!(
                steps.iter().any(|s| s["expected_exit"] != 0
                    && (s["stderr_contains"].is_array() || s["diagnostic_codes"].is_array())),
                "{case_id}: a negative case needs a stable identifier for its failure"
            );
        }
        let text = case.to_string();
        for forbidden in [
            "sm_vm::",
            "sm_verify::",
            "RuntimeQuotas {",
            "/tmp/",
            "C:\\\\",
        ] {
            assert!(!text.contains(forbidden), "{case_id}: encodes {forbidden}");
        }
    }
}
