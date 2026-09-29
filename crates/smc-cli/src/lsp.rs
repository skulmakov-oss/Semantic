//! SSF-09 #1580: `smc lsp` - a diagnostics-first Language Server Protocol
//! baseline over stdio.
//!
//! Authority boundary (SSF-09 "Presentation/LSP boundary"): the server owns
//! no language semantics. Every diagnostic it publishes is a projection of
//! [`crate::canonical_check::check_canonical`] - the same path
//! `smc check --format human|json` renders - with only two transport
//! conversions: the canonical UTF-8 byte range becomes an LSP UTF-16
//! position range, and a diagnostic is routed to the document locator the
//! client supplied (or, for another source of the same check, that
//! source's `file:` URI). Formatting is the canonical formatter
//! ([`crate::formatter::format_source_checked`]), including its refusals.
//!
//! Determinism and staleness: messages are processed strictly in arrival
//! order and every request is answered before the next message is read, so
//! a published result always reflects exactly the document versions known
//! at that point. A `didChange` whose version is not newer than the stored
//! one is rejected as stale and changes nothing. `$/cancelRequest` can only
//! ever name an already-answered request and is therefore a no-op.
//!
//! The server reads only stdin and the project files the canonical check
//! itself reads; it writes only stdout. It opens no network connection and
//! sends no telemetry.

use crate::canonical_check::{
    check_canonical, check_standalone_text, CanonicalCheckReport, CheckRequest, HostCheckFailure,
    SourceOverlay,
};
use crate::diagnostic_schema::family_name;
use crate::formatter::format_source_checked;
use serde_json::{json, Value};
use sm_diagnostic::{Diagnostic, DiagnosticCause, DiagnosticSeverity};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

/// Upper bound on one message body; larger frames are a fatal protocol
/// error rather than an unbounded allocation.
pub const MAX_MESSAGE_BYTES: usize = 64 * 1024 * 1024;

const PARSE_ERROR: i64 = -32700;
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;
const SERVER_NOT_INITIALIZED: i64 = -32002;
const REQUEST_FAILED: i64 = -32803;

/// How a server session ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LspExit {
    /// `exit` after `shutdown`: process exit code 0.
    Clean,
    /// `exit` without `shutdown`, or end of input: exit code 1.
    Unclean,
    /// Unrecoverable framing error: exit code 1.
    Framing(String),
}

impl LspExit {
    pub fn code(&self) -> u8 {
        match self {
            LspExit::Clean => 0,
            LspExit::Unclean | LspExit::Framing(_) => 1,
        }
    }
}

struct Document {
    version: i64,
    text: String,
}

/// Server state. Generic over the transport so fixtures can drive it with
/// in-memory byte streams.
pub struct LspServer<W: Write> {
    out: W,
    initialized: bool,
    shutdown: bool,
    workspace_roots: Vec<PathBuf>,
    documents: BTreeMap<String, Document>,
    published: BTreeSet<String>,
}

/// Runs the server over the process's stdin/stdout.
pub fn serve_stdio() -> LspExit {
    let stdin = io::stdin();
    let stdout = io::stdout();
    serve(stdin.lock(), stdout.lock())
}

/// Runs the server until `exit`, end of input, or a framing error.
pub fn serve<R: BufRead, W: Write>(mut input: R, out: W) -> LspExit {
    let mut server = LspServer::new(out);
    loop {
        match read_message(&mut input) {
            Ok(None) => return LspExit::Unclean,
            Ok(Some(body)) => {
                if let Some(exit) = server.handle_body(&body) {
                    return exit;
                }
            }
            Err(message) => return LspExit::Framing(message),
        }
    }
}

/// Reads one `Content-Length`-framed message body. `Ok(None)` is a clean
/// end of input before any header byte.
pub fn read_message<R: BufRead>(input: &mut R) -> Result<Option<Vec<u8>>, String> {
    let mut content_length: Option<usize> = None;
    let mut saw_header = false;
    loop {
        let mut line = Vec::new();
        let read = input
            .read_until(b'\n', &mut line)
            .map_err(|e| format!("read failed: {e}"))?;
        if read == 0 {
            return if saw_header {
                Err("unexpected end of input inside message header".to_string())
            } else {
                Ok(None)
            };
        }
        saw_header = true;
        if !line.ends_with(b"\r\n") {
            return Err("message header line is not CRLF-terminated".to_string());
        }
        let line = &line[..line.len() - 2];
        if line.is_empty() {
            break;
        }
        let text =
            std::str::from_utf8(line).map_err(|_| "message header is not ASCII".to_string())?;
        let (name, value) = text
            .split_once(':')
            .ok_or_else(|| format!("malformed message header '{text}'"))?;
        if name.trim().eq_ignore_ascii_case("content-length") {
            if content_length.is_some() {
                return Err("duplicate Content-Length header".to_string());
            }
            let length = value
                .trim()
                .parse::<usize>()
                .map_err(|_| format!("invalid Content-Length '{}'", value.trim()))?;
            if length > MAX_MESSAGE_BYTES {
                return Err(format!(
                    "Content-Length {length} exceeds {MAX_MESSAGE_BYTES}"
                ));
            }
            content_length = Some(length);
        }
    }
    let length = content_length.ok_or_else(|| "missing Content-Length header".to_string())?;
    let mut body = vec![0u8; length];
    input
        .read_exact(&mut body)
        .map_err(|_| "unexpected end of input inside message body".to_string())?;
    Ok(Some(body))
}

/// Writes one framed message.
pub fn write_message<W: Write>(out: &mut W, value: &Value) -> io::Result<()> {
    let body = serde_json::to_string(value).expect("serde_json::Value always serializes");
    write!(out, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
    out.flush()
}

impl<W: Write> LspServer<W> {
    pub fn new(out: W) -> Self {
        Self {
            out,
            initialized: false,
            shutdown: false,
            workspace_roots: Vec::new(),
            documents: BTreeMap::new(),
            published: BTreeSet::new(),
        }
    }

    fn send(&mut self, value: Value) {
        // A broken stdout leaves nothing to report to; the next read ends
        // the session.
        let _ = write_message(&mut self.out, &value);
    }

    fn respond(&mut self, id: Value, result: Value) {
        self.send(json!({"jsonrpc": "2.0", "id": id, "result": result}));
    }

    fn respond_error(&mut self, id: Value, code: i64, message: &str) {
        self.send(json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {"code": code, "message": message}
        }));
    }

    fn log(&mut self, kind: i64, message: &str) {
        self.send(json!({
            "jsonrpc": "2.0",
            "method": "window/logMessage",
            "params": {"type": kind, "message": message}
        }));
    }

    /// Handles one message body; returns `Some` when the session ends.
    pub fn handle_body(&mut self, body: &[u8]) -> Option<LspExit> {
        let value: Value = match serde_json::from_slice(body) {
            Ok(value) => value,
            Err(_) => {
                self.respond_error(Value::Null, PARSE_ERROR, "message body is not valid JSON");
                return None;
            }
        };
        let Some(object) = value.as_object() else {
            self.respond_error(
                Value::Null,
                INVALID_REQUEST,
                "message must be a JSON object",
            );
            return None;
        };
        let id = object.get("id").cloned();
        if let Some(id) = &id {
            if !(id.is_i64() || id.is_u64() || id.is_string()) {
                self.respond_error(
                    Value::Null,
                    INVALID_REQUEST,
                    "request id must be an integer or string",
                );
                return None;
            }
        }
        if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
            self.respond_error(
                id.unwrap_or(Value::Null),
                INVALID_REQUEST,
                "missing or unsupported jsonrpc version",
            );
            return None;
        }
        let Some(method) = object.get("method").and_then(Value::as_str) else {
            // A response from the client: this server sends no requests,
            // so there is nothing to correlate it with.
            if id.is_none() {
                self.respond_error(Value::Null, INVALID_REQUEST, "message has no method");
            }
            return None;
        };
        let params = object.get("params").cloned().unwrap_or(Value::Null);
        match id {
            Some(id) => {
                self.handle_request(id, method, &params);
                None
            }
            None => self.handle_notification(method, &params),
        }
    }

    fn handle_request(&mut self, id: Value, method: &str, params: &Value) {
        if self.shutdown {
            self.respond_error(id, INVALID_REQUEST, "server is shut down");
            return;
        }
        if !self.initialized && method != "initialize" {
            self.respond_error(id, SERVER_NOT_INITIALIZED, "server is not initialized");
            return;
        }
        match method {
            "initialize" => {
                if self.initialized {
                    self.respond_error(id, INVALID_REQUEST, "server is already initialized");
                    return;
                }
                self.workspace_roots = initialize_roots(params);
                self.initialized = true;
                self.respond(
                    id,
                    json!({
                        "capabilities": {
                            "positionEncoding": "utf-16",
                            "textDocumentSync": {
                                "openClose": true,
                                "change": 1,
                                "save": {"includeText": false}
                            },
                            "documentFormattingProvider": true,
                            "workspace": {
                                "workspaceFolders": {"supported": true, "changeNotifications": true}
                            }
                        },
                        "serverInfo": {"name": "smc-lsp", "version": env!("CARGO_PKG_VERSION")}
                    }),
                );
            }
            "shutdown" => {
                self.shutdown = true;
                self.respond(id, Value::Null);
            }
            "textDocument/formatting" => self.formatting(id, params),
            _ => self.respond_error(
                id,
                METHOD_NOT_FOUND,
                &format!("unsupported method '{method}'"),
            ),
        }
    }

    fn handle_notification(&mut self, method: &str, params: &Value) -> Option<LspExit> {
        if method == "exit" {
            return Some(if self.shutdown {
                LspExit::Clean
            } else {
                LspExit::Unclean
            });
        }
        if !self.initialized || self.shutdown {
            return None;
        }
        match method {
            "initialized" | "$/cancelRequest" | "$/setTrace" => {}
            "textDocument/didOpen" => {
                let doc = &params["textDocument"];
                match (
                    doc["uri"].as_str(),
                    doc["version"].as_i64(),
                    doc["text"].as_str(),
                ) {
                    (Some(uri), Some(version), Some(text)) => {
                        self.documents.insert(
                            uri.to_string(),
                            Document {
                                version,
                                text: text.to_string(),
                            },
                        );
                        self.publish_all();
                    }
                    _ => self.log(1, "ignored textDocument/didOpen with invalid params"),
                }
            }
            "textDocument/didChange" => self.did_change(params),
            "textDocument/didClose" => match params["textDocument"]["uri"].as_str() {
                Some(uri) if self.documents.remove(uri).is_some() => self.publish_all(),
                _ => self.log(
                    2,
                    "ignored textDocument/didClose for a document that is not open",
                ),
            },
            "textDocument/didSave" => self.publish_all(),
            "workspace/didChangeWorkspaceFolders" => {
                let event = &params["event"];
                for removed in folder_paths(&event["removed"]) {
                    self.workspace_roots.retain(|root| root != &removed);
                }
                for added in folder_paths(&event["added"]) {
                    if !self.workspace_roots.contains(&added) {
                        self.workspace_roots.push(added);
                    }
                }
                self.publish_all();
            }
            other if other.starts_with("$/") => {}
            _ => {}
        }
        None
    }

    fn did_change(&mut self, params: &Value) {
        let doc = &params["textDocument"];
        let (Some(uri), Some(version)) = (doc["uri"].as_str(), doc["version"].as_i64()) else {
            self.log(1, "ignored textDocument/didChange with invalid params");
            return;
        };
        let changes = params["contentChanges"].as_array();
        let text = match changes.map(Vec::as_slice) {
            Some([change]) if change.get("range").is_none() => change["text"].as_str(),
            _ => None,
        };
        let Some(text) = text else {
            self.log(
                1,
                "ignored textDocument/didChange: only one full-document change is supported",
            );
            return;
        };
        let Some(current) = self.documents.get_mut(uri) else {
            self.log(
                2,
                "ignored textDocument/didChange for a document that is not open",
            );
            return;
        };
        if version <= current.version {
            let message = format!(
                "ignored stale textDocument/didChange for {uri}: version {version} is not newer than {}",
                current.version
            );
            self.log(2, &message);
            return;
        }
        current.version = version;
        current.text = text.to_string();
        self.publish_all();
    }

    fn formatting(&mut self, id: Value, params: &Value) {
        let Some(uri) = params["textDocument"]["uri"].as_str() else {
            self.respond_error(id, INVALID_PARAMS, "missing textDocument.uri");
            return;
        };
        let Some(doc) = self.documents.get(uri) else {
            self.respond_error(id, REQUEST_FAILED, "document is not open");
            return;
        };
        match format_source_checked(&doc.text) {
            Ok(formatted) if formatted == doc.text => self.respond(id, json!([])),
            Ok(formatted) => {
                let end = lsp_position(&doc.text, doc.text.len());
                self.respond(
                    id,
                    json!([{
                        "range": {"start": {"line": 0, "character": 0}, "end": end},
                        "newText": formatted
                    }]),
                );
            }
            Err(refusal) => {
                let message = format!("formatting refused: {refusal}");
                self.respond_error(id, REQUEST_FAILED, &message);
            }
        }
    }

    /// Re-checks every open document (in URI order) against the current
    /// overlay of all open documents and republishes every affected URI,
    /// clearing URIs that no longer carry diagnostics.
    fn publish_all(&mut self) {
        let mut overlay = SourceOverlay::new();
        for (uri, doc) in &self.documents {
            if let Some(path) = uri_to_path(uri) {
                overlay.insert(&path, doc.text.clone());
            }
        }
        let mut outgoing: BTreeMap<String, Vec<Value>> = BTreeMap::new();
        let uris: Vec<String> = self.documents.keys().cloned().collect();
        for uri in &uris {
            outgoing.entry(uri.clone()).or_default();
            for (target, diagnostic) in self.check_document(uri, &overlay) {
                let list = outgoing.entry(target).or_default();
                // Presentation-level deduplication: the same source's
                // diagnostic reached through two open roots is shown once.
                if !list.contains(&diagnostic) {
                    list.push(diagnostic);
                }
            }
        }
        let stale: Vec<String> = self
            .published
            .iter()
            .filter(|uri| !outgoing.contains_key(*uri))
            .cloned()
            .collect();
        for uri in stale {
            outgoing.insert(uri, Vec::new());
        }
        self.published = outgoing
            .iter()
            .filter(|(_, list)| !list.is_empty())
            .map(|(uri, _)| uri.clone())
            .collect();
        for (uri, diagnostics) in outgoing {
            let mut params = json!({"uri": uri, "diagnostics": diagnostics});
            if let Some(doc) = self.documents.get(&uri) {
                params["version"] = json!(doc.version);
            }
            self.send(json!({
                "jsonrpc": "2.0",
                "method": "textDocument/publishDiagnostics",
                "params": params
            }));
        }
    }

    /// Runs the canonical check for one open document and returns
    /// `(target uri, LSP diagnostic)` pairs in producer order.
    fn check_document(&self, uri: &str, overlay: &SourceOverlay) -> Vec<(String, Value)> {
        let doc = &self.documents[uri];
        let result = match uri_to_path(uri) {
            Some(path) => check_canonical(&CheckRequest {
                display_base: self
                    .workspace_roots
                    .iter()
                    .find(|root| path.starts_with(root))
                    .cloned(),
                entry: path,
                overlay: overlay.clone(),
                root_display_fallback: None,
            }),
            None => check_standalone_text(&doc.text),
        };
        match result {
            Ok(report) => project_report(uri, &report),
            Err(failure) => vec![(uri.to_string(), host_failure_diagnostic(&failure))],
        }
    }
}

fn project_report(uri: &str, report: &CanonicalCheckReport) -> Vec<(String, Value)> {
    let root = report.sources.first().map(|s| s.id);
    report
        .diagnostics
        .iter()
        .map(|d| {
            let ctx = d.source_context.as_ref();
            let source = ctx.and_then(|ctx| report.source(ctx.source));
            let target = match source {
                Some(source) if Some(source.id) != root => source
                    .host_path
                    .as_deref()
                    .map(path_to_uri)
                    .unwrap_or_else(|| uri.to_string()),
                _ => uri.to_string(),
            };
            let range = match (ctx.and_then(|c| c.range), source) {
                (Some(range), Some(source)) => Some(json!({
                    "start": lsp_position(&source.text, range.start().as_u64() as usize),
                    "end": lsp_position(&source.text, range.end().as_u64() as usize),
                })),
                _ => None,
            };
            // Related information: this diagnostic's related locations,
            // then each direct cause's own location and related locations,
            // in producer order. Entries without a proven range are omitted
            // here (they remain in `data.cause`).
            let mut located: Vec<(&sm_diagnostic::SourceContext, String)> = d
                .related_locations
                .iter()
                .map(|r| (&r.context, r.message.clone().unwrap_or_default()))
                .collect();
            let causes: &[Diagnostic] = match d.cause.as_deref() {
                None => &[],
                Some(DiagnosticCause::Diagnostic(child)) => std::slice::from_ref(child),
                Some(DiagnosticCause::Report(children)) => children,
            };
            for child in causes {
                if let Some(ctx) = &child.source_context {
                    located.push((
                        ctx,
                        format!("caused by {}: {}", child.code.as_str(), child.message.as_str()),
                    ));
                }
                for r in &child.related_locations {
                    located.push((&r.context, r.message.clone().unwrap_or_default()));
                }
            }
            let related: Vec<Value> = located
                .into_iter()
                .filter_map(|(context, message)| {
                    let source = report.source(context.source)?;
                    let range = context.range?;
                    let uri = if Some(source.id) == root {
                        uri.to_string()
                    } else {
                        path_to_uri(source.host_path.as_deref()?)
                    };
                    Some(json!({
                        "location": {
                            "uri": uri,
                            "range": {
                                "start": lsp_position(&source.text, range.start().as_u64() as usize),
                                "end": lsp_position(&source.text, range.end().as_u64() as usize),
                            }
                        },
                        "message": message,
                    }))
                })
                .collect();
            let mut diagnostic = lsp_diagnostic(d, range, source.is_some());
            if !related.is_empty() {
                diagnostic["relatedInformation"] = Value::Array(related);
            }
            (target, diagnostic)
        })
        .collect()
}

fn lsp_diagnostic(d: &Diagnostic, range: Option<Value>, source_bound: bool) -> Value {
    // LSP requires a range. When the canonical range is absent the
    // diagnostic is anchored at the document start and says so in `data`;
    // the anchor is a transport requirement, never a claimed location.
    let range_absent = range.is_none();
    let range = range.unwrap_or_else(
        || json!({"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 0}}),
    );
    let severity = match d.severity {
        DiagnosticSeverity::Error => 1,
        DiagnosticSeverity::Warning => 2,
    };
    let cause: Vec<Value> = match d.cause.as_deref() {
        None => Vec::new(),
        Some(DiagnosticCause::Diagnostic(child)) => vec![cause_entry(child)],
        Some(DiagnosticCause::Report(children)) => children.iter().map(cause_entry).collect(),
    };
    json!({
        "range": range,
        "severity": severity,
        "code": d.code.as_str(),
        "source": "semantic",
        "message": d.message.as_str(),
        "data": {
            "family": family_name(d.family),
            "rangeAbsent": range_absent,
            "sourceBound": source_bound,
            "notes": d.notes.iter().map(|n| n.message.clone()).collect::<Vec<_>>(),
            "proposal": d.fix_proposal.as_ref().map(|f| f.message.clone()),
            "cause": cause
        }
    })
}

fn cause_entry(d: &Diagnostic) -> Value {
    json!({"code": d.code.as_str(), "message": d.message.as_str()})
}

fn host_failure_diagnostic(failure: &HostCheckFailure) -> Value {
    // Not a canonical diagnostic (no producer code): shown so the editor
    // never looks clean while `smc check` fails.
    json!({
        "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 0}},
        "severity": 1,
        "source": "semantic",
        "message": format!("check could not run: {}", failure.message),
        "data": {"hostFailure": true, "rangeAbsent": true, "sourceBound": false}
    })
}

/// LSP position (0-based line, UTF-16 code-unit character) of byte
/// `offset` in `text`. Line terminators are `\n`, `\r\n`, and a lone `\r`,
/// as the protocol specifies.
pub fn lsp_position(text: &str, offset: usize) -> Value {
    let (line, character) = utf16_position(text, offset);
    json!({"line": line, "character": character})
}

/// `(line, character)` of byte `offset` under LSP's UTF-16 convention.
pub fn utf16_position(text: &str, offset: usize) -> (usize, usize) {
    let mut line = 0usize;
    let mut character = 0usize;
    let mut chars = text[..offset].chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                line += 1;
                character = 0;
            }
            '\n' => {
                line += 1;
                character = 0;
            }
            c => character += c.len_utf16(),
        }
    }
    // An offset between `\r` and `\n` of one CRLF still belongs to the
    // line the `\r` ends.
    if offset > 0
        && text.as_bytes().get(offset) == Some(&b'\n')
        && text.as_bytes()[offset - 1] == b'\r'
    {
        let before = utf16_position(text, offset - 1);
        return (before.0, before.1);
    }
    (line, character)
}

fn initialize_roots(params: &Value) -> Vec<PathBuf> {
    let mut roots = folder_paths(&params["workspaceFolders"]);
    if roots.is_empty() {
        if let Some(root) = params["rootUri"].as_str().and_then(uri_to_path) {
            roots.push(root);
        }
    }
    roots
}

fn folder_paths(folders: &Value) -> Vec<PathBuf> {
    folders
        .as_array()
        .map(|list| {
            list.iter()
                .filter_map(|folder| folder["uri"].as_str().and_then(uri_to_path))
                .collect()
        })
        .unwrap_or_default()
}

/// The filesystem path of a `file:` URI (percent-decoded), or `None` for
/// any other scheme, a remote host, or an undecodable URI.
pub fn uri_to_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let slash = rest.find('/')?;
    let (host, path) = (&rest[..slash], &rest[slash..]);
    if !host.is_empty() && !host.eq_ignore_ascii_case("localhost") {
        return None;
    }
    let decoded = percent_decode(path)?;
    if cfg!(windows) {
        // `/C:/x` -> `C:/x`
        let trimmed = decoded.strip_prefix('/').unwrap_or(&decoded);
        Some(PathBuf::from(trimmed))
    } else {
        Some(PathBuf::from(decoded))
    }
}

/// The `file:` URI of an absolute path, percent-encoding every byte outside
/// the RFC 3986 unreserved set and `/`.
pub fn path_to_uri(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    let mut out = String::from("file://");
    if !text.starts_with('/') {
        out.push('/');
    }
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~/:".contains(&byte) {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

fn percent_decode(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = text.get(i + 1..i + 3)?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_position_counts_code_units_not_bytes_or_chars() {
        let text = "é𝄞x\nb";
        // `é` is 2 bytes / 1 unit, `𝄞` is 4 bytes / 2 units.
        assert_eq!(utf16_position(text, 0), (0, 0));
        assert_eq!(utf16_position(text, 2), (0, 1));
        assert_eq!(utf16_position(text, 6), (0, 3));
        assert_eq!(utf16_position(text, 7), (0, 4));
        assert_eq!(utf16_position(text, 8), (1, 0));
    }

    #[test]
    fn utf16_position_honours_every_lsp_line_terminator() {
        assert_eq!(utf16_position("a\r\nb", 3), (1, 0));
        assert_eq!(utf16_position("a\r\nb", 2), (0, 1));
        assert_eq!(utf16_position("a\rb", 2), (1, 0));
    }

    #[test]
    fn file_uri_round_trip_percent_encodes() {
        let path = PathBuf::from("/tmp/a dir/ü.sm");
        let uri = path_to_uri(&path);
        assert_eq!(uri, "file:///tmp/a%20dir/%C3%BC.sm");
        if !cfg!(windows) {
            assert_eq!(uri_to_path(&uri), Some(path));
        }
        assert_eq!(uri_to_path("untitled:Untitled-1"), None);
        assert_eq!(uri_to_path("file://remote/x.sm"), None);
        assert_eq!(uri_to_path("file:///bad%zz"), None);
    }
}
