//! `smc hub ...` -- CLI surface for Semantic Hub v0 (Issue #1553).
//!
//! ```text
//! smc hub tools
//! smc hub describe <tool-id>
//! smc hub invoke <tool-id> <operation-id> --input <file> [--out <file>]
//! smc hub audit --request <request-id>
//! ```
//!
//! Every invocation goes through [`semantic_hub::runtime::Hub::invoke`] --
//! there is no direct route from this module to `TurboVecAdapter` that
//! bypasses admission, budgets, or audit. The Hub CLI is one short-lived
//! process per invocation; persistent state (the TurboVec index files and
//! the audit log) lives under `.semantic/hub/` relative to the current
//! working directory and is reloaded by each invocation, matching the
//! project-local storage convention proposed by issue #1372.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use semantic_hub::runtime::Hub;
use semantic_hub::{
    HubApiVersion, HubAuditTrail, HubCallerIdentity, HubCapability, HubCapabilitySet,
    HubOperationId, HubPrivacyClass, HubRequest, HubRequestId, HubResourceBudget, HubSessionId,
    HubToolId, HUB_ENVELOPE_SCHEMA_VERSION,
};
use semantic_hub_turbovec::TurboVecAdapter;

/// Bound checked against file metadata before any read, per the same
/// convention as `smc look ui frame`'s `MAX_SOURCE_BYTES`. Applies only to
/// one caller-supplied request file, not to the audit log (see
/// `MAX_AUDIT_LOG_BYTES` below) -- the two have very different growth
/// profiles and must not share a limit.
const MAX_INPUT_BYTES: u64 = 8 * 1024 * 1024;
/// Bound on the whole persisted audit log, checked before every read.
/// `audit.log` grows by one bounded record (`MAX_AUDIT_RECORD_BYTES` in
/// `semantic_hub::audit`) per invocation for the lifetime of a project and
/// has no retention/rotation policy in v0 (a documented limitation) -- it
/// must not share `MAX_INPUT_BYTES` (sized for one caller-supplied request
/// file), or normal accumulated history would eventually make every
/// subsequent `smc hub invoke`/`smc hub audit` call fail permanently. This
/// is still a bound, not "unbounded", to reject a pathologically corrupted
/// or maliciously huge file before allocating; it is sized generously
/// rather than tuned to any expected real-world history size.
const MAX_AUDIT_LOG_BYTES: u64 = 512 * 1024 * 1024;
const HUB_DATA_DIR: &str = ".semantic/hub";
const AUDIT_LOG_FILE: &str = "audit.log";
const TURBOVEC_DATA_SUBDIR: &str = "vector.turbovec";
const PENDING_DIR: &str = "pending";
const CLI_ENVELOPE_SCHEMA_VERSION: u32 = 1;

pub fn cmd_hub(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("tools") => cmd_hub_tools(&args[1..]),
        Some("describe") => cmd_hub_describe(&args[1..]),
        Some("invoke") => cmd_hub_invoke(&args[1..]),
        Some("audit") => cmd_hub_audit(&args[1..]),
        _ => Err(format!("InvalidArguments: {}", hub_usage())),
    }
}

fn hub_usage() -> String {
    [
        "usage:",
        "  smc hub tools",
        "  smc hub describe <tool-id>",
        "  smc hub invoke <tool-id> <operation-id> --input <file> [--out <file>]",
        "  smc hub audit --request <request-id>",
    ]
    .join("\n")
}

fn hub_data_root() -> PathBuf {
    PathBuf::from(HUB_DATA_DIR)
}

fn build_hub() -> Hub {
    let mut hub = Hub::new();
    let turbovec_dir = hub_data_root().join(TURBOVEC_DATA_SUBDIR);
    let adapter = TurboVecAdapter::new(turbovec_dir, HubResourceBudget::V0_CEILING);
    hub.register_tool(Box::new(adapter))
        .expect("the one built-in vector.turbovec registration cannot conflict with itself");
    hub
}

/// The Hub CLI's own effect (e.g. an adapter's `.tvim` write) and the
/// durable audit record for that same invocation are two separate atomic
/// writes to two separate files -- true single-transaction commit across
/// both isn't attempted in v0. To still leave recoverable evidence if the
/// process is interrupted between them (a crash, a full disk on the audit
/// write, a permissions change mid-run), a small pending marker is written
/// *before* dispatch and removed only after the audit log write succeeds.
/// If dispatch mutated durable state but the marker is still present
/// afterward, that is itself the recoverable evidence: an operation was
/// attempted and possibly applied, but this invocation's audit record
/// never made it to disk. `smc hub audit` surfaces a stale marker
/// distinctly from a truly-unknown request_id instead of silently
/// reporting "unknown".
fn pending_marker_path(request_id: &HubRequestId) -> PathBuf {
    hub_data_root()
        .join(PENDING_DIR)
        .join(format!("{}.json", request_id.as_str()))
}

fn write_pending_marker(
    request_id: &HubRequestId,
    tool_id: &HubToolId,
    operation_id: &HubOperationId,
) -> Result<(), String> {
    let marker = serde_json::json!({
        "request_id": request_id.as_str(),
        "tool_id": tool_id.as_str(),
        "operation_id": operation_id.as_str(),
        "started_at_nanos": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0),
    });
    let text = serde_json::to_string_pretty(&marker)
        .map_err(|e| format!("InternalHubFault: could not render pending marker: {e}"))?;
    write_output_atomic(&pending_marker_path(request_id), &text)
}

fn clear_pending_marker(request_id: &HubRequestId) {
    // Best-effort: a failure to remove the marker only means a future
    // lookup sees stale-but-harmless evidence of a completed invocation,
    // never a false claim that one failed.
    let _ = fs::remove_file(pending_marker_path(request_id));
}

fn next_value(args: &[String], i: &mut usize, flag: &str) -> Result<String, String> {
    let value = args
        .get(*i + 1)
        .ok_or_else(|| format!("InvalidArguments: missing value after '{flag}'"))?;
    *i += 2;
    Ok(value.clone())
}

fn read_bounded(path: &str, max_bytes: u64) -> Result<Vec<u8>, String> {
    let meta = fs::metadata(path)
        .map_err(|e| format!("InputRejected: cannot stat input file '{path}': {e}"))?;
    if meta.len() > max_bytes {
        return Err(format!(
            "InputRejected: input file '{path}' is {} bytes, exceeding the maximum {max_bytes} bytes",
            meta.len()
        ));
    }
    fs::read(path).map_err(|e| format!("InputRejected: cannot read input file '{path}': {e}"))
}

/// Same atomic-write pattern as `ui_frame_inspect::write_output_atomic`
/// (write to a sibling temp file, `sync_all`, then rename): duplicated
/// locally rather than exposed from that unrelated module, since this is
/// the only Hub-owned write path and the two modules are not otherwise
/// coupled.
fn write_output_atomic(path: &Path, contents: &str) -> Result<(), String> {
    let dir = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    fs::create_dir_all(dir).map_err(|e| {
        format!(
            "AuditProvenanceFailure: cannot create '{}': {e}",
            dir.display()
        )
    })?;
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("smc-hub-out");
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp_path = dir.join(format!(
        ".{file_name}.tmp.{}.{}",
        std::process::id(),
        suffix
    ));

    let write_result = (|| {
        use std::io::Write;
        let mut f = fs::File::create(&tmp_path)?;
        f.write_all(contents.as_bytes())?;
        f.sync_all()
    })();
    if let Err(e) = write_result {
        let _ = fs::remove_file(&tmp_path);
        return Err(format!(
            "AuditProvenanceFailure: failed to write temporary file '{}': {e}",
            tmp_path.display()
        ));
    }
    if let Err(e) = fs::rename(&tmp_path, path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(format!(
            "AuditProvenanceFailure: failed to atomically replace '{}': {e}",
            path.display()
        ));
    }
    Ok(())
}

fn load_audit_trail(path: &Path) -> Result<HubAuditTrail, String> {
    if !path.is_file() {
        return Ok(HubAuditTrail::new());
    }
    let bytes = read_bounded(
        path.to_str()
            .ok_or("AuditProvenanceFailure: audit log path is not valid UTF-8")?,
        MAX_AUDIT_LOG_BYTES,
    )
    .map_err(|e| e.replace("InputRejected", "AuditProvenanceFailure"))?;
    let text = String::from_utf8(bytes)
        .map_err(|e| format!("AuditProvenanceFailure: audit log is not valid UTF-8: {e}"))?;
    HubAuditTrail::from_canonical_text(&text)
        .map_err(|e| format!("AuditProvenanceFailure: corrupt audit log: {e}"))
}

fn save_audit_trail(path: &Path, trail: &HubAuditTrail) -> Result<(), String> {
    write_output_atomic(path, &trail.to_canonical_text())
}

/// True if appending one more record (worst case
/// `semantic_hub::audit::MAX_AUDIT_RECORD_BYTES`) to a trail whose current
/// canonical-text length is `current_text_len` would push it past
/// `MAX_AUDIT_LOG_BYTES`. A pure function (no I/O) so the boundary can be
/// unit-tested directly instead of via a multi-hundred-megabyte fixture.
fn would_exceed_audit_log_cap(current_text_len: usize) -> bool {
    current_text_len as u64 + semantic_hub::audit::MAX_AUDIT_RECORD_BYTES as u64
        > MAX_AUDIT_LOG_BYTES
}

// ---------------------------------------------------------------------
// smc hub tools
// ---------------------------------------------------------------------

fn cmd_hub_tools(args: &[String]) -> Result<(), String> {
    if let Some(extra) = args.first() {
        return Err(format!("InvalidArguments: unexpected argument '{extra}'"));
    }
    let hub = build_hub();
    for tool in hub.registry().list() {
        let state = hub
            .registry()
            .worker_state(&tool.tool_id)
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        println!(
            "{}\t{}\t{}\t{}",
            tool.tool_id, tool.tool_version, tool.execution_mode, state
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------
// smc hub describe <tool-id>
// ---------------------------------------------------------------------

fn cmd_hub_describe(args: &[String]) -> Result<(), String> {
    let raw = args
        .first()
        .ok_or_else(|| "InvalidArguments: missing <tool-id>".to_string())?;
    if args.len() > 1 {
        return Err(format!(
            "InvalidArguments: unexpected argument '{}'",
            args[1]
        ));
    }
    let tool_id = HubToolId::new(raw.as_str()).map_err(|e| format!("InvalidArguments: {e}"))?;
    let hub = build_hub();
    let descriptor = hub
        .registry()
        .descriptor(&tool_id)
        .ok_or_else(|| format!("UnknownTool: {raw}"))?;

    println!("tool_id: {}", descriptor.tool_id);
    println!("name: {}", descriptor.name);
    println!("version: {}", descriptor.tool_version);
    println!("hub_api_version: {}", descriptor.hub_api_version);
    println!("execution_mode: {}", descriptor.execution_mode);
    println!("trust_class: {}", descriptor.trust_class);
    println!("adapter_provenance: {}", descriptor.adapter_provenance);
    println!("operations:");
    for op in &descriptor.operations {
        let caps: Vec<&str> = op
            .required_capabilities
            .iter()
            .map(|c| c.as_str())
            .collect();
        println!(
            "  - {} determinism={} mutates_tool_state={} required_capabilities=[{}]",
            op.operation_id,
            op.determinism,
            op.mutates_tool_state,
            caps.join(",")
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------
// smc hub invoke <tool-id> <operation-id> --input <file> [--out <file>]
// ---------------------------------------------------------------------

#[derive(serde::Deserialize)]
struct CliRequestFile {
    #[serde(default = "default_schema_version")]
    schema_version: u32,
    #[serde(default)]
    request_id: Option<String>,
    #[serde(default = "default_session_id")]
    session_id: String,
    #[serde(default = "default_caller_identity")]
    caller_identity: String,
    #[serde(default)]
    capabilities: Vec<String>,
    #[serde(default = "default_privacy_class")]
    privacy_class: String,
    #[serde(default)]
    resource_budget: Option<CliResourceBudgetOverride>,
    payload: serde_json::Value,
}

fn default_schema_version() -> u32 {
    CLI_ENVELOPE_SCHEMA_VERSION
}
fn default_session_id() -> String {
    "cli-session".to_string()
}
fn default_caller_identity() -> String {
    "cli:local".to_string()
}
fn default_privacy_class() -> String {
    "ProjectLocal".to_string()
}

#[derive(serde::Deserialize, Default)]
struct CliResourceBudgetOverride {
    wall_time_millis: Option<u64>,
    memory_bytes: Option<u64>,
    input_bytes: Option<u64>,
    output_bytes: Option<u64>,
    index_item_count: Option<u64>,
    vector_dimensions: Option<u32>,
    result_count: Option<u32>,
    queue_depth: Option<u32>,
    concurrent_requests: Option<u32>,
    storage_read_bytes: Option<u64>,
    storage_write_bytes: Option<u64>,
    audit_bytes: Option<u64>,
}

fn merge_budget(overrides: Option<CliResourceBudgetOverride>) -> HubResourceBudget {
    let ceiling = HubResourceBudget::V0_CEILING;
    let Some(o) = overrides else { return ceiling };
    HubResourceBudget {
        wall_time_millis: o.wall_time_millis.unwrap_or(ceiling.wall_time_millis),
        memory_bytes: o.memory_bytes.unwrap_or(ceiling.memory_bytes),
        input_bytes: o.input_bytes.unwrap_or(ceiling.input_bytes),
        output_bytes: o.output_bytes.unwrap_or(ceiling.output_bytes),
        index_item_count: o.index_item_count.unwrap_or(ceiling.index_item_count),
        vector_dimensions: o.vector_dimensions.unwrap_or(ceiling.vector_dimensions),
        result_count: o.result_count.unwrap_or(ceiling.result_count),
        queue_depth: o.queue_depth.unwrap_or(ceiling.queue_depth),
        concurrent_requests: o.concurrent_requests.unwrap_or(ceiling.concurrent_requests),
        storage_read_bytes: o.storage_read_bytes.unwrap_or(ceiling.storage_read_bytes),
        storage_write_bytes: o.storage_write_bytes.unwrap_or(ceiling.storage_write_bytes),
        audit_bytes: o.audit_bytes.unwrap_or(ceiling.audit_bytes),
    }
}

fn generate_request_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("req-{}-{nanos}", std::process::id())
}

fn cmd_hub_invoke(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        return Err(format!("InvalidArguments: {}", hub_usage()));
    }
    let tool_id_raw = &args[0];
    let operation_id_raw = &args[1];
    let mut input_path: Option<String> = None;
    let mut out_path: Option<String> = None;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => input_path = Some(next_value(args, &mut i, "--input")?),
            "--out" => out_path = Some(next_value(args, &mut i, "--out")?),
            other => return Err(format!("InvalidArguments: unexpected argument '{other}'")),
        }
    }
    let input_path =
        input_path.ok_or_else(|| "InvalidArguments: missing --input <file>".to_string())?;

    let tool_id =
        HubToolId::new(tool_id_raw.as_str()).map_err(|e| format!("InvalidArguments: {e}"))?;
    let operation_id = HubOperationId::new(operation_id_raw.as_str())
        .map_err(|e| format!("InvalidArguments: {e}"))?;

    let raw_input = read_bounded(&input_path, MAX_INPUT_BYTES)?;
    let request_file: CliRequestFile = serde_json::from_slice(&raw_input)
        .map_err(|e| format!("InputRejected: malformed request file: {e}"))?;
    if request_file.schema_version != CLI_ENVELOPE_SCHEMA_VERSION {
        return Err(format!(
            "SchemaVersionUnsupported: request file declares schema_version {}, this build supports {}",
            request_file.schema_version, CLI_ENVELOPE_SCHEMA_VERSION
        ));
    }

    let request_id = match request_file.request_id {
        Some(id) => HubRequestId::new(id).map_err(|e| format!("InvalidArguments: {e}"))?,
        None => HubRequestId::new(generate_request_id())
            .expect("generated request id always satisfies the handle charset/length rules"),
    };
    let session_id =
        HubSessionId::new(request_file.session_id).map_err(|e| format!("InvalidArguments: {e}"))?;
    let caller_identity = HubCallerIdentity::new(request_file.caller_identity)
        .map_err(|e| format!("InvalidArguments: {e}"))?;
    let privacy_class = HubPrivacyClass::parse(&request_file.privacy_class).ok_or_else(|| {
        format!(
            "InvalidArguments: unsupported privacy_class '{}'",
            request_file.privacy_class
        )
    })?;
    let mut capability_context = HubCapabilitySet::empty();
    for name in &request_file.capabilities {
        let cap = HubCapability::parse(name)
            .ok_or_else(|| format!("InvalidArguments: unknown capability '{name}'"))?;
        capability_context = capability_context.grant(cap);
    }
    let resource_budget = merge_budget(request_file.resource_budget);
    let payload = serde_json::to_vec(&request_file.payload)
        .map_err(|e| format!("InputRejected: payload is not representable as JSON: {e}"))?;

    let request_id_for_marker = request_id.clone();
    let tool_id_for_marker = tool_id.clone();
    let operation_id_for_marker = operation_id.clone();

    let request = HubRequest {
        schema_version: HUB_ENVELOPE_SCHEMA_VERSION,
        api_version: HubApiVersion::CURRENT,
        request_id,
        session_id,
        caller_identity,
        tool_id,
        operation_id,
        capability_context,
        privacy_class,
        resource_budget,
        payload,
    };

    let mut hub = build_hub();
    let audit_path = hub_data_root().join(AUDIT_LOG_FILE);
    let mut persisted_trail = load_audit_trail(&audit_path)?;
    hub.seed_next_sequence(persisted_trail.next_sequence());

    // Reject a reused request_id before writing anything: `find_by_request`
    // only ever returns the *first* matching record, so a second audited
    // invocation under the same id would be silently unreachable by
    // lookup; and reusing an id that still has an unresolved pending
    // marker would let a retry's own `clear_pending_marker` erase the
    // evidence that the *earlier*, still-unresolved invocation may have
    // applied its effect.
    if persisted_trail
        .find_by_request(&request_id_for_marker)
        .is_some()
        || pending_marker_path(&request_id_for_marker).is_file()
    {
        return Err(format!(
            "DuplicateRequestId: request_id '{}' was already used by a prior invocation \
             (either audited or still unresolved) -- request ids must be unique",
            request_id_for_marker.as_str()
        ));
    }

    // Preflight before dispatch: save_audit_trail itself never checks
    // MAX_AUDIT_LOG_BYTES, it always writes whatever's given to it. Without
    // this, a caller could keep invoking until audit.log crossed that cap,
    // at which point load_audit_trail's own read bound would make every
    // subsequent invoke/audit call -- mutating or read-only -- fail
    // permanently, since nothing in v0 rotates or trims the log. Rejecting
    // here, before the mutation runs, keeps the log itself always under
    // the cap and gives a clear, actionable error instead of a silent,
    // self-perpetuating outage discovered only on the next call.
    if would_exceed_audit_log_cap(persisted_trail.to_canonical_text().len()) {
        return Err(format!(
            "AuditLogFull: appending this invocation's record would grow '{}' past the \
             {MAX_AUDIT_LOG_BYTES} byte cap -- rotate or delete the audit log to continue",
            audit_path.display()
        ));
    }

    // Durable evidence-of-attempt written BEFORE dispatch: see
    // `write_pending_marker`'s doc comment for why this exists.
    write_pending_marker(
        &request_id_for_marker,
        &tool_id_for_marker,
        &operation_id_for_marker,
    )?;

    let reply = hub.invoke(request, None);

    for record in hub.audit().records() {
        persisted_trail.push(record.clone());
    }
    save_audit_trail(&audit_path, &persisted_trail)?;
    clear_pending_marker(&request_id_for_marker);

    let reply_json = build_cli_reply_json(&reply);
    let output_text = serde_json::to_string_pretty(&reply_json)
        .map_err(|e| format!("InternalHubFault: could not render reply: {e}"))?;
    match out_path {
        Some(path) => write_output_atomic(Path::new(&path), &output_text)?,
        None => println!("{output_text}"),
    }

    match &reply.status {
        semantic_hub::HubReplyStatus::Success => Ok(()),
        status => {
            let fault = status
                .fault()
                .expect("non-success status always carries a fault");
            Err(format!("{}: {fault}", fault.code()))
        }
    }
}

fn build_cli_reply_json(reply: &semantic_hub::HubReply) -> serde_json::Value {
    let payload_value = if reply.payload.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::from_slice(&reply.payload).unwrap_or_else(|_| {
            serde_json::Value::String(String::from_utf8_lossy(&reply.payload).into_owned())
        })
    };
    let (status_str, fault_code, fault_message) = match &reply.status {
        semantic_hub::HubReplyStatus::Success => ("Success", None, None),
        other => {
            let fault = other.fault().expect("checked above");
            (
                other.as_str(),
                Some(fault.code().to_string()),
                Some(fault.to_string()),
            )
        }
    };
    serde_json::json!({
        "schema_version": CLI_ENVELOPE_SCHEMA_VERSION,
        "request_id": reply.request_id.as_str(),
        "tool_id": reply.tool_id.as_str(),
        "tool_version": reply.tool_version.to_string(),
        "operation_id": reply.operation_id.as_str(),
        "status": status_str,
        "fault_code": fault_code,
        "fault_message": fault_message,
        "payload": payload_value,
        "resource_usage": {
            "wall_time_millis": reply.resource_usage.wall_time_millis,
            "input_bytes": reply.resource_usage.input_bytes,
            "output_bytes": reply.resource_usage.output_bytes,
        },
    })
}

// ---------------------------------------------------------------------
// smc hub audit --request <request-id>
// ---------------------------------------------------------------------

fn cmd_hub_audit(args: &[String]) -> Result<(), String> {
    let mut request_id_raw: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--request" => request_id_raw = Some(next_value(args, &mut i, "--request")?),
            other => return Err(format!("InvalidArguments: unexpected argument '{other}'")),
        }
    }
    let request_id_raw = request_id_raw
        .ok_or_else(|| "InvalidArguments: missing --request <request-id>".to_string())?;
    let request_id =
        HubRequestId::new(request_id_raw.as_str()).map_err(|e| format!("InvalidArguments: {e}"))?;

    let audit_path = hub_data_root().join(AUDIT_LOG_FILE);
    let trail = load_audit_trail(&audit_path)?;
    let record = match trail.find_by_request(&request_id) {
        Some(record) => record,
        None => {
            // Distinguish "never attempted" from "attempted but never
            // durably audited" (see `write_pending_marker`'s doc comment)
            // instead of reporting a bare, potentially misleading
            // UnknownRequest for the latter case.
            if pending_marker_path(&request_id).is_file() {
                return Err(format!(
                    "PendingUnresolved: an invocation for request_id '{request_id_raw}' was \
                     started but never durably audited (a crash or audit-write failure during \
                     that invocation) -- the underlying tool operation may or may not have \
                     applied; verify directly (e.g. `smc hub invoke ... vector.index.describe`)"
                ));
            }
            return Err(format!(
                "UnknownRequest: no audit record for request_id '{request_id_raw}'"
            ));
        }
    };

    println!("request_id: {}", record.request_id);
    println!("session_id: {}", record.session_id);
    println!("caller_identity: {}", record.caller_identity);
    println!("tool_id: {}", record.tool_id);
    println!("tool_version: {}", record.tool_version);
    println!("adapter_provenance: {}", record.adapter_provenance);
    println!("operation_id: {}", record.operation_id);
    println!("execution_mode: {}", record.execution_mode);
    println!("determinism: {}", record.determinism);
    println!("trust_class: {}", record.trust_class);
    println!("privacy_class: {}", record.privacy_class);
    println!("input_digest: {}", record.input_digest);
    println!("output_digest: {}", record.output_digest);
    println!("worker_state_after: {}", record.worker_state_after);
    println!("status: {}", record.status_code);
    println!("fault_code: {}", record.fault_code.unwrap_or("-"));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_hub_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "smc-hub-unit-{tag}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn read_bounded_accepts_a_file_over_the_request_file_limit_under_the_audit_log_limit() {
        // Regression test: the audit log must not share MAX_INPUT_BYTES
        // (sized for one caller-supplied request file). A file larger than
        // MAX_INPUT_BYTES but well under MAX_AUDIT_LOG_BYTES must still be
        // readable when checked against the audit-log-specific bound.
        let dir = temp_hub_dir("audit-size");
        let path = dir.join("big.log");
        let over_request_limit = vec![b'x'; (MAX_INPUT_BYTES + 1024) as usize];
        fs::write(&path, &over_request_limit).unwrap();

        assert!(
            read_bounded(path.to_str().unwrap(), MAX_INPUT_BYTES).is_err(),
            "sanity check: this file really does exceed MAX_INPUT_BYTES"
        );
        assert!(
            read_bounded(path.to_str().unwrap(), MAX_AUDIT_LOG_BYTES).is_ok(),
            "a file over MAX_INPUT_BYTES but under MAX_AUDIT_LOG_BYTES must be readable \
             when bounded against the audit-log-specific limit"
        );
    }

    #[test]
    fn would_exceed_audit_log_cap_rejects_only_once_the_projected_size_passes_the_cap() {
        // Regression test: save_audit_trail previously had no preflight at
        // all, so audit.log could grow past MAX_AUDIT_LOG_BYTES and every
        // subsequent invoke/audit call would then fail permanently at
        // load_audit_trail's own read cap. Exercised as a pure function
        // rather than by writing a real multi-hundred-megabyte fixture.
        let per_record_max = semantic_hub::audit::MAX_AUDIT_RECORD_BYTES as u64;
        assert!(!would_exceed_audit_log_cap(0));
        assert!(!would_exceed_audit_log_cap(
            (MAX_AUDIT_LOG_BYTES - per_record_max) as usize
        ));
        assert!(would_exceed_audit_log_cap(
            (MAX_AUDIT_LOG_BYTES - per_record_max + 1) as usize
        ));
    }
}
