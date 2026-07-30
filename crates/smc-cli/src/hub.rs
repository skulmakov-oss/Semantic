//! `smc hub ...` -- CLI surface for Semantic Hub v0 (Issue #1553, #1526
//! completion pass).
//!
//! ```text
//! smc hub tools
//! smc hub describe <tool-id>
//! smc hub invoke <tool-id> <operation-id> --input <file> [--out <file>]
//! smc hub session --requests <file> [--out <file>]
//! smc hub audit --request <request-id>
//! ```
//!
//! Every invocation goes through [`semantic_hub::runtime::Hub::invoke`] (or,
//! for `smc hub session`, [`semantic_hub::runtime::Hub::invoke_in_session`]
//! via [`semantic_hub::HubSession`]) -- there is no direct route from this
//! module to `TurboVecAdapter` that bypasses admission, budgets, or audit.
//! The Hub CLI is one short-lived process per invocation (a `session` call
//! processes many requests within that one process, still exiting when the
//! batch completes); persistent state (the TurboVec index files and the
//! audit log) lives under `.semantic/hub/` relative to the current working
//! directory and is reloaded by each invocation, matching the
//! project-local storage convention proposed by issue #1372.
//!
//! There is no dedicated `smc hub recover` subcommand: recovery is just
//! another bounded operation (`vector.index.recover`) on the same
//! `vector.turbovec` tool, reachable through the existing generic `invoke`
//! (or `session`) commands like any other operation -- see
//! `docs/spec/hub/turbovec_adapter_v0.md`.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use semantic_hub::runtime::Hub;
use semantic_hub::{
    content_digest, HubApiVersion, HubAuditTrail, HubCallerIdentity, HubCapability,
    HubCapabilitySet, HubOperationId, HubPrivacyClass, HubRequest, HubRequestId, HubResourceBudget,
    HubSession, HubSessionCeiling, HubSessionId, HubToolId, HUB_ENVELOPE_SCHEMA_VERSION,
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
const PROJECT_LOCK_FILE: &str = "hub.lock";
const TURBOVEC_DATA_SUBDIR: &str = "vector.turbovec";
const PENDING_DIR: &str = "pending";
const CLI_ENVELOPE_SCHEMA_VERSION: u32 = 1;

pub fn cmd_hub(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("tools") => cmd_hub_tools(&args[1..]),
        Some("describe") => cmd_hub_describe(&args[1..]),
        Some("invoke") => cmd_hub_invoke(&args[1..]),
        Some("session") => cmd_hub_session(&args[1..]),
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
        "  smc hub session --requests <file> [--out <file>] [--max-requests <n>] [--session-id <id>]",
        "  smc hub audit --request <request-id>",
    ]
    .join("\n")
}

fn hub_data_root() -> PathBuf {
    PathBuf::from(HUB_DATA_DIR)
}

fn lexical_absolute(path: &Path) -> Result<PathBuf, String> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| format!("InternalHubFault: cannot resolve current directory: {e}"))?
            .join(path)
    };
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if !normalized.pop() {
                    return Err(format!(
                        "InvalidArguments: output path '{}' escapes its filesystem root",
                        path.display()
                    ));
                }
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    Ok(normalized)
}

/// Reject a caller-selected output file that aliases Hub-owned state.
/// `--out` is written only after execution, so allowing it to target
/// `.semantic/hub/audit.log`, a pending marker, or a TurboVec artifact
/// would turn an otherwise successful command into self-corruption.
fn validate_output_path(path: &Path) -> Result<(), String> {
    let hub_root = lexical_absolute(&hub_data_root())?;
    let output = lexical_absolute(path)?;
    if path.file_name().is_none() || output.is_dir() {
        return Err(format!(
            "InvalidArguments: --out path '{}' must name a file",
            path.display()
        ));
    }
    if output.starts_with(&hub_root) {
        return Err(format!(
            "ScopedStorageViolation: --out path '{}' is inside Hub-owned data root '{}'",
            path.display(),
            hub_data_root().display()
        ));
    }

    // Catch an existing symlink/alias whose lexical spelling is outside
    // the root but whose resolved target is inside it.
    if hub_root.exists() {
        let resolved_root = fs::canonicalize(&hub_root).map_err(|e| {
            format!(
                "InternalHubFault: cannot resolve Hub-owned data root '{}': {e}",
                hub_root.display()
            )
        })?;
        let resolved_output = if output.exists() {
            Some(fs::canonicalize(&output).map_err(|e| {
                format!(
                    "InvalidArguments: cannot resolve --out path '{}': {e}",
                    path.display()
                )
            })?)
        } else {
            output
                .parent()
                .filter(|parent| parent.exists())
                .map(|parent| {
                    let file_name = output.file_name().ok_or_else(|| {
                        format!(
                            "InvalidArguments: --out path '{}' does not name a file",
                            path.display()
                        )
                    })?;
                    fs::canonicalize(parent)
                        .map(|resolved| resolved.join(file_name))
                        .map_err(|e| {
                            format!(
                                "InvalidArguments: cannot resolve --out parent '{}': {e}",
                                parent.display()
                            )
                        })
                })
                .transpose()?
        };
        if resolved_output.is_some_and(|resolved| resolved.starts_with(&resolved_root)) {
            return Err(format!(
                "ScopedStorageViolation: --out path '{}' resolves inside Hub-owned data root '{}'",
                path.display(),
                hub_data_root().display()
            ));
        }
    }
    Ok(())
}

/// Reject a directory whose path contains a symlink/junction in any
/// component. Mirrors `semantic_hub_turbovec::storage::ScopedStorage`'s
/// `check_root_is_not_a_symlink` (walking every successively-longer
/// prefix via `symlink_metadata`, never `canonicalize`, which would
/// silently resolve through the very link this is checking for) -- that
/// check only ever protects the TurboVec adapter's own `.tvim` files,
/// reached from inside `Hub::invoke`. This module's own direct
/// filesystem writes (the project lock, the pending marker, the audit
/// log) never route through `ScopedStorage` at all, so without an
/// equivalent check here, a checked-out project supplying `.semantic` or
/// `.semantic/hub` as a pre-planted symlink would have every one of
/// these CLI-level reads/writes silently follow it, escaping the
/// project directory before the adapter's own check ever runs. A
/// directory that does not exist yet is not a violation.
fn check_dir_is_not_a_symlink(dir: &Path) -> Result<(), String> {
    let mut probe = PathBuf::new();
    for component in dir.components() {
        probe.push(component);
        match fs::symlink_metadata(&probe) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(format!(
                    "ScopedStorageViolation: '{}' is a symlink or reparse point, which could \
                     resolve outside the project directory",
                    probe.display()
                ));
            }
            Ok(_) => continue,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => {
                return Err(format!(
                    "InternalHubFault: could not stat '{}': {e}",
                    probe.display()
                ))
            }
        }
    }
    Ok(())
}

/// Reject a *leaf* file that is itself a symlink/junction.
/// `check_dir_is_not_a_symlink` only ever walks a directory's own
/// components -- it never sees a final file appended after it. A
/// perfectly ordinary `.semantic/hub/` directory can still contain a
/// pre-planted symlink AT `hub.lock`, `audit.log`, or a pending-marker
/// file's own path: `OpenOptions::open`, `fs::read`, and `path.is_file()`
/// all dereference a symlink by default, so without this, that leaf
/// symlink would be followed the same way a symlinked directory would.
/// A file that does not exist yet is not a violation.
fn check_file_is_not_a_symlink(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => Err(format!(
            "ScopedStorageViolation: '{}' is a symlink or reparse point, which could resolve \
             outside the project directory",
            path.display()
        )),
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!(
            "InternalHubFault: could not stat '{}': {e}",
            path.display()
        )),
    }
}

/// Acquire an exclusive, whole-project advisory lock (`std::fs::File::lock`,
/// which wraps `flock(2)` on Unix / `LockFileEx` on Windows) covering one
/// invocation's entire load-audit-trail -> dispatch -> save-audit-trail
/// sequence.
/// Without this, two `smc hub invoke` processes racing against the same
/// project directory could each load the same on-disk `audit.log`/`.tvim`
/// snapshot, mutate their own in-memory copy, and then atomically rename
/// their own replacement over the other's -- the last rename wins and the
/// other process's already-`Success`-reported record or mutation is
/// silently gone. Blocks (does not time out) until the lock is free: v0's
/// CLI is a single-user tool, so a second invocation simply queues behind
/// the first rather than racing it. The lock is released automatically
/// when the returned `File` is dropped, including on a process crash --
/// this is exactly why an OS-level advisory lock is used here instead of
/// a hand-rolled marker file, which would need its own stale-lock
/// detection to recover from an ungraceful exit.
fn acquire_project_lock() -> Result<fs::File, String> {
    let dir = hub_data_root();
    check_dir_is_not_a_symlink(&dir)?;
    fs::create_dir_all(&dir).map_err(|e| {
        format!(
            "InternalHubFault: could not create '{}': {e}",
            dir.display()
        )
    })?;
    let lock_path = dir.join(PROJECT_LOCK_FILE);
    check_file_is_not_a_symlink(&lock_path)?;
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&lock_path)
        .map_err(|e| {
            format!(
                "InternalHubFault: could not open project lock file '{}': {e}",
                lock_path.display()
            )
        })?;
    file.lock().map_err(|e| {
        format!(
            "InternalHubFault: could not acquire project lock '{}': {e}",
            lock_path.display()
        )
    })?;
    Ok(file)
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
/// Derives the marker's filename from a digest of `request_id`, never the
/// raw id text. `HubRequestId`'s own validation allows `:` (a legal
/// handle character in the Hub's ID model, and a legal filename
/// character on Unix), but `:` is reserved on Windows (drive letters,
/// alternate data streams) -- a colon-containing but otherwise
/// perfectly valid request_id would make this filename creation fail
/// there, and Windows' reserved device names (`CON`, `NUL`, `COM1`, ...)
/// and case-insensitive collisions are the same class of problem. The
/// digest (FNV-1a-64, already used elsewhere in this crate for
/// correlation fingerprints -- not cryptographic, but not needed to be:
/// this is an internal lookup key, not a security boundary) is always a
/// plain lowercase-hex string, filesystem-safe on every platform. The
/// real `request_id` is still recorded verbatim inside the marker's own
/// JSON content by `write_pending_marker`.
fn pending_marker_path(request_id: &HubRequestId) -> PathBuf {
    let digest = content_digest(request_id.as_str().as_bytes());
    hub_data_root()
        .join(PENDING_DIR)
        .join(format!("{digest:016x}.json"))
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
    let marker_path = pending_marker_path(request_id);
    if let Some(dir) = marker_path.parent() {
        check_dir_is_not_a_symlink(dir)?;
    }
    check_file_is_not_a_symlink(&marker_path)?;
    write_output_atomic(&marker_path, &text)
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
    if let Some(dir) = path.parent() {
        check_dir_is_not_a_symlink(dir)?;
    }
    check_file_is_not_a_symlink(path)?;
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
    if let Some(dir) = path.parent() {
        check_dir_is_not_a_symlink(dir)?;
    }
    check_file_is_not_a_symlink(path)?;
    // Streaming form (the "unbounded" header sentinel), not the exact-count
    // form: once any Hub command touches a project's audit log, it stays
    // in the append-friendly shape `smc hub session` needs for a cheap
    // append, rather than flip-flopping back to exact-count on every
    // single-invoke call.
    write_output_atomic(path, &trail.to_canonical_text_streaming())
}

/// True if appending one more record (worst case
/// `semantic_hub::audit::MAX_AUDIT_RECORD_BYTES`) to a trail whose current
/// canonical-text length is `current_text_len` would push it past
/// `MAX_AUDIT_LOG_BYTES`. A pure function (no I/O) so the boundary can be
/// unit-tested directly instead of via a multi-hundred-megabyte fixture.
fn would_exceed_audit_log_cap(current_text_len: u64, additional_records: usize) -> bool {
    let projected = (additional_records as u64)
        .checked_mul(semantic_hub::audit::MAX_AUDIT_RECORD_BYTES as u64)
        .and_then(|delta| current_text_len.checked_add(delta));
    projected.is_none_or(|bytes| bytes > MAX_AUDIT_LOG_BYTES)
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
#[serde(deny_unknown_fields)]
struct CliRequestFile {
    #[serde(default = "default_schema_version")]
    schema_version: u32,
    #[serde(default)]
    request_id: Option<String>,
    /// Required for `smc hub session` (each line names its own tool);
    /// left unset for `smc hub invoke`, which takes `<tool-id>` as a
    /// positional CLI argument instead. Providing both is not an error --
    /// the CLI argument always wins for `invoke`, this field is simply
    /// ignored there.
    #[serde(default)]
    tool_id: Option<String>,
    /// Same convention as `tool_id`, for `<operation-id>`.
    #[serde(default)]
    operation_id: Option<String>,
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
#[serde(deny_unknown_fields)]
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

/// Build a validated [`HubRequest`] from one parsed request file/line,
/// shared by `smc hub invoke` (which supplies `tool_id`/`operation_id` as
/// positional CLI arguments, via the `*_override` parameters) and
/// `smc hub session` (which reads them from the request line itself).
fn build_hub_request(
    request_file: CliRequestFile,
    tool_id_override: Option<HubToolId>,
    operation_id_override: Option<HubOperationId>,
) -> Result<HubRequest, String> {
    if request_file.schema_version != CLI_ENVELOPE_SCHEMA_VERSION {
        return Err(format!(
            "SchemaVersionUnsupported: request declares schema_version {}, this build supports {}",
            request_file.schema_version, CLI_ENVELOPE_SCHEMA_VERSION
        ));
    }
    let tool_id = match tool_id_override {
        Some(t) => t,
        None => {
            let raw = request_file
                .tool_id
                .as_deref()
                .ok_or_else(|| "InvalidArguments: missing \"tool_id\" field".to_string())?;
            HubToolId::new(raw).map_err(|e| format!("InvalidArguments: {e}"))?
        }
    };
    let operation_id = match operation_id_override {
        Some(o) => o,
        None => {
            let raw = request_file
                .operation_id
                .as_deref()
                .ok_or_else(|| "InvalidArguments: missing \"operation_id\" field".to_string())?;
            HubOperationId::new(raw).map_err(|e| format!("InvalidArguments: {e}"))?
        }
    };
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

    Ok(HubRequest {
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
    })
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
    if let Some(path) = out_path.as_deref() {
        validate_output_path(Path::new(path))?;
    }

    let tool_id =
        HubToolId::new(tool_id_raw.as_str()).map_err(|e| format!("InvalidArguments: {e}"))?;
    let operation_id = HubOperationId::new(operation_id_raw.as_str())
        .map_err(|e| format!("InvalidArguments: {e}"))?;

    let raw_input = read_bounded(&input_path, MAX_INPUT_BYTES)?;
    let request_file: CliRequestFile = serde_json::from_slice(&raw_input)
        .map_err(|e| format!("InputRejected: malformed request file: {e}"))?;
    let request = build_hub_request(request_file, Some(tool_id), Some(operation_id))?;

    let request_id_for_marker = request.request_id.clone();
    let tool_id_for_marker = request.tool_id.clone();
    let operation_id_for_marker = request.operation_id.clone();

    let mut hub = build_hub();
    let audit_path = hub_data_root().join(AUDIT_LOG_FILE);

    // Held for the rest of this function (dropped -- and so unlocked --
    // on every return path, success or error): see
    // `acquire_project_lock`'s doc comment for why this exists. Acquired
    // before the first read of shared on-disk state so the whole
    // load -> dispatch -> save sequence below is serialized against any
    // other `smc hub invoke` process for this same project.
    let _project_lock = acquire_project_lock()?;

    let mut persisted_trail = load_audit_trail(&audit_path)?;
    let next_sequence = persisted_trail.next_sequence().ok_or_else(|| {
        "AuditProvenanceFailure: audit sequence numbers exhausted (the last record is already \
         at u64::MAX) -- rotate or archive the audit log to continue"
            .to_string()
    })?;
    let _ = hub.seed_next_sequence(next_sequence);

    // Reject a reused request_id before writing anything: `find_by_request`
    // only ever returns the *first* matching record, so a second audited
    // invocation under the same id would be silently unreachable by
    // lookup; and reusing an id that still has an unresolved pending
    // marker would let a retry's own `clear_pending_marker` erase the
    // evidence that the *earlier*, still-unresolved invocation may have
    // applied its effect. Checked for symlinks the same way
    // `write_pending_marker`/`cmd_hub_audit` are: without this, a
    // pre-planted symlink at the pending directory or the marker leaf
    // would be silently followed by `.is_file()`, reporting
    // `DuplicateRequestId` based on a file outside the scoped Hub
    // directory instead of the real `ScopedStorageViolation`.
    let pending_marker_for_check = pending_marker_path(&request_id_for_marker);
    if let Some(dir) = pending_marker_for_check.parent() {
        check_dir_is_not_a_symlink(dir)?;
    }
    check_file_is_not_a_symlink(&pending_marker_for_check)?;
    if persisted_trail
        .find_by_request(&request_id_for_marker)
        .is_some()
        || pending_marker_for_check.is_file()
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
    let current_audit_bytes = fs::metadata(&audit_path).map(|m| m.len()).unwrap_or(0);
    if would_exceed_audit_log_cap(current_audit_bytes, 1) {
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

// ---------------------------------------------------------------------
// smc hub session --requests <file> [--out <file>]
// ---------------------------------------------------------------------

/// Bound on the whole newline-delimited requests file for one `smc hub
/// session` batch. Larger than `MAX_INPUT_BYTES` (sized for one
/// single-invoke request) since a legitimate batch is many requests, but
/// still a bound -- not "unbounded" -- to reject a pathologically huge or
/// corrupted file before allocating.
const MAX_SESSION_REQUESTS_FILE_BYTES: u64 = 64 * 1024 * 1024;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CliCancelLine {
    /// The `request_id` of an earlier or later line in the same batch to
    /// treat as pre-cancelled. See `semantic_hub::HubSession`'s own doc
    /// comment for the cooperative, pre-admission-only cancellation model
    /// this implements.
    cancel: String,
}

enum CliSessionLine {
    Cancel(HubRequestId),
    Request(Box<HubRequest>),
}

/// Parse and validate one NDJSON line from a session's requests file. A
/// line is either a control record (`{"cancel": "<request_id>"}`) or a
/// full request record (the same shape `smc hub invoke`'s `--input` file
/// uses, but with `tool_id`/`operation_id` required fields rather than
/// positional CLI arguments, since a batch can target different
/// operations -- even different tools, once more than one is registered
/// -- line by line).
///
/// Rejects a `request_id` that collides with an earlier line in this
/// batch. Persisted-audit and pending-marker uniqueness are checked once
/// for the fully parsed batch, under the project lock, before dispatch.
fn parse_session_line(
    raw_line: &str,
    seen_in_batch: &mut std::collections::BTreeSet<HubRequestId>,
) -> Result<CliSessionLine, String> {
    if raw_line.len() as u64 > MAX_INPUT_BYTES {
        return Err(format!(
            "InputRejected: session request record is {} bytes, exceeding the per-record maximum {MAX_INPUT_BYTES} bytes",
            raw_line.len()
        ));
    }
    let value: serde_json::Value = serde_json::from_str(raw_line)
        .map_err(|e| format!("InputRejected: malformed session line: {e}"))?;
    if value.get("cancel").is_some() {
        let cancel_line: CliCancelLine = serde_json::from_value(value)
            .map_err(|e| format!("InputRejected: malformed cancel line: {e}"))?;
        let id =
            HubRequestId::new(cancel_line.cancel).map_err(|e| format!("InvalidArguments: {e}"))?;
        return Ok(CliSessionLine::Cancel(id));
    }

    let request_file: CliRequestFile = serde_json::from_value(value)
        .map_err(|e| format!("InputRejected: malformed session request line: {e}"))?;
    let request = build_hub_request(request_file, None, None)?;

    if !seen_in_batch.insert(request.request_id.clone()) {
        return Err(format!(
            "DuplicateRequestId: request_id '{}' appears more than once in this session batch",
            request.request_id.as_str()
        ));
    }
    Ok(CliSessionLine::Request(Box::new(request)))
}

/// Execute a bounded, ordered batch of requests through one
/// [`semantic_hub::HubSession`] within this one CLI process, emitting one
/// structured JSON reply per line (in submission order) plus a final
/// `{"session_summary": {...}}` line, all as newline-delimited JSON
/// (matching the input format). Every admitted request's audit record is
/// durably appended (via [`HubAuditTrail::append_records_to_file`], a
/// true streaming append after the first request in a project's history)
/// immediately after that request completes -- before the next request in
/// the batch is even submitted -- so a crash partway through a large
/// batch loses at most the not-yet-appended tail, never a
/// already-completed-and-reported request's evidence.
///
/// Each request gets the same durable pre-dispatch pending marker as
/// `smc hub invoke`; it is cleared only after that request's audit record
/// has been synced. Thus a crash between a tool commit and audit append is
/// queryable as `PendingUnresolved`, never indistinguishable from an
/// unknown request.
fn cmd_hub_session(args: &[String]) -> Result<(), String> {
    let mut requests_path: Option<String> = None;
    let mut out_path: Option<String> = None;
    let mut max_requests: Option<u32> = None;
    let mut session_id_override: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--requests" => requests_path = Some(next_value(args, &mut i, "--requests")?),
            "--out" => out_path = Some(next_value(args, &mut i, "--out")?),
            "--session-id" => session_id_override = Some(next_value(args, &mut i, "--session-id")?),
            "--max-requests" => {
                let raw = next_value(args, &mut i, "--max-requests")?;
                max_requests = Some(raw.parse().map_err(|_| {
                    format!("InvalidArguments: --max-requests value '{raw}' is not a u32")
                })?);
            }
            other => return Err(format!("InvalidArguments: unexpected argument '{other}'")),
        }
    }
    let requests_path =
        requests_path.ok_or_else(|| "InvalidArguments: missing --requests <file>".to_string())?;
    if let Some(path) = out_path.as_deref() {
        validate_output_path(Path::new(path))?;
    }
    let ceiling = HubSessionCeiling {
        max_request_count: max_requests.unwrap_or(HubSessionCeiling::V0_DEFAULT.max_request_count),
        ..HubSessionCeiling::V0_DEFAULT
    };

    let raw = read_bounded(&requests_path, MAX_SESSION_REQUESTS_FILE_BYTES)?;
    let text = String::from_utf8(raw)
        .map_err(|e| format!("InputRejected: session requests file is not valid UTF-8: {e}"))?;

    // Parse and validate the complete bounded batch before acquiring shared
    // state or dispatching its first effect. A malformed or duplicate later
    // line must not turn the batch into an unreported partial application.
    let mut seen_in_batch = std::collections::BTreeSet::new();
    let mut parsed_lines = Vec::new();
    let mut request_count = 0usize;
    for (line_no, raw_line) in text.lines().enumerate() {
        let raw_line = raw_line.trim();
        if raw_line.is_empty() {
            continue;
        }
        let parsed = parse_session_line(raw_line, &mut seen_in_batch)
            .map_err(|e| format!("{e} (session requests file line {})", line_no + 1))?;
        if matches!(parsed, CliSessionLine::Request(_)) {
            request_count = request_count.checked_add(1).ok_or_else(|| {
                "SessionLimitExceeded: session request count overflowed".to_string()
            })?;
        }
        parsed_lines.push(parsed);
    }
    if request_count > ceiling.max_request_count as usize {
        return Err(format!(
            "SessionLimitExceeded: batch contains {request_count} requests, exceeding --max-requests {}",
            ceiling.max_request_count
        ));
    }
    let mut session_caller: Option<HubCallerIdentity> = None;
    let mut session_capabilities = HubCapabilitySet::empty();
    let mut session_privacy = HubPrivacyClass::PublicSafe;
    for parsed in &parsed_lines {
        let CliSessionLine::Request(request) = parsed else {
            continue;
        };
        match &session_caller {
            Some(caller) if caller != &request.caller_identity => {
                return Err(format!(
                "InputRejected: session requests use multiple caller identities ('{}' and '{}')",
                caller, request.caller_identity
            ))
            }
            None => session_caller = Some(request.caller_identity.clone()),
            _ => {}
        }
        for capability in request.capability_context.iter() {
            session_capabilities = session_capabilities.grant(*capability);
        }
        session_privacy = session_privacy.max(request.privacy_class);
    }
    let session_caller = session_caller.unwrap_or_else(|| {
        HubCallerIdentity::new(default_caller_identity())
            .expect("the built-in CLI caller identity is valid")
    });

    let mut hub = build_hub();
    let audit_path = hub_data_root().join(AUDIT_LOG_FILE);

    // Held for the whole batch: see `acquire_project_lock`'s doc comment.
    // A session batch is still exactly one logical unit of work against
    // one project, same as a single `smc hub invoke` call -- it must not
    // interleave with another `smc hub invoke`/`session` process's own
    // load -> dispatch -> append sequence.
    let _project_lock = acquire_project_lock()?;

    if let Some(dir) = audit_path.parent() {
        check_dir_is_not_a_symlink(dir)?;
    }
    check_file_is_not_a_symlink(&audit_path)?;
    let existing_audit_text = if audit_path.is_file() {
        let bytes = read_bounded(
            audit_path
                .to_str()
                .ok_or("AuditProvenanceFailure: audit log path is not valid UTF-8")?,
            MAX_AUDIT_LOG_BYTES,
        )
        .map_err(|e| e.replace("InputRejected", "AuditProvenanceFailure"))?;
        String::from_utf8(bytes)
            .map_err(|e| format!("AuditProvenanceFailure: audit log is not valid UTF-8: {e}"))?
    } else {
        String::new()
    };

    // Persisted and unresolved request-id uniqueness is checked for the
    // entire parsed batch before the first dispatch.
    for request_id in &seen_in_batch {
        if !existing_audit_text.is_empty()
            && HubAuditTrail::find_by_request_streaming(&existing_audit_text, request_id)
                .map_err(|e| format!("AuditProvenanceFailure: corrupt audit log: {e}"))?
                .is_some()
        {
            return Err(format!(
                "DuplicateRequestId: request_id '{}' was already used by a prior invocation",
                request_id.as_str()
            ));
        }
        let marker_path = pending_marker_path(request_id);
        if let Some(dir) = marker_path.parent() {
            check_dir_is_not_a_symlink(dir)?;
        }
        check_file_is_not_a_symlink(&marker_path)?;
        if marker_path.is_file() {
            return Err(format!(
                "DuplicateRequestId: request_id '{}' has an unresolved pending invocation",
                request_id.as_str()
            ));
        }
    }

    let current_audit_bytes = fs::metadata(&audit_path).map(|m| m.len()).unwrap_or(0);
    if would_exceed_audit_log_cap(current_audit_bytes, request_count) {
        return Err(format!(
            "AuditLogFull: appending {request_count} session records would grow '{}' past the \
             {MAX_AUDIT_LOG_BYTES} byte cap -- rotate or archive the audit log to continue",
            audit_path.display()
        ));
    }

    let next_sequence = if existing_audit_text.is_empty() {
        0u64
    } else {
        let seq = HubAuditTrail::next_sequence_streaming(&existing_audit_text)
            .map_err(|e| format!("AuditProvenanceFailure: corrupt audit log: {e}"))?;
        seq.ok_or_else(|| {
            "AuditProvenanceFailure: audit sequence numbers exhausted (the last record is \
             already at u64::MAX) -- rotate or archive the audit log to continue"
                .to_string()
        })?
    };
    let _ = hub.seed_next_sequence(next_sequence);

    let session_id = match session_id_override {
        Some(raw) => HubSessionId::new(raw).map_err(|e| format!("InvalidArguments: {e}"))?,
        None => HubSessionId::new(format!("session-{}", generate_request_id()))
            .expect("generated session id always satisfies the handle charset/length rules"),
    };
    let mut session = HubSession::new(
        session_id,
        session_caller,
        session_capabilities,
        session_privacy,
        ceiling,
    );

    let mut output_lines: Vec<String> = Vec::new();
    let mut had_failure = false;

    // Control records apply to the batch before deterministic execution,
    // including a cancellation line that appears after its target request.
    for parsed in &parsed_lines {
        if let CliSessionLine::Cancel(id) = parsed {
            session.cancel(id.clone());
        }
    }
    for parsed in parsed_lines {
        let CliSessionLine::Request(request) = parsed else {
            continue;
        };
        let request_id = request.request_id.clone();
        write_pending_marker(&request_id, &request.tool_id, &request.operation_id)?;
        let reply = session.submit(&mut hub, *request, None);
        if !reply.status.is_success() {
            had_failure = true;
        }
        let record = hub.audit().records().last().ok_or_else(|| {
            "AuditProvenanceFailure: Hub produced no audit record for a completed request"
                .to_string()
        })?;
        HubAuditTrail::append_records_to_file(&audit_path, std::slice::from_ref(record))
            .map_err(|e| format!("AuditProvenanceFailure: {e}"))?;
        clear_pending_marker(&request_id);

        let reply_json = build_cli_reply_json(&reply);
        output_lines.push(
            serde_json::to_string(&reply_json)
                .map_err(|e| format!("InternalHubFault: could not render reply: {e}"))?,
        );
    }

    let summary = session.summary();
    let summary_json = serde_json::json!({
        "session_summary": {
            "session_id": summary.session_id.as_str(),
            "caller_identity": summary.caller_identity.as_str(),
            "capability_ceiling": summary.capability_ceiling.iter().map(|cap| cap.as_str()).collect::<Vec<_>>(),
            "privacy_ceiling": summary.privacy_ceiling.to_string(),
            "requests_submitted": summary.requests_submitted,
            "requests_admitted": summary.requests_admitted,
            "requests_rejected_pre_dispatch": summary.requests_rejected_pre_dispatch,
            "cumulative_input_bytes": summary.cumulative_input_bytes,
            "cumulative_output_bytes": summary.cumulative_output_bytes,
            "cumulative_wall_time_millis": summary.cumulative_wall_time_millis,
            "first_logical_sequence": summary.first_logical_sequence,
            "last_logical_sequence": summary.last_logical_sequence,
        }
    });
    output_lines.push(
        serde_json::to_string(&summary_json)
            .map_err(|e| format!("InternalHubFault: could not render session summary: {e}"))?,
    );

    let output_text = output_lines.join("\n") + "\n";
    match out_path {
        Some(path) => write_output_atomic(Path::new(&path), &output_text)?,
        None => print!("{output_text}"),
    }

    if had_failure {
        Err(
            "SessionCompletedWithFailures: one or more requests in this session did not \
             succeed -- see the per-request replies above for fault codes"
                .to_string(),
        )
    } else {
        Ok(())
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
    let artifact = reply.provenance.artifact.as_ref().map(|a| {
        serde_json::json!({
            "kind": a.kind,
            "id": a.id,
            "digest": a.digest.to_string(),
        })
    });
    serde_json::json!({
        "schema_version": CLI_ENVELOPE_SCHEMA_VERSION,
        "request_id": reply.request_id.as_str(),
        "logical_sequence": reply.logical_sequence,
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
        "provenance": {
            "input_digest": reply.provenance.input_digest.to_string(),
            "output_digest": reply.provenance.output_digest.to_string(),
            "worker_state_after": reply.provenance.worker_state_after.to_string(),
            "artifact": artifact,
        },
        "warnings": reply.warnings,
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

    // Serialize the lookup against invoke/session append or migration so a
    // reader never observes an in-progress file replacement or tail write.
    let _project_lock = acquire_project_lock()?;
    let audit_path = hub_data_root().join(AUDIT_LOG_FILE);
    if let Some(dir) = audit_path.parent() {
        check_dir_is_not_a_symlink(dir)?;
    }
    check_file_is_not_a_symlink(&audit_path)?;
    // Streamed lookup: parses only up to the matching record, not the
    // whole history into a `Vec<HubAuditRecord>` just to linear-scan it
    // for one id (what `load_audit_trail` + `HubAuditTrail::find_by_request`
    // did before this completion pass).
    let audit_text = if audit_path.is_file() {
        let bytes = read_bounded(
            audit_path
                .to_str()
                .ok_or("AuditProvenanceFailure: audit log path is not valid UTF-8")?,
            MAX_AUDIT_LOG_BYTES,
        )
        .map_err(|e| e.replace("InputRejected", "AuditProvenanceFailure"))?;
        String::from_utf8(bytes)
            .map_err(|e| format!("AuditProvenanceFailure: audit log is not valid UTF-8: {e}"))?
    } else {
        String::new()
    };
    let found = if audit_text.is_empty() {
        None
    } else {
        HubAuditTrail::find_by_request_streaming(&audit_text, &request_id)
            .map_err(|e| format!("AuditProvenanceFailure: corrupt audit log: {e}"))?
    };
    let record = match &found {
        Some(record) => record,
        None => {
            // Distinguish "never attempted" from "attempted but never
            // durably audited" (see `write_pending_marker`'s doc comment)
            // instead of reporting a bare, potentially misleading
            // UnknownRequest for the latter case. Checked for symlinks
            // the same way the lock/audit-log paths are: without this, a
            // pre-planted symlink at the pending directory or the marker
            // leaf itself would be silently followed by `is_file()`,
            // letting an external target be mistaken for durable
            // evidence and reported as PendingUnresolved instead of the
            // real ScopedStorageViolation.
            let marker_path = pending_marker_path(&request_id);
            if let Some(dir) = marker_path.parent() {
                check_dir_is_not_a_symlink(dir)?;
            }
            check_file_is_not_a_symlink(&marker_path)?;
            if marker_path.is_file() {
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
    fn pending_marker_path_is_filesystem_safe_for_a_colon_containing_request_id() {
        // Regression test: HubRequestId's own validation allows ':' (a
        // legal handle character in the Hub's ID model, and a legal
        // Unix filename character), but ':' is reserved on Windows
        // (drive letters, alternate data streams). Before this fix,
        // the marker's filename embedded request_id.as_str() verbatim,
        // so an otherwise perfectly valid request_id containing ':'
        // would fail to create its marker file on Windows. The path is
        // now derived from a digest, always a plain lowercase-hex
        // string regardless of what characters the request_id itself
        // contains.
        let request_id = HubRequestId::new("req:with:colons").unwrap();
        let path = pending_marker_path(&request_id);
        let file_name = path.file_name().unwrap().to_str().unwrap();
        assert!(
            file_name
                .strip_suffix(".json")
                .is_some_and(|stem| !stem.is_empty() && stem.chars().all(|c| c.is_ascii_hexdigit())),
            "marker filename must be a plain hex stem + .json, got {file_name:?}"
        );
    }

    #[test]
    fn check_dir_is_not_a_symlink_accepts_a_real_or_not_yet_created_directory() {
        let dir = temp_hub_dir("symlink-check-real");
        assert!(check_dir_is_not_a_symlink(&dir).is_ok());
        assert!(check_dir_is_not_a_symlink(&dir.join("not-yet-created")).is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn check_dir_is_not_a_symlink_rejects_a_symlinked_ancestor() {
        // Regression test: none of this module's own direct writes (the
        // project lock, the pending marker, the audit log) route through
        // semantic_hub_turbovec::storage::ScopedStorage's symlink check --
        // that only protects the TurboVec adapter's own .tvim files. A
        // checked-out project supplying an ancestor like ".semantic" as a
        // pre-planted symlink must still be rejected here, before any of
        // this module's own reads/writes follow it.
        let real_target = std::env::temp_dir().join(format!(
            "smc-hub-unit-symlink-target-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&real_target).unwrap();
        let fake_ancestor = std::env::temp_dir().join(format!(
            "smc-hub-unit-symlink-ancestor-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::os::unix::fs::symlink(&real_target, &fake_ancestor).unwrap();

        let dir_through_link = fake_ancestor.join("hub").join("pending");
        let err = check_dir_is_not_a_symlink(&dir_through_link).unwrap_err();
        assert!(
            err.starts_with("ScopedStorageViolation"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn check_file_is_not_a_symlink_accepts_a_real_or_not_yet_created_file() {
        let dir = temp_hub_dir("leaf-check-real");
        let path = dir.join("audit.log");
        fs::write(&path, b"real content").unwrap();
        assert!(check_file_is_not_a_symlink(&path).is_ok());
        assert!(check_file_is_not_a_symlink(&dir.join("not-yet-created.log")).is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn check_file_is_not_a_symlink_rejects_a_symlinked_leaf_file() {
        // Regression test: check_dir_is_not_a_symlink only ever walks a
        // directory's own components -- it never sees a final file
        // appended after it. A perfectly ordinary ".semantic/hub/"
        // directory containing a pre-planted symlink AT hub.lock's or
        // audit.log's own path must still be rejected, since
        // OpenOptions::open/fs::read/path.is_file() all dereference a
        // symlink by default.
        let dir = temp_hub_dir("leaf-check-symlink");
        let outside_target = std::env::temp_dir().join(format!(
            "smc-hub-unit-leaf-symlink-target-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::write(&outside_target, b"not really an audit log").unwrap();

        let leaf_path = dir.join("audit.log");
        std::os::unix::fs::symlink(&outside_target, &leaf_path).unwrap();

        let err = check_file_is_not_a_symlink(&leaf_path).unwrap_err();
        assert!(
            err.starts_with("ScopedStorageViolation"),
            "unexpected error: {err}"
        );
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
        assert!(!would_exceed_audit_log_cap(0, 1));
        assert!(!would_exceed_audit_log_cap(
            MAX_AUDIT_LOG_BYTES - per_record_max,
            1
        ));
        assert!(would_exceed_audit_log_cap(
            MAX_AUDIT_LOG_BYTES - per_record_max + 1,
            1
        ));
        assert!(would_exceed_audit_log_cap(
            MAX_AUDIT_LOG_BYTES - per_record_max,
            2
        ));
    }

    #[test]
    fn resource_budget_override_rejects_a_misspelled_field_instead_of_silently_defaulting() {
        // Regression test: without deny_unknown_fields, a typo such as
        // "output_byte" (missing the trailing 's') was silently dropped by
        // serde, and merge_budget's unwrap_or(ceiling.field) then
        // substituted the full, generous V0_CEILING value for that
        // dimension instead of erroring -- exactly backwards from what a
        // caller narrowing their own budget intends.
        let result: Result<CliResourceBudgetOverride, _> =
            serde_json::from_str(r#"{"output_byte": 1}"#);
        assert!(result.is_err());
    }

    #[test]
    fn request_envelope_rejects_an_unknown_top_level_field() {
        // Regression test: without deny_unknown_fields on the outer
        // envelope, a typo such as "resource_budgett" (extra 't') was
        // silently dropped, resource_budget fell back to its
        // #[serde(default)] of None, and merge_budget(None) substitutes
        // the full, generous V0_CEILING for every dimension -- the same
        // fail-open-to-a-permissive-default pattern already fixed one
        // layer down in CliResourceBudgetOverride and payload.rs's
        // request structs.
        let result: Result<CliRequestFile, _> =
            serde_json::from_str(r#"{"resource_budgett": {"memory_bytes": 1}, "payload": {}}"#);
        assert!(result.is_err());
    }
}
