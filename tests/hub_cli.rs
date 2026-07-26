//! End-to-end tests for `smc hub ...` (Issue #1553).
//!
//! Real subprocess invocations of the built `smc` binary
//! (`Command::new(env!("CARGO_BIN_EXE_smc"))`), matching the existing
//! convention (`tests/cli_look_ui_frame.rs`). Every test runs in its own
//! fresh temporary working directory so `.semantic/hub/` state never
//! collides between tests or leaks into the repo working tree.
//!
//! This exercises the full path: CLI argument parsing -> request-file
//! parsing -> Hub admission -> registry lookup -> capability/resource
//! policy -> the real TurboVec worker -> reply validation ->
//! audit/provenance -> CLI output. Nothing here calls
//! `semantic_hub_turbovec::TurboVecAdapter` directly -- only the built
//! binary, so there is no bypass of Hub admission in this test file.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn smc_in(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_smc"))
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run smc")
}

fn fixture_path(name: &str) -> String {
    let path = std::env::current_dir()
        .expect("cwd")
        .join("tests/fixtures/hub")
        .join(name);
    path.to_string_lossy().into_owned()
}

fn temp_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "{prefix}_{}_{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    fs::create_dir_all(&dir).expect("mkdir temp dir");
    dir
}

fn stdout_json(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "expected stdout to be JSON: {e}\nstdout={}\nstderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn invoke(dir: &Path, operation: &str, fixture: &str) -> Output {
    smc_in(
        dir,
        &[
            "hub",
            "invoke",
            "vector.turbovec",
            operation,
            "--input",
            &fixture_path(fixture),
        ],
    )
}

// ---- smc hub tools / describe -----------------------------------------

#[test]
fn tools_lists_vector_turbovec_deterministically() {
    let dir = temp_dir("hub_cli_tools");
    let output = smc_in(&dir, &["hub", "tools"]);
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.starts_with("vector.turbovec\t0.9.0\tInProcess\tRegistered"));
}

#[test]
fn describe_reports_all_seven_operations() {
    let dir = temp_dir("hub_cli_describe");
    let output = smc_in(&dir, &["hub", "describe", "vector.turbovec"]);
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    for op in [
        "vector.index.create",
        "vector.index.describe",
        "vector.index.insert",
        "vector.index.remove",
        "vector.search",
        "vector.search.filtered",
        "vector.index.reset",
    ] {
        assert!(text.contains(op), "describe output missing {op}:\n{text}");
    }
}

#[test]
fn describe_unknown_tool_is_a_distinct_error() {
    let dir = temp_dir("hub_cli_describe_unknown");
    let output = smc_in(&dir, &["hub", "describe", "solver.z3"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("UnknownTool"));
}

// ---- full real workflow: create -> insert -> describe -> search -------
// ---- -> filtered search -> remove -> search again -> audit ------------

#[test]
fn full_workflow_create_insert_search_filter_remove_search_again_audit() {
    let dir = temp_dir("hub_cli_workflow");

    let create = invoke(&dir, "vector.index.create", "valid_index_create.json");
    assert!(create.status.success(), "{:?}", create);
    assert_eq!(stdout_json(&create)["status"], "Success");

    let insert = invoke(&dir, "vector.index.insert", "valid_index_insert.json");
    assert!(insert.status.success());
    let insert_json = stdout_json(&insert);
    assert_eq!(insert_json["status"], "Success");
    assert_eq!(insert_json["payload"]["inserted"], 4);

    let describe = invoke(&dir, "vector.index.describe", "valid_index_describe.json");
    assert!(describe.status.success());
    assert_eq!(stdout_json(&describe)["payload"]["len"], 4);

    let search = invoke(&dir, "vector.search", "valid_search.json");
    assert!(search.status.success());
    let search_json = stdout_json(&search);
    let first_hit_id = search_json["payload"]["hits"][0][0]["external_id"]
        .as_u64()
        .expect("first hit external_id");
    assert_eq!(
        first_hit_id, 102,
        "query [0,1,0,...] should rank id 102 first"
    );

    let filtered = invoke(&dir, "vector.search.filtered", "valid_search_filtered.json");
    assert!(filtered.status.success());
    let filtered_json = stdout_json(&filtered);
    let filtered_ids: Vec<u64> = filtered_json["payload"]["hits"][0]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["external_id"].as_u64().unwrap())
        .collect();
    assert!(
        !filtered_ids.contains(&102),
        "filtered search must exclude id 102: {filtered_ids:?}"
    );
    assert!(
        !filtered_ids.contains(&104),
        "filtered search must exclude id 104: {filtered_ids:?}"
    );

    let request_id = filtered_json["request_id"]
        .as_str()
        .expect("request_id")
        .to_string();

    let remove = invoke(&dir, "vector.index.remove", "valid_index_remove.json");
    assert!(remove.status.success());
    let remove_json = stdout_json(&remove);
    assert_eq!(remove_json["payload"]["removed"], serde_json::json!([102]));
    assert_eq!(remove_json["payload"]["len"], 3);

    let search_again = invoke(&dir, "vector.search", "valid_search.json");
    assert!(search_again.status.success());
    let again_json = stdout_json(&search_again);
    let again_ids: Vec<u64> = again_json["payload"]["hits"][0]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["external_id"].as_u64().unwrap())
        .collect();
    assert!(
        !again_ids.contains(&102),
        "removed id 102 must not reappear in search: {again_ids:?}"
    );

    let audit = smc_in(&dir, &["hub", "audit", "--request", &request_id]);
    assert!(audit.status.success(), "{:?}", audit);
    let audit_text = String::from_utf8_lossy(&audit.stdout);
    assert!(audit_text.contains(&format!("request_id: {request_id}")));
    assert!(audit_text.contains("tool_id: vector.turbovec"));
    assert!(audit_text.contains("status: Success"));

    let reset = invoke(&dir, "vector.index.reset", "valid_index_reset.json");
    assert!(reset.status.success());
    let describe_after_reset = invoke(&dir, "vector.index.describe", "valid_index_describe.json");
    assert_eq!(stdout_json(&describe_after_reset)["payload"]["len"], 0);
}

// ---- adversarial / rejection paths --------------------------------------

#[test]
fn capability_denial_is_rejected_and_hub_remains_usable_afterward() {
    // Regression coverage for a real bug found via manual dogfooding: a
    // capability-denial rejection's audit record once corrupted the
    // persisted audit log for every subsequent invocation.
    let dir = temp_dir("hub_cli_cap_denied");
    invoke(&dir, "vector.index.create", "valid_index_create.json");
    invoke(&dir, "vector.index.insert", "valid_index_insert.json");

    let denied = invoke(&dir, "vector.search", "reject_capability_denied.json");
    assert!(!denied.status.success());
    assert!(String::from_utf8_lossy(&denied.stderr).starts_with("CapabilityDenied"));
    let denied_json = stdout_json(&denied);
    assert_eq!(denied_json["status"], "Rejected");
    assert_eq!(denied_json["fault_code"], "CapabilityDenied");
    // The real tool_version must appear even on a pre-dispatch rejection,
    // not the internal placeholder's 0.0.0 (another real bug fixed here).
    assert_eq!(denied_json["tool_version"], "0.9.0");

    let still_works = invoke(&dir, "vector.search", "valid_search.json");
    assert!(
        still_works.status.success(),
        "hub must remain usable after a capability denial: {:?}",
        still_works
    );
}

#[test]
fn unsupported_schema_version_is_rejected_before_payload_is_interpreted() {
    let dir = temp_dir("hub_cli_schema_version");
    let output = invoke(
        &dir,
        "vector.search",
        "reject_unsupported_schema_version.json",
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("SchemaVersionUnsupported"));
}

#[test]
fn invalid_vector_dimension_is_a_typed_failure_not_a_crash() {
    let dir = temp_dir("hub_cli_bad_dim");
    invoke(&dir, "vector.index.create", "valid_index_create.json");
    let output = invoke(
        &dir,
        "vector.index.insert",
        "reject_invalid_vector_dimension.json",
    );
    assert!(!output.status.success());
    let json = stdout_json(&output);
    assert_eq!(json["status"], "ToolFailed");
    assert_eq!(json["fault_code"], "ToolDeclaredFailure");
}

#[test]
fn malformed_truncated_request_is_rejected_as_input_rejected() {
    let dir = temp_dir("hub_cli_malformed");
    let output = invoke(&dir, "vector.search", "reject_malformed_truncated.json");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("InputRejected"));
}

#[test]
fn audit_lookup_for_unknown_request_id_is_a_distinct_error() {
    let dir = temp_dir("hub_cli_audit_unknown");
    invoke(&dir, "vector.index.create", "valid_index_create.json");
    let output = smc_in(&dir, &["hub", "audit", "--request", "req-does-not-exist"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("UnknownRequest"));
}

// ---- --out atomic write --------------------------------------------------

#[test]
fn out_flag_writes_reply_atomically_and_matches_stdout_semantics() {
    let dir = temp_dir("hub_cli_out_flag");
    invoke(&dir, "vector.index.create", "valid_index_create.json");
    invoke(&dir, "vector.index.insert", "valid_index_insert.json");

    let out_path = dir.join("reply.json");
    let output = smc_in(
        &dir,
        &[
            "hub",
            "invoke",
            "vector.turbovec",
            "vector.search",
            "--input",
            &fixture_path("valid_search.json"),
            "--out",
            out_path.to_str().unwrap(),
        ],
    );
    assert!(output.status.success());
    assert!(
        output.stdout.is_empty(),
        "with --out, stdout should not also carry the reply"
    );
    let written = fs::read_to_string(&out_path).expect("read --out file");
    let written_json: serde_json::Value = serde_json::from_str(&written).expect("valid JSON");
    assert_eq!(written_json["status"], "Success");

    // No leftover temp file from the atomic-write pattern.
    let leftovers: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains(".tmp."))
        .collect();
    assert!(
        leftovers.is_empty(),
        "atomic write left a temp file behind: {leftovers:?}"
    );
}

#[test]
fn same_request_repeated_produces_deterministic_search_ranking() {
    // CLI-level determinism check, complementing the adapter-level
    // qualification in crates/semantic-hub-turbovec/tests/determinism_qualification.rs:
    // asserts the *ranking* (not the full byte-identical reply envelope,
    // since request_id/timing legitimately vary per invocation) is stable
    // across repeated real CLI invocations against the persisted index.
    let dir = temp_dir("hub_cli_determinism");
    invoke(&dir, "vector.index.create", "valid_index_create.json");
    invoke(&dir, "vector.index.insert", "valid_index_insert.json");

    let extract_ranking = |output: &Output| -> Vec<u64> {
        stdout_json(output)["payload"]["hits"][0]
            .as_array()
            .unwrap()
            .iter()
            .map(|h| h["external_id"].as_u64().unwrap())
            .collect()
    };

    let first = invoke(&dir, "vector.search", "valid_search.json");
    let first_ranking = extract_ranking(&first);
    for _ in 0..3 {
        let repeat = invoke(&dir, "vector.search", "valid_search.json");
        assert_eq!(extract_ranking(&repeat), first_ranking);
    }
}
