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

/// Launches without waiting, so callers can start several real `smc`
/// subprocesses before any of them completes -- required to actually
/// exercise the cross-process project lock, unlike `smc_in`'s `.output()`
/// which blocks until that one process exits.
fn smc_spawn_in(dir: &Path, args: &[String]) -> std::process::Child {
    Command::new(env!("CARGO_BIN_EXE_smc"))
        .args(args)
        .current_dir(dir)
        .spawn()
        .expect("spawn smc")
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
fn describe_reports_all_eight_operations() {
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
        "vector.index.recover",
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

#[test]
fn audit_lookup_omits_sensitive_correlation_and_identity_metadata() {
    let dir = temp_dir("hub_cli_audit_confidentiality");
    let request_id = "confidential-request-id";
    let session_id = "confidential-session-id";
    let caller_identity = "confidential-caller-identity";
    let request = serde_json::json!({
        "request_id": request_id,
        "session_id": session_id,
        "caller_identity": caller_identity,
        "capabilities": ["VectorIndexCreate", "PrivateStorageRead", "PrivateStorageWrite"],
        "payload": {"index": "confidentiality-docs", "dim": 8, "bit_width": 4}
    });
    let input = dir.join("confidentiality_request.json");
    fs::write(&input, serde_json::to_vec(&request).unwrap()).unwrap();

    let invoke = smc_in(
        &dir,
        &[
            "hub",
            "invoke",
            "vector.turbovec",
            "vector.index.create",
            "--input",
            input.to_str().unwrap(),
        ],
    );
    assert!(invoke.status.success(), "{:?}", invoke);

    let audit = smc_in(&dir, &["hub", "audit", "--request", request_id]);
    assert!(audit.status.success(), "{:?}", audit);
    let audit_text = String::from_utf8_lossy(&audit.stdout);
    assert!(
        audit_text.contains(&format!("request_id: {request_id}")),
        "audit stdout must retain its public lookup handle: {audit_text}"
    );
    for sensitive_value in [session_id, caller_identity] {
        assert!(
            !audit_text.contains(sensitive_value),
            "audit stdout leaked {sensitive_value:?}: {audit_text}"
        );
    }
    assert!(
        audit_text.contains("adapter_provenance: <redacted>"),
        "{audit_text}"
    );
    assert!(
        audit_text.contains("input_digest: <redacted>"),
        "{audit_text}"
    );
    assert!(
        audit_text.contains("output_digest: <redacted>"),
        "{audit_text}"
    );
    assert!(!audit_text.contains("fnv1a64:"), "{audit_text}");
    assert!(
        audit_text.contains("tool_id: vector.turbovec"),
        "{audit_text}"
    );
    assert!(audit_text.contains("status: Success"), "{audit_text}");

    let persisted_audit = fs::read_to_string(dir.join(".semantic/hub/audit.log")).unwrap();
    for persisted_value in [request_id, session_id, caller_identity] {
        assert!(
            persisted_audit.contains(persisted_value),
            "audit persistence unexpectedly lost {persisted_value:?}"
        );
    }
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
fn out_flag_cannot_overwrite_hub_owned_state() {
    let dir = temp_dir("hub_cli_out_state_guard");
    let internal_out = dir.join(".semantic/hub/audit.log");
    let invoke_output = smc_in(
        &dir,
        &[
            "hub",
            "invoke",
            "vector.turbovec",
            "vector.index.create",
            "--input",
            &fixture_path("valid_index_create.json"),
            "--out",
            internal_out.to_str().unwrap(),
        ],
    );
    assert!(!invoke_output.status.success(), "{:?}", invoke_output);
    assert!(
        String::from_utf8_lossy(&invoke_output.stderr).starts_with("ScopedStorageViolation"),
        "{:?}",
        invoke_output
    );
    assert!(!dir
        .join(".semantic/hub/vector.turbovec/fixture-docs.tvim")
        .exists());

    let session_output = session(
        &dir,
        "session_workflow.ndjson",
        &["--out", internal_out.to_str().unwrap()],
    );
    assert!(!session_output.status.success(), "{:?}", session_output);
    assert!(
        String::from_utf8_lossy(&session_output.stderr).starts_with("ScopedStorageViolation"),
        "{:?}",
        session_output
    );
    assert!(!dir
        .join(".semantic/hub/vector.turbovec/session-docs.tvim")
        .exists());
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

// ---- Codex review remediation regression tests --------------------------

#[test]
fn create_rejects_dimension_exceeding_the_admitted_budget() {
    let dir = temp_dir("hub_cli_dim_budget");
    let req = serde_json::json!({
        "capabilities": ["VectorIndexCreate", "PrivateStorageRead", "PrivateStorageWrite"],
        "resource_budget": {"vector_dimensions": 8},
        "payload": {"index": "docs", "dim": 4096, "bit_width": 4}
    });
    let input = dir.join("create_over_budget.json");
    fs::write(&input, serde_json::to_vec(&req).unwrap()).unwrap();
    let output = smc_in(
        &dir,
        &[
            "hub",
            "invoke",
            "vector.turbovec",
            "vector.index.create",
            "--input",
            input.to_str().unwrap(),
        ],
    );
    assert!(!output.status.success());
    let json = stdout_json(&output);
    assert_eq!(json["status"], "ToolFailed");
    assert_eq!(json["payload"], serde_json::Value::Null);
    let fault_message = json["fault_message"].as_str().unwrap();
    assert!(
        fault_message.contains("DimensionExceedsBudget"),
        "{fault_message}"
    );
}

#[test]
fn payload_exceeding_the_requests_own_declared_input_budget_is_rejected() {
    // Regression test: a caller narrowing resource_budget.input_bytes below
    // the global ceiling previously got no enforcement at all.
    let dir = temp_dir("hub_cli_input_budget");
    invoke(&dir, "vector.index.create", "valid_index_create.json");
    let req = serde_json::json!({
        "capabilities": ["VectorSearch", "PrivateStorageRead"],
        "resource_budget": {"input_bytes": 4},
        "payload": {"index": "fixture-docs", "queries": [[0,1,0,0,0,0,0,0]], "k": 3}
    });
    let input = dir.join("search_over_input_budget.json");
    fs::write(&input, serde_json::to_vec(&req).unwrap()).unwrap();
    let output = smc_in(
        &dir,
        &[
            "hub",
            "invoke",
            "vector.turbovec",
            "vector.search",
            "--input",
            input.to_str().unwrap(),
        ],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("InputRejected"));
}

#[test]
fn granting_a_sensitive_capability_is_rejected_and_survives_audit_round_trip() {
    // Regression test, updated for the v0-completion hardening pass: a
    // request whose capability_context grants ANY sensitive capability
    // (e.g. NetworkAccess) is now rejected outright at admission with
    // SensitiveCapabilityDenied -- see semantic_hub::admission::admit --
    // rather than silently succeeding with the grant just stripped before
    // the adapter saw it. The audit writer must still durably record that
    // rejection (including the sensitive capability that was asked for,
    // as evidence) and remain readable for the *next* invocation
    // afterward -- this is the original regression this test guarded:
    // the audit writer previously could not parse back a sensitive
    // capability name at all, corrupting the whole audit log for every
    // subsequent invocation.
    let dir = temp_dir("hub_cli_sensitive_cap_audit");
    let req = serde_json::json!({
        "capabilities": ["VectorIndexCreate", "PrivateStorageRead", "PrivateStorageWrite", "NetworkAccess"],
        "payload": {"index": "sensitive-cap-docs", "dim": 8, "bit_width": 4}
    });
    let input = dir.join("create_with_sensitive_cap.json");
    fs::write(&input, serde_json::to_vec(&req).unwrap()).unwrap();
    let first = smc_in(
        &dir,
        &[
            "hub",
            "invoke",
            "vector.turbovec",
            "vector.index.create",
            "--input",
            input.to_str().unwrap(),
        ],
    );
    assert!(
        !first.status.success(),
        "a request granting a sensitive capability must be rejected: {:?}",
        first
    );
    assert!(String::from_utf8_lossy(&first.stderr).starts_with("SensitiveCapabilityDenied"));

    // No index was ever created (admission rejected before dispatch) --
    // confirm this by creating the SAME index name for real, without the
    // sensitive grant, which would fail with IndexAlreadyExists if the
    // rejected request had somehow still created it.
    let clean_req = serde_json::json!({
        "capabilities": ["VectorIndexCreate", "PrivateStorageRead", "PrivateStorageWrite"],
        "payload": {"index": "sensitive-cap-docs", "dim": 8, "bit_width": 4}
    });
    let clean_input = dir.join("create_without_sensitive_cap.json");
    fs::write(&clean_input, serde_json::to_vec(&clean_req).unwrap()).unwrap();
    let second = smc_in(
        &dir,
        &[
            "hub",
            "invoke",
            "vector.turbovec",
            "vector.index.create",
            "--input",
            clean_input.to_str().unwrap(),
        ],
    );
    // The audit log must still be readable after a record containing a
    // sensitive capability grant -- this is exactly the failure mode the
    // earlier status_code/fault_code bug produced.
    assert!(
        second.status.success(),
        "audit log must remain readable after a rejected request granting a sensitive capability: {:?}",
        second
    );
}

#[test]
fn audit_write_failure_leaves_a_pending_marker_and_the_mutation_still_applies() {
    // Regression test for the "commit-before-audit-persists" gap: force the
    // durable audit write to fail (by replacing audit.log with a directory
    // right before a mutating call) and confirm (a) the underlying TurboVec
    // mutation still applies, (b) the invocation itself is reported as a
    // failure (never a false "success"), and (c) `smc hub audit` surfaces a
    // distinct PendingUnresolved status for that request_id instead of a
    // bare UnknownRequest, because a pending marker was written before
    // dispatch and never cleared.
    let dir = temp_dir("hub_cli_pending_marker");
    invoke(&dir, "vector.index.create", "valid_index_create.json");
    invoke(&dir, "vector.index.insert", "valid_index_insert.json");

    let audit_log = dir.join(".semantic").join("hub").join("audit.log");
    assert!(
        audit_log.is_file(),
        "audit.log must exist after two successful invocations"
    );
    fs::remove_file(&audit_log).unwrap();
    fs::create_dir_all(&audit_log).unwrap(); // audit.log is now a directory

    let request_id = "req-pending-marker-test";
    let remove_req = serde_json::json!({
        "request_id": request_id,
        "capabilities": ["VectorIndexMutate", "PrivateStorageRead", "PrivateStorageWrite"],
        "payload": {"index": "fixture-docs", "ids": [102]}
    });
    let input = dir.join("remove_with_broken_audit.json");
    fs::write(&input, serde_json::to_vec(&remove_req).unwrap()).unwrap();
    let broken = smc_in(
        &dir,
        &[
            "hub",
            "invoke",
            "vector.turbovec",
            "vector.index.remove",
            "--input",
            input.to_str().unwrap(),
        ],
    );
    assert!(
        !broken.status.success(),
        "an invocation must never report success when its audit write failed"
    );
    assert!(String::from_utf8_lossy(&broken.stderr).starts_with("AuditProvenanceFailure"));

    // The marker's filename is a digest of request_id, not the raw text
    // (see pending_marker_path's own doc comment) -- check that the
    // pending directory has a real file in it rather than trying to
    // replicate the digest here.
    let pending_dir = dir.join(".semantic").join("hub").join("pending");
    assert!(
        fs::read_dir(&pending_dir)
            .map(|mut entries| entries.next().is_some())
            .unwrap_or(false),
        "a pending marker must remain on disk when the audit write fails"
    );

    // Restore audit.log to a normal (missing -> freshly empty) file so
    // subsequent commands can run, then confirm the removal itself DID
    // apply even though it was never durably audited.
    fs::remove_dir_all(&audit_log).unwrap();
    let search = invoke(&dir, "vector.search", "valid_search.json");
    assert!(search.status.success());
    let ids: Vec<u64> = stdout_json(&search)["payload"]["hits"][0]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["external_id"].as_u64().unwrap())
        .collect();
    assert!(
        !ids.contains(&102),
        "the remove mutation must have applied even though its audit write failed: {ids:?}"
    );

    let lookup = smc_in(&dir, &["hub", "audit", "--request", request_id]);
    assert!(!lookup.status.success());
    assert!(
        String::from_utf8_lossy(&lookup.stderr).starts_with("PendingUnresolved"),
        "{:?}",
        lookup
    );
}

#[test]
fn a_reused_request_id_is_rejected_and_does_not_append_a_second_audit_record() {
    // Regression test: request_id was accepted without checking whether it
    // already existed in the persisted audit trail. `find_by_request` only
    // ever returns the *first* matching record, so a second invocation
    // reusing the same id would append a record that no lookup could ever
    // surface.
    let dir = temp_dir("hub_cli_dup_request_id");
    let request_id = "req-reused-once";
    let req = serde_json::json!({
        "request_id": request_id,
        "capabilities": ["VectorIndexCreate", "PrivateStorageRead", "PrivateStorageWrite"],
        "payload": {"index": "dup-id-docs", "dim": 8, "bit_width": 4}
    });
    let input = dir.join("create_with_fixed_id.json");
    fs::write(&input, serde_json::to_vec(&req).unwrap()).unwrap();

    let first = smc_in(
        &dir,
        &[
            "hub",
            "invoke",
            "vector.turbovec",
            "vector.index.create",
            "--input",
            input.to_str().unwrap(),
        ],
    );
    assert!(first.status.success(), "{:?}", first);

    let second = smc_in(
        &dir,
        &[
            "hub",
            "invoke",
            "vector.turbovec",
            "vector.index.create",
            "--input",
            input.to_str().unwrap(),
        ],
    );
    assert!(!second.status.success());
    assert!(String::from_utf8_lossy(&second.stderr).starts_with("DuplicateRequestId"));

    let audit_log = dir.join(".semantic").join("hub").join("audit.log");
    let contents = fs::read_to_string(&audit_log).expect("read audit.log");
    let record_lines: Vec<&str> = contents
        .lines()
        .filter(|l| l.contains(request_id))
        .collect();
    assert_eq!(
        record_lines.len(),
        1,
        "a rejected duplicate must not append a second audit record: {record_lines:?}"
    );
}

#[test]
fn reusing_a_request_id_with_an_unresolved_pending_marker_is_rejected() {
    // Continuation of `audit_write_failure_leaves_a_pending_marker_and_the_
    // mutation_still_applies`: once a request_id's audit write has failed
    // and left a pending marker, retrying that SAME id must be rejected
    // outright, not silently proceed and clear the marker -- clearing it
    // would erase the only durable evidence that the earlier, still-
    // unresolved invocation may already have applied its effect.
    let dir = temp_dir("hub_cli_dup_pending");
    invoke(&dir, "vector.index.create", "valid_index_create.json");
    invoke(&dir, "vector.index.insert", "valid_index_insert.json");

    let audit_log = dir.join(".semantic").join("hub").join("audit.log");
    fs::remove_file(&audit_log).unwrap();
    fs::create_dir_all(&audit_log).unwrap(); // audit.log is now a directory: writes will fail

    let request_id = "req-pending-then-reused";
    let remove_req = serde_json::json!({
        "request_id": request_id,
        "capabilities": ["VectorIndexMutate", "PrivateStorageRead", "PrivateStorageWrite"],
        "payload": {"index": "fixture-docs", "ids": [102]}
    });
    let input = dir.join("remove_broken.json");
    fs::write(&input, serde_json::to_vec(&remove_req).unwrap()).unwrap();

    let broken = smc_in(
        &dir,
        &[
            "hub",
            "invoke",
            "vector.turbovec",
            "vector.index.remove",
            "--input",
            input.to_str().unwrap(),
        ],
    );
    assert!(!broken.status.success());
    assert!(String::from_utf8_lossy(&broken.stderr).starts_with("AuditProvenanceFailure"));

    fs::remove_dir_all(&audit_log).unwrap(); // restore audit.log so the retry can run at all

    let retry = smc_in(
        &dir,
        &[
            "hub",
            "invoke",
            "vector.turbovec",
            "vector.index.remove",
            "--input",
            input.to_str().unwrap(),
        ],
    );
    assert!(!retry.status.success());
    assert!(String::from_utf8_lossy(&retry.stderr).starts_with("DuplicateRequestId"));

    // The marker's filename is a digest of request_id, not the raw text
    // -- check the pending directory still has a real file in it.
    let pending_dir = dir.join(".semantic").join("hub").join("pending");
    assert!(
        fs::read_dir(&pending_dir)
            .map(|mut entries| entries.next().is_some())
            .unwrap_or(false),
        "a rejected retry must not have cleared the still-unresolved pending marker"
    );
}

// ---- concurrent-process regression tests (project-level lock) ---------

#[test]
fn concurrent_invocations_against_the_same_project_do_not_lose_audit_records() {
    // Regression test: without a cross-process lock, two "smc hub invoke"
    // processes racing against the same project directory could each load
    // the same on-disk audit.log, mutate their own in-memory copy, and
    // atomically rename their own replacement over the other's -- the
    // last rename wins and an earlier process's already-Success-reported
    // record silently vanishes. Real subprocesses, actually launched
    // concurrently (spawn, not output), not simulated.
    let dir = temp_dir("hub_cli_concurrent_audit");
    invoke(&dir, "vector.index.create", "valid_index_create.json");
    invoke(&dir, "vector.index.insert", "valid_index_insert.json");

    const N: usize = 8;
    let children: Vec<_> = (0..N)
        .map(|i| {
            let request_id = format!("req-concurrent-audit-{i}");
            let req = serde_json::json!({
                "request_id": request_id,
                "capabilities": ["VectorSearch", "PrivateStorageRead"],
                "payload": {"index": "fixture-docs", "queries": [[0,1,0,0,0,0,0,0]], "k": 3}
            });
            let input = dir.join(format!("concurrent_search_{i}.json"));
            fs::write(&input, serde_json::to_vec(&req).unwrap()).unwrap();
            let args = vec![
                "hub".to_string(),
                "invoke".to_string(),
                "vector.turbovec".to_string(),
                "vector.search".to_string(),
                "--input".to_string(),
                input.to_str().unwrap().to_string(),
            ];
            (request_id, smc_spawn_in(&dir, &args))
        })
        .collect();

    for (request_id, child) in children {
        let output = child.wait_with_output().expect("wait for smc");
        assert!(
            output.status.success(),
            "invocation for {request_id} failed: {output:?}"
        );
    }

    let audit_log = fs::read_to_string(dir.join(".semantic").join("hub").join("audit.log"))
        .expect("read audit.log");
    for i in 0..N {
        let request_id = format!("req-concurrent-audit-{i}");
        assert!(
            audit_log.contains(&request_id),
            "audit.log is missing the record for {request_id} -- a concurrent invocation's \
             record was lost:\n{audit_log}"
        );
    }
}

#[test]
fn concurrent_inserts_against_the_same_index_do_not_lose_updates() {
    // Regression test: the identical race one layer down -- concurrent
    // inserts into the same .tvim index could each load the same old
    // snapshot and rename their own replacement over the other's,
    // silently losing an insert even though every invocation reports
    // Success.
    let dir = temp_dir("hub_cli_concurrent_insert");
    invoke(&dir, "vector.index.create", "valid_index_create.json");

    const N: usize = 6;
    let children: Vec<_> = (0..N)
        .map(|i| {
            let mut vector = vec![0.0f64; 8];
            vector[i % 8] = 1.0;
            let req = serde_json::json!({
                "request_id": format!("req-concurrent-insert-{i}"),
                "capabilities": ["VectorIndexMutate", "PrivateStorageRead", "PrivateStorageWrite"],
                "payload": {
                    "index": "fixture-docs",
                    "vectors": [vector],
                    "ids": [500 + i],
                }
            });
            let input = dir.join(format!("concurrent_insert_{i}.json"));
            fs::write(&input, serde_json::to_vec(&req).unwrap()).unwrap();
            let args = vec![
                "hub".to_string(),
                "invoke".to_string(),
                "vector.turbovec".to_string(),
                "vector.index.insert".to_string(),
                "--input".to_string(),
                input.to_str().unwrap().to_string(),
            ];
            smc_spawn_in(&dir, &args)
        })
        .collect();

    for child in children {
        let output = child.wait_with_output().expect("wait for smc");
        assert!(
            output.status.success(),
            "concurrent insert failed: {output:?}"
        );
    }

    let describe = invoke(&dir, "vector.index.describe", "valid_index_describe.json");
    assert_eq!(
        stdout_json(&describe)["payload"]["len"],
        N,
        "not all N concurrent inserts landed in the final index -- at least one was lost"
    );
}

// ---- CLI-level symlink confinement (not the TurboVec adapter's own) ----

#[cfg(unix)]
#[test]
fn invoke_rejects_a_project_whose_semantic_directory_is_a_pre_planted_symlink() {
    // Regression test: the TurboVec adapter's own scoped-storage symlink
    // check (crates/semantic-hub-turbovec/src/storage.rs) only protects
    // its own .tvim files, reached from inside Hub::invoke. This CLI
    // module's own direct writes (the project lock, the pending marker,
    // the audit log) never route through that check at all -- a project
    // shipping ".semantic" itself as a pre-planted symlink must still be
    // rejected before any of those CLI-level writes follow it outside the
    // project directory.
    let dir = temp_dir("hub_cli_semantic_symlink");
    let real_target = temp_dir("hub_cli_semantic_symlink_target");

    std::os::unix::fs::symlink(&real_target, dir.join(".semantic")).unwrap();

    let output = invoke(&dir, "vector.index.create", "valid_index_create.json");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).starts_with("ScopedStorageViolation"),
        "{:?}",
        output
    );

    // Nothing must have been written through the symlink into the real
    // target directory.
    let leaked: Vec<_> = fs::read_dir(&real_target)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(
        leaked.is_empty(),
        "a rejected invocation must not have written anything through the symlink: {leaked:?}"
    );
}

#[test]
fn reset_without_private_storage_read_is_denied() {
    // Regression test: handle_reset loads the existing index (to read its
    // dim/bit_width before writing a fresh empty one) but the operation's
    // declared required_capabilities previously omitted
    // PrivateStorageRead, letting a caller reset an index while granting
    // only VectorIndexMutate + PrivateStorageWrite -- admission never
    // checked for the read capability the operation actually performs.
    let dir = temp_dir("hub_cli_reset_needs_read");
    invoke(&dir, "vector.index.create", "valid_index_create.json");

    let req = serde_json::json!({
        "capabilities": ["VectorIndexMutate", "PrivateStorageWrite"],
        "payload": {"index": "fixture-docs"}
    });
    let input = dir.join("reset_missing_read.json");
    fs::write(&input, serde_json::to_vec(&req).unwrap()).unwrap();
    let output = smc_in(
        &dir,
        &[
            "hub",
            "invoke",
            "vector.turbovec",
            "vector.index.reset",
            "--input",
            input.to_str().unwrap(),
        ],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("CapabilityDenied"));
}

#[cfg(unix)]
#[test]
fn invoke_rejects_a_pre_planted_symlinked_audit_log_leaf() {
    // Regression test: check_dir_is_not_a_symlink only ever walks a
    // directory's own components -- it never sees the final audit.log
    // file appended after it. A perfectly ordinary ".semantic/hub/"
    // directory containing a pre-planted symlink AT audit.log's own path
    // must still be rejected, since fs::read/path.is_file() dereference
    // a symlink by default.
    let dir = temp_dir("hub_cli_audit_log_leaf_symlink");
    invoke(&dir, "vector.index.create", "valid_index_create.json");

    let audit_log = dir.join(".semantic").join("hub").join("audit.log");
    fs::remove_file(&audit_log).unwrap();
    let outside_target = temp_dir("hub_cli_audit_log_leaf_symlink_target");
    let forged_log = outside_target.join("forged.log");
    fs::write(&forged_log, b"not a real audit log").unwrap();
    std::os::unix::fs::symlink(&forged_log, &audit_log).unwrap();

    let output = invoke(&dir, "vector.index.describe", "valid_index_describe.json");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).starts_with("ScopedStorageViolation"),
        "{:?}",
        output
    );
}

#[cfg(unix)]
#[test]
fn audit_lookup_rejects_a_symlinked_pending_marker_leaf() {
    // Regression test: cmd_hub_audit's own pending-marker existence
    // check used a bare `.is_file()` with none of the directory/leaf
    // symlink checks applied to the lock, pending-marker write, and
    // audit log elsewhere in this module. A pre-planted symlink at a
    // pending marker's own path must be rejected as a
    // ScopedStorageViolation, not silently followed and reported as a
    // misleading PendingUnresolved.
    //
    // The marker's filename is a digest of the request_id, not the raw
    // text (see pending_marker_path's own doc comment), so this test
    // lets a real broken-audit invocation create the marker first (same
    // trick as audit_write_failure_leaves_a_pending_marker_and_the_
    // mutation_still_applies) and discovers its real path by reading
    // the pending directory, rather than trying to replicate the digest.
    let dir = temp_dir("hub_cli_audit_pending_leaf_symlink");
    invoke(&dir, "vector.index.create", "valid_index_create.json");
    invoke(&dir, "vector.index.insert", "valid_index_insert.json");

    let audit_log = dir.join(".semantic").join("hub").join("audit.log");
    fs::remove_file(&audit_log).unwrap();
    fs::create_dir_all(&audit_log).unwrap(); // audit.log is now a directory: writes will fail

    let request_id = "req-pending-leaf-symlink";
    let remove_req = serde_json::json!({
        "request_id": request_id,
        "capabilities": ["VectorIndexMutate", "PrivateStorageRead", "PrivateStorageWrite"],
        "payload": {"index": "fixture-docs", "ids": [102]}
    });
    let input = dir.join("remove_broken.json");
    fs::write(&input, serde_json::to_vec(&remove_req).unwrap()).unwrap();
    let broken = smc_in(
        &dir,
        &[
            "hub",
            "invoke",
            "vector.turbovec",
            "vector.index.remove",
            "--input",
            input.to_str().unwrap(),
        ],
    );
    assert!(!broken.status.success());
    assert!(String::from_utf8_lossy(&broken.stderr).starts_with("AuditProvenanceFailure"));

    fs::remove_dir_all(&audit_log).unwrap(); // restore audit.log so the audit lookup can run at all

    let pending_dir = dir.join(".semantic").join("hub").join("pending");
    let marker_path = fs::read_dir(&pending_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .next()
        .expect("a real pending marker must exist after the broken-audit invocation");

    let outside_target = temp_dir("hub_cli_audit_pending_leaf_symlink_target");
    let forged_marker = outside_target.join("forged.json");
    fs::write(&forged_marker, b"{}").unwrap();
    fs::remove_file(&marker_path).unwrap();
    std::os::unix::fs::symlink(&forged_marker, &marker_path).unwrap();

    let output = smc_in(&dir, &["hub", "audit", "--request", request_id]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).starts_with("ScopedStorageViolation"),
        "{:?}",
        output
    );
}

#[test]
fn invoke_succeeds_with_a_colon_containing_request_id() {
    // Regression test: HubRequestId's own validation allows ':' (a
    // legal handle character), but ':' is reserved in Windows
    // filenames. Before pending_marker_path derived its filename from a
    // digest instead of the raw request_id text, a request_id
    // containing ':' would make write_pending_marker's file creation
    // fail on Windows even though the request_id itself was perfectly
    // valid -- directly reproducible on this platform, not a
    // hypothetical.
    let dir = temp_dir("hub_cli_colon_request_id");
    let request_id = "req:with:colons";
    let req = serde_json::json!({
        "request_id": request_id,
        "capabilities": ["VectorIndexCreate", "PrivateStorageRead", "PrivateStorageWrite"],
        "payload": {"index": "fixture-docs", "dim": 8, "bit_width": 4}
    });
    let input = dir.join("create_colon_id.json");
    fs::write(&input, serde_json::to_vec(&req).unwrap()).unwrap();
    let output = smc_in(
        &dir,
        &[
            "hub",
            "invoke",
            "vector.turbovec",
            "vector.index.create",
            "--input",
            input.to_str().unwrap(),
        ],
    );
    assert!(output.status.success(), "{:?}", output);
    assert_eq!(stdout_json(&output)["request_id"], request_id);
}

#[test]
fn invoke_rejects_a_wall_time_budget_of_u64_max_instead_of_crashing() {
    // Regression test: on some platforms (observed on Windows), adding
    // an extreme Duration to an Instant panics rather than saturating.
    // A caller-supplied resource_budget.wall_time_millis of u64::MAX
    // previously reached that raw addition inside Hub::invoke before
    // admission's own ceiling check ever ran, crashing the whole smc
    // process instead of producing a typed ResourceBudgetInvalid
    // rejection -- directly reproducible on this dev machine, not a
    // hypothetical.
    let dir = temp_dir("hub_cli_wall_time_u64_max");
    let req = serde_json::json!({
        "capabilities": ["VectorIndexCreate", "PrivateStorageRead", "PrivateStorageWrite"],
        "resource_budget": {"wall_time_millis": u64::MAX},
        "payload": {"index": "fixture-docs", "dim": 8, "bit_width": 4}
    });
    let input = dir.join("create_wall_time_max.json");
    fs::write(&input, serde_json::to_vec(&req).unwrap()).unwrap();
    let output = smc_in(
        &dir,
        &[
            "hub",
            "invoke",
            "vector.turbovec",
            "vector.index.create",
            "--input",
            input.to_str().unwrap(),
        ],
    );
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).starts_with("ResourceBudgetInvalid"),
        "{:?}",
        output
    );
}

// ---- smc hub session (Semantic Hub v0 completion pass) ----------------

fn stdout_ndjson_lines(output: &Output) -> Vec<serde_json::Value> {
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            serde_json::from_str(l).unwrap_or_else(|e| {
                panic!(
                    "expected NDJSON line, got parse error {e}: {l}\nfull stdout={:?}",
                    output
                )
            })
        })
        .collect()
}

fn session(dir: &Path, requests_fixture: &str, extra_args: &[&str]) -> Output {
    let mut args = vec![
        "hub".to_string(),
        "session".to_string(),
        "--requests".to_string(),
        fixture_path(requests_fixture),
    ];
    args.extend(extra_args.iter().map(|s| s.to_string()));
    smc_in(dir, &args.iter().map(String::as_str).collect::<Vec<_>>())
}

#[test]
fn session_processes_a_full_create_insert_search_remove_search_recover_workflow_in_one_batch() {
    let dir = temp_dir("hub_cli_session_workflow");
    let output = session(&dir, "session_workflow.ndjson", &[]);
    assert!(output.status.success(), "{:?}", output);

    let lines = stdout_ndjson_lines(&output);
    assert_eq!(lines.len(), 7, "6 replies + 1 session_summary line");

    let (replies, summary_lines): (Vec<_>, Vec<_>) = lines
        .into_iter()
        .partition(|l| l.get("session_summary").is_none());
    assert_eq!(summary_lines.len(), 1);

    // Every reply succeeded, in submission order, with strictly increasing
    // logical_sequence.
    let mut prev_seq: Option<u64> = None;
    for reply in &replies {
        assert_eq!(reply["status"], "Success", "{:?}", reply);
        let seq = reply["logical_sequence"].as_u64().unwrap();
        if let Some(p) = prev_seq {
            assert!(seq > p, "logical_sequence must strictly increase");
        }
        prev_seq = Some(seq);
    }

    // The create's own artifact provenance is present and the search
    // operations correctly reflect the insert/remove mutations: search
    // before remove sees both id 101 and 102 in the top 2, search after
    // removing 102 no longer includes it.
    assert!(
        replies[0]["provenance"]["artifact"].is_object(),
        "{:?}",
        replies[0]
    );
    let search_1_ids: Vec<u64> = replies[2]["payload"]["hits"][0]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["external_id"].as_u64().unwrap())
        .collect();
    assert!(search_1_ids.contains(&102), "{search_1_ids:?}");
    let search_2_ids: Vec<u64> = replies[4]["payload"]["hits"][0]
        .as_array()
        .unwrap()
        .iter()
        .map(|h| h["external_id"].as_u64().unwrap())
        .collect();
    assert!(!search_2_ids.contains(&102), "{search_2_ids:?}");

    // The recover operation, run against a cleanly-committed index, reports
    // AlreadyCommitted -- not an error, not Indeterminate.
    assert_eq!(replies[5]["payload"]["outcome"], "AlreadyCommitted");

    // Session summary reflects the whole batch accurately.
    let summary = &summary_lines[0]["session_summary"];
    assert_eq!(summary["requests_submitted"], 6);
    assert_eq!(summary["requests_admitted"], 6);
    assert_eq!(summary["requests_rejected_pre_dispatch"], 0);
    assert_eq!(
        summary["first_logical_sequence"],
        replies[0]["logical_sequence"]
    );
    assert_eq!(
        summary["last_logical_sequence"],
        replies[5]["logical_sequence"]
    );
}

#[test]
fn session_summary_omits_sensitive_session_and_caller_metadata() {
    let dir = temp_dir("hub_cli_session_confidentiality");
    let request_id = "session-confidential-request";
    let session_id = "session-confidential-session";
    let caller_identity = "session-confidential-caller";
    let requests = dir.join("confidentiality_session.ndjson");
    let output_path = dir.join("confidentiality_session_output.ndjson");
    let request = serde_json::json!({
        "request_id": request_id,
        "caller_identity": caller_identity,
        "tool_id": "vector.turbovec",
        "operation_id": "vector.index.create",
        "capabilities": ["VectorIndexCreate", "PrivateStorageRead", "PrivateStorageWrite"],
        "payload": {"index": "session-confidentiality-docs", "dim": 8, "bit_width": 4}
    });
    fs::write(&requests, serde_json::to_string(&request).unwrap()).unwrap();

    let output = smc_in(
        &dir,
        &[
            "hub",
            "session",
            "--requests",
            requests.to_str().unwrap(),
            "--session-id",
            session_id,
            "--out",
            output_path.to_str().unwrap(),
        ],
    );
    assert!(output.status.success(), "{:?}", output);
    assert!(output.stdout.is_empty(), "{:?}", output);
    assert!(output.stderr.is_empty(), "{:?}", output);
    let output_text = fs::read_to_string(&output_path).unwrap();
    for sensitive_value in [session_id, caller_identity] {
        assert!(
            !output_text.contains(sensitive_value),
            "session stdout leaked {sensitive_value:?}: {output_text}"
        );
    }

    let lines: Vec<serde_json::Value> = output_text
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(lines.len(), 2, "one reply plus one summary");
    assert_eq!(lines[0]["request_id"], request_id);
    let summary = &lines[1]["session_summary"];
    assert!(summary.get("session_id").is_none(), "{summary}");
    assert!(summary.get("caller_identity").is_none(), "{summary}");
    assert_eq!(summary["requests_submitted"], 1);
    assert_eq!(summary["requests_admitted"], 1);

    let persisted_audit = fs::read_to_string(dir.join(".semantic/hub/audit.log")).unwrap();
    for persisted_value in [request_id, session_id, caller_identity] {
        assert!(
            persisted_audit.contains(persisted_value),
            "audit persistence unexpectedly lost {persisted_value:?}"
        );
    }
}

#[test]
fn session_mixed_caller_rejection_does_not_echo_caller_identity() {
    let dir = temp_dir("hub_cli_session_mixed_caller_confidentiality");
    let first_caller = "first-confidential-caller";
    let second_caller = "second-confidential-caller";
    let requests = dir.join("mixed_callers.ndjson");
    let request = |request_id: &str, caller_identity: &str| {
        serde_json::json!({
            "request_id": request_id,
            "caller_identity": caller_identity,
            "tool_id": "vector.turbovec",
            "operation_id": "vector.index.create",
            "capabilities": ["VectorIndexCreate", "PrivateStorageRead", "PrivateStorageWrite"],
            "payload": {"index": "mixed-caller-docs", "dim": 8, "bit_width": 4}
        })
    };
    fs::write(
        &requests,
        format!(
            "{}\n{}",
            serde_json::to_string(&request("mixed-caller-1", first_caller)).unwrap(),
            serde_json::to_string(&request("mixed-caller-2", second_caller)).unwrap(),
        ),
    )
    .unwrap();

    let output = smc_in(
        &dir,
        &["hub", "session", "--requests", requests.to_str().unwrap()],
    );
    assert!(!output.status.success(), "{:?}", output);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.starts_with("InputRejected: session requests use multiple caller identities"),
        "{stderr}"
    );
    for sensitive_value in [first_caller, second_caller] {
        assert!(
            !stderr.contains(sensitive_value),
            "session stderr leaked {sensitive_value:?}: {stderr}"
        );
    }
    assert!(output.stdout.is_empty());
}

#[test]
fn session_mutations_persist_across_the_process_boundary_and_are_reloadable_via_plain_invoke() {
    let dir = temp_dir("hub_cli_session_persist");
    let batch = session(&dir, "session_workflow.ndjson", &[]);
    assert!(batch.status.success(), "{:?}", batch);

    // A completely separate `smc hub invoke` process (not part of the
    // session) reloads the SAME index from disk and sees the mutations
    // the session batch committed.
    let describe_req = serde_json::json!({
        "capabilities": ["VectorIndexRead", "PrivateStorageRead"],
        "payload": {"index": "session-docs"}
    });
    let describe_input = dir.join("describe_session_docs.json");
    fs::write(&describe_input, serde_json::to_vec(&describe_req).unwrap()).unwrap();
    let describe = smc_in(
        &dir,
        &[
            "hub",
            "invoke",
            "vector.turbovec",
            "vector.index.describe",
            "--input",
            describe_input.to_str().unwrap(),
        ],
    );
    assert!(describe.status.success(), "{:?}", describe);
    // 3 inserted, 1 removed -> 2 remain.
    assert_eq!(stdout_json(&describe)["payload"]["len"], 2);
}

#[test]
fn session_rejects_further_requests_once_max_requests_ceiling_is_reached() {
    let dir = temp_dir("hub_cli_session_ceiling");
    let output = session(&dir, "session_workflow.ndjson", &["--max-requests", "2"]);
    assert!(!output.status.success(), "{:?}", output);
    assert!(
        String::from_utf8_lossy(&output.stderr).starts_with("SessionLimitExceeded"),
        "{:?}",
        output
    );
    assert!(
        output.stdout.is_empty(),
        "a batch that exceeds its intake ceiling must be rejected before dispatch"
    );
    assert!(
        !dir.join(".semantic/hub/vector.turbovec/session-docs.tvim")
            .exists(),
        "preflight rejection must not apply the first mutation"
    );
}

#[test]
fn session_rejects_a_non_numeric_max_requests_value() {
    let dir = temp_dir("hub_cli_session_bad_max_requests");
    let output = session(
        &dir,
        "session_workflow.ndjson",
        &["--max-requests", "not-a-number"],
    );
    assert!(!output.status.success(), "{:?}", output);
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("InvalidArguments"));
}

#[test]
fn session_cancel_line_rejects_the_named_later_request_with_cancelled() {
    let dir = temp_dir("hub_cli_session_cancel");
    let requests = dir.join("cancel_batch.ndjson");
    fs::write(
        &requests,
        [
            r#"{"request_id":"c-1","tool_id":"vector.turbovec","operation_id":"vector.index.create","capabilities":["VectorIndexCreate","PrivateStorageRead","PrivateStorageWrite"],"payload":{"index":"cancel-docs","dim":8,"bit_width":4}}"#,
            r#"{"cancel":"c-2"}"#,
            r#"{"request_id":"c-2","tool_id":"vector.turbovec","operation_id":"vector.index.describe","capabilities":["VectorIndexRead","PrivateStorageRead"],"payload":{"index":"cancel-docs"}}"#,
        ]
        .join("\n"),
    )
    .unwrap();
    let output = smc_in(
        &dir,
        &["hub", "session", "--requests", requests.to_str().unwrap()],
    );
    assert!(!output.status.success(), "{:?}", output);
    let lines = stdout_ndjson_lines(&output);
    assert_eq!(lines[0]["status"], "Success");
    assert_eq!(lines[1]["status"], "Rejected");
    assert_eq!(lines[1]["fault_code"], "Cancelled");
}

#[test]
fn session_cancel_line_after_its_target_is_still_applied_before_dispatch() {
    let dir = temp_dir("hub_cli_session_late_cancel");
    let requests = dir.join("late_cancel_batch.ndjson");
    fs::write(
        &requests,
        [
            r#"{"request_id":"late-cancel","tool_id":"vector.turbovec","operation_id":"vector.index.create","capabilities":["VectorIndexCreate","PrivateStorageRead","PrivateStorageWrite"],"payload":{"index":"late-cancel-docs","dim":8,"bit_width":4}}"#,
            r#"{"cancel":"late-cancel"}"#,
        ]
        .join("\n"),
    )
    .unwrap();
    let output = smc_in(
        &dir,
        &["hub", "session", "--requests", requests.to_str().unwrap()],
    );
    assert!(!output.status.success(), "{:?}", output);
    let lines = stdout_ndjson_lines(&output);
    assert_eq!(lines[0]["status"], "Rejected");
    assert_eq!(lines[0]["fault_code"], "Cancelled");
    assert!(!dir
        .join(".semantic/hub/vector.turbovec/late-cancel-docs.tvim")
        .exists());
}

#[test]
fn session_rejects_a_duplicate_request_id_within_the_same_batch() {
    let dir = temp_dir("hub_cli_session_dup_batch");
    let requests = dir.join("dup_batch.ndjson");
    fs::write(
        &requests,
        [
            r#"{"request_id":"dup-1","tool_id":"vector.turbovec","operation_id":"vector.index.create","capabilities":["VectorIndexCreate","PrivateStorageRead","PrivateStorageWrite"],"payload":{"index":"dup-docs","dim":8,"bit_width":4}}"#,
            r#"{"request_id":"dup-1","tool_id":"vector.turbovec","operation_id":"vector.index.describe","capabilities":["VectorIndexRead","PrivateStorageRead"],"payload":{"index":"dup-docs"}}"#,
        ]
        .join("\n"),
    )
    .unwrap();
    let output = smc_in(
        &dir,
        &["hub", "session", "--requests", requests.to_str().unwrap()],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("DuplicateRequestId"));
    assert!(output.stdout.is_empty());
    assert!(
        !dir.join(".semantic/hub/vector.turbovec/dup-docs.tvim")
            .exists(),
        "a duplicate later line must reject the complete batch before its first mutation"
    );
}

#[test]
fn session_rejects_a_request_id_already_used_by_a_prior_plain_invoke() {
    let dir = temp_dir("hub_cli_session_dup_prior");
    let create_req = serde_json::json!({
        "request_id": "already-used",
        "capabilities": ["VectorIndexCreate", "PrivateStorageRead", "PrivateStorageWrite"],
        "payload": {"index": "prior-docs", "dim": 8, "bit_width": 4}
    });
    let input = dir.join("create_already_used.json");
    fs::write(&input, serde_json::to_vec(&create_req).unwrap()).unwrap();
    let first = smc_in(
        &dir,
        &[
            "hub",
            "invoke",
            "vector.turbovec",
            "vector.index.create",
            "--input",
            input.to_str().unwrap(),
        ],
    );
    assert!(first.status.success(), "{:?}", first);

    let requests = dir.join("reuse_batch.ndjson");
    fs::write(
        &requests,
        r#"{"request_id":"already-used","tool_id":"vector.turbovec","operation_id":"vector.index.describe","capabilities":["VectorIndexRead","PrivateStorageRead"],"payload":{"index":"prior-docs"}}"#,
    )
    .unwrap();
    let output = smc_in(
        &dir,
        &["hub", "session", "--requests", requests.to_str().unwrap()],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("DuplicateRequestId"));
}

#[test]
fn session_malformed_json_line_is_a_typed_rejection_naming_the_line_number() {
    let dir = temp_dir("hub_cli_session_malformed");
    let requests = dir.join("malformed_batch.ndjson");
    fs::write(
        &requests,
        [
            r#"{"request_id":"ok-1","tool_id":"vector.turbovec","operation_id":"vector.index.create","capabilities":["VectorIndexCreate","PrivateStorageRead","PrivateStorageWrite"],"payload":{"index":"malformed-docs","dim":8,"bit_width":4}}"#,
            r#"{"tool_id": not valid json"#,
        ]
        .join("\n"),
    )
    .unwrap();
    let output = smc_in(
        &dir,
        &["hub", "session", "--requests", requests.to_str().unwrap()],
    );
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.starts_with("InputRejected"), "{stderr}");
    assert!(stderr.contains("line 2"), "{stderr}");
    assert!(output.stdout.is_empty());
    assert!(
        !dir.join(".semantic/hub/vector.turbovec/malformed-docs.tvim")
            .exists(),
        "a malformed later line must reject the complete batch before its first mutation"
    );
}

#[test]
fn session_with_no_valid_lines_succeeds_with_an_empty_summary() {
    let dir = temp_dir("hub_cli_session_empty");
    let requests = dir.join("empty_batch.ndjson");
    fs::write(&requests, "\n\n").unwrap();
    let output = smc_in(
        &dir,
        &["hub", "session", "--requests", requests.to_str().unwrap()],
    );
    assert!(output.status.success(), "{:?}", output);
    let lines = stdout_ndjson_lines(&output);
    assert_eq!(lines.len(), 1);
    let summary = &lines[0]["session_summary"];
    assert_eq!(summary["requests_submitted"], 0);
}

#[test]
fn recover_is_reachable_through_the_generic_invoke_command_not_a_dedicated_subcommand() {
    let dir = temp_dir("hub_cli_recover_via_invoke");
    invoke(&dir, "vector.index.create", "valid_index_create.json");
    invoke(&dir, "vector.index.insert", "valid_index_insert.json");

    let recover_req = serde_json::json!({
        "capabilities": ["VectorIndexMutate", "PrivateStorageRead", "PrivateStorageWrite"],
        "payload": {"index": "fixture-docs"}
    });
    let input = dir.join("recover.json");
    fs::write(&input, serde_json::to_vec(&recover_req).unwrap()).unwrap();
    let output = smc_in(
        &dir,
        &[
            "hub",
            "invoke",
            "vector.turbovec",
            "vector.index.recover",
            "--input",
            input.to_str().unwrap(),
        ],
    );
    assert!(output.status.success(), "{:?}", output);
    assert_eq!(
        stdout_json(&output)["payload"]["outcome"],
        "AlreadyCommitted"
    );
}
