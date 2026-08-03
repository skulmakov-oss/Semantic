use crate::adapter::repo_root;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmlspBridgeRequest {
  pub workspace_root: String,
  pub relative_path: String,
  pub content: String,
  pub line: u32,
  pub character: u32,
  pub command: String,
  pub args: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SmlspBridgeResult {
  pub transport: String,
  pub command_line: Vec<String>,
  pub capabilities: Vec<String>,
  pub hover_markdown: Option<String>,
  pub definition_path: Option<String>,
  pub definition_line: Option<u32>,
  pub definition_character: Option<u32>,
  pub formatting_text: Option<String>,
  pub diagnostics: Vec<SmlspDiagnostic>,
  pub stderr: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SmlspDiagnostic {
  pub severity: String,
  pub code: Option<String>,
  pub message: String,
  pub line: u32,
  pub character: u32,
  pub end_line: u32,
  pub end_character: u32,
}

struct BridgeState {
  diagnostics: Vec<SmlspDiagnostic>,
  pending_notifications: Vec<String>,
}

impl BridgeState {
  fn new() -> Self {
    Self {
      diagnostics: Vec::new(),
      pending_notifications: Vec::new(),
    }
  }
}

pub fn run_smlsp_bridge(
  request: SmlspBridgeRequest,
) -> Result<SmlspBridgeResult, String> {
  let repo_root = repo_root()?;
  let workspace_root = resolve_workspace_root(&repo_root, &request.workspace_root)?;
  let document_path = resolve_document_path(&workspace_root, &request.relative_path)?;
  let command_name = normalized_command(&request.command)?;
  let mut command_line = vec![command_name.clone()];
  command_line.extend(request.args.clone());

  let mut child = Command::new(&command_name)
    .args(&request.args)
    .current_dir(&workspace_root)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .map_err(|error| {
      format!(
        "failed to start smlsp command '{}': {error}. Set a valid smlsp command in Workbench Settings, point it to a full executable path, or disable experimental workflows if smlsp is unavailable.",
        command_name
      )
    })?;

  let mut stdin = child
    .stdin
    .take()
    .ok_or_else(|| "failed to open smlsp stdin".to_string())?;
  let stdout = child
    .stdout
    .take()
    .ok_or_else(|| "failed to open smlsp stdout".to_string())?;
  let stderr = child
    .stderr
    .take()
    .ok_or_else(|| "failed to open smlsp stderr".to_string())?;

  let message_rx = spawn_stdout_reader(stdout);
  let stderr_rx = spawn_stderr_reader(stderr);
  let mut state = BridgeState::new();

  let root_uri = file_uri(&workspace_root);
  let document_uri = file_uri(&document_path);

  write_message(
    &mut stdin,
    &json!({
      "jsonrpc": "2.0",
      "id": 1,
      "method": "initialize",
      "params": {
        "processId": serde_json::Value::Null,
        "rootUri": root_uri,
        "workspaceFolders": [{
          "uri": file_uri(&workspace_root),
          "name": workspace_root.file_name().and_then(|value| value.to_str()).unwrap_or("workspace")
        }],
        "capabilities": {
          "textDocument": {
            "hover": { "contentFormat": ["markdown", "plaintext"] },
            "definition": { "linkSupport": true },
            "formatting": {}
          }
        },
        "clientInfo": {
          "name": "semantic-workbench",
          "version": env!("CARGO_PKG_VERSION")
        }
      }
    }),
  )?;

  let initialize_result = wait_for_response(&message_rx, &mut state, 1, Duration::from_secs(3))?;
  let capabilities = extract_capabilities(initialize_result.get("capabilities"));

  write_message(
    &mut stdin,
    &json!({
      "jsonrpc": "2.0",
      "method": "initialized",
      "params": {}
    }),
  )?;

  write_message(
    &mut stdin,
    &json!({
      "jsonrpc": "2.0",
      "method": "textDocument/didOpen",
      "params": {
        "textDocument": {
          "uri": document_uri,
          "languageId": "semantic",
          "version": 1,
          "text": request.content
        }
      }
    }),
  )?;

  let position = json!({
    "line": request.line,
    "character": request.character
  });

  write_message(
    &mut stdin,
    &json!({
      "jsonrpc": "2.0",
      "id": 2,
      "method": "textDocument/hover",
      "params": {
        "textDocument": { "uri": document_uri },
        "position": position
      }
    }),
  )?;

  write_message(
    &mut stdin,
    &json!({
      "jsonrpc": "2.0",
      "id": 3,
      "method": "textDocument/definition",
      "params": {
        "textDocument": { "uri": document_uri },
        "position": position
      }
    }),
  )?;

  write_message(
    &mut stdin,
    &json!({
      "jsonrpc": "2.0",
      "id": 4,
      "method": "textDocument/formatting",
      "params": {
        "textDocument": { "uri": document_uri },
        "options": {
          "tabSize": 2,
          "insertSpaces": true,
          "trimTrailingWhitespace": true,
          "insertFinalNewline": true,
          "trimFinalNewlines": true
        }
      }
    }),
  )?;

  let hover_result = wait_for_response(&message_rx, &mut state, 2, Duration::from_secs(3))?;
  let definition_result = wait_for_response(&message_rx, &mut state, 3, Duration::from_secs(3))?;
  let formatting_result = wait_for_response(&message_rx, &mut state, 4, Duration::from_secs(3))?;

  drain_notifications(&message_rx, &mut state, Duration::from_millis(250))?;

  let _ = write_message(
    &mut stdin,
    &json!({
      "jsonrpc": "2.0",
      "id": 99,
      "method": "shutdown",
      "params": serde_json::Value::Null
    }),
  );
  let _ = wait_for_response(&message_rx, &mut state, 99, Duration::from_secs(1));
  let _ = write_message(
    &mut stdin,
    &json!({
      "jsonrpc": "2.0",
      "method": "exit",
      "params": serde_json::Value::Null
    }),
  );
  drop(stdin);

  let _ = child.wait();
  let stderr_text = stderr_rx
    .recv_timeout(Duration::from_secs(1))
    .unwrap_or_default();

  let hover_markdown = extract_hover_text(&hover_result);
  let definition = extract_definition_location(&definition_result);
  let formatting_text = apply_formatting_result(&request.content, &formatting_result)?;

  Ok(SmlspBridgeResult {
    transport: "stdio".into(),
    command_line,
    capabilities,
    hover_markdown,
    definition_path: definition
      .as_ref()
      .map(|location| location.path.to_string_lossy().replace('\\', "/")),
    definition_line: definition.as_ref().map(|location| location.line),
    definition_character: definition.as_ref().map(|location| location.character),
    formatting_text,
    diagnostics: state.diagnostics,
    stderr: stderr_text,
  })
}

fn resolve_workspace_root(repo_root: &Path, workspace_root: &str) -> Result<PathBuf, String> {
  let requested = PathBuf::from(workspace_root);
  let absolute = if requested.is_absolute() {
    requested
  } else {
    repo_root.join(requested)
  };

  let canonical = absolute
    .canonicalize()
    .map_err(|error| format!("failed to resolve workspace root '{}': {error}", absolute.display()))?;

  if !canonical.starts_with(repo_root) {
    return Err("workspace root must stay inside the repository root".into());
  }

  Ok(canonical)
}

fn resolve_document_path(workspace_root: &Path, relative_path: &str) -> Result<PathBuf, String> {
  let candidate = workspace_root.join(relative_path);
  if !candidate.starts_with(workspace_root) {
    return Err("smlsp document path must stay inside the workspace root".into());
  }
  Ok(candidate)
}

fn normalized_command(command: &str) -> Result<String, String> {
  let value = command.trim();
  if value.is_empty() {
    return Err(
      "smlsp command cannot be empty. Configure a valid command in Workbench Settings or disable experimental workflows."
        .into(),
    );
  }
  Ok(value.to_string())
}

fn spawn_stdout_reader(stdout: impl Read + Send + 'static) -> Receiver<Result<Value, String>> {
  let (tx, rx) = mpsc::channel();
  thread::spawn(move || {
    let mut reader = BufReader::new(stdout);
    loop {
      match read_json_rpc_message(&mut reader) {
        Ok(Some(message)) => {
          if tx.send(Ok(message)).is_err() {
            break;
          }
        }
        Ok(None) => break,
        Err(error) => {
          let _ = tx.send(Err(error));
          break;
        }
      }
    }
  });
  rx
}

fn spawn_stderr_reader(stderr: impl Read + Send + 'static) -> Receiver<String> {
  let (tx, rx) = mpsc::channel();
  thread::spawn(move || {
    let mut reader = BufReader::new(stderr);
    let mut buffer = String::new();
    let _ = reader.read_to_string(&mut buffer);
    let _ = tx.send(buffer);
  });
  rx
}

fn read_json_rpc_message(reader: &mut impl BufRead) -> Result<Option<Value>, String> {
  let mut headers = HashMap::new();

  loop {
    let mut line = String::new();
    let bytes_read = reader
      .read_line(&mut line)
      .map_err(|error| format!("failed to read smlsp header: {error}"))?;

    if bytes_read == 0 {
      if headers.is_empty() {
        return Ok(None);
      }
      return Err("unexpected EOF while reading smlsp headers".into());
    }

    let trimmed = line.trim_end_matches(['\r', '\n']);
    if trimmed.is_empty() {
      break;
    }

    if let Some((name, value)) = trimmed.split_once(':') {
      headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
    }
  }

  let content_length = headers
    .get("content-length")
    .ok_or_else(|| "missing Content-Length header from smlsp".to_string())?
    .parse::<usize>()
    .map_err(|error| format!("invalid Content-Length header from smlsp: {error}"))?;

  let mut body = vec![0u8; content_length];
  reader
    .read_exact(&mut body)
    .map_err(|error| format!("failed to read smlsp body: {error}"))?;

  serde_json::from_slice::<Value>(&body)
    .map(Some)
    .map_err(|error| format!("failed to parse smlsp JSON-RPC message: {error}"))
}

fn write_message(writer: &mut impl Write, payload: &Value) -> Result<(), String> {
  let encoded = serde_json::to_vec(payload)
    .map_err(|error| format!("failed to serialize smlsp JSON-RPC message: {error}"))?;
  writer
    .write_all(format!("Content-Length: {}\r\n\r\n", encoded.len()).as_bytes())
    .and_then(|_| writer.write_all(&encoded))
    .and_then(|_| writer.flush())
    .map_err(|error| format!("failed to write smlsp message: {error}"))
}

fn wait_for_response(
  rx: &Receiver<Result<Value, String>>,
  state: &mut BridgeState,
  request_id: i64,
  timeout: Duration,
) -> Result<Value, String> {
  let deadline = Instant::now() + timeout;

  loop {
    let now = Instant::now();
    if now >= deadline {
      return Err(format!(
        "timed out waiting for smlsp response id={request_id}. The configured smlsp process may not be speaking the expected protocol or may need a valid executable path."
      ));
    }

    let remaining = deadline.saturating_duration_since(now);
    let message = rx
      .recv_timeout(remaining)
      .map_err(|_| {
        format!(
          "timed out waiting for smlsp response id={request_id}. The configured smlsp process may not be speaking the expected protocol or may need a valid executable path."
        )
      })??;

    if handle_notification(&message, state)? {
      continue;
    }

    if message.get("id").and_then(Value::as_i64) == Some(request_id) {
      if let Some(error) = message.get("error") {
        return Err(format!("smlsp request id={request_id} failed: {error}"));
      }
      return Ok(message
        .get("result")
        .cloned()
        .unwrap_or(Value::Null));
    }
  }
}

fn drain_notifications(
  rx: &Receiver<Result<Value, String>>,
  state: &mut BridgeState,
  timeout: Duration,
) -> Result<(), String> {
  let deadline = Instant::now() + timeout;

  loop {
    let now = Instant::now();
    if now >= deadline {
      return Ok(());
    }

    let remaining = deadline.saturating_duration_since(now);
    match rx.recv_timeout(remaining) {
      Ok(Ok(message)) => {
        let _ = handle_notification(&message, state)?;
      }
      Ok(Err(error)) => return Err(error),
      Err(_) => return Ok(()),
    }
  }
}

fn handle_notification(message: &Value, state: &mut BridgeState) -> Result<bool, String> {
  let Some(method) = message.get("method").and_then(Value::as_str) else {
    return Ok(false);
  };

  if method == "textDocument/publishDiagnostics" {
    state.diagnostics = extract_diagnostics(message.get("params"));
    return Ok(true);
  }

  state.pending_notifications.push(method.to_string());
  Ok(true)
}

fn extract_capabilities(capabilities: Option<&Value>) -> Vec<String> {
  let Some(capabilities) = capabilities else {
    return Vec::new();
  };

  let mut values = Vec::new();
  if capability_present(capabilities.get("hoverProvider")) {
    values.push("hover".into());
  }
  if capability_present(capabilities.get("definitionProvider")) {
    values.push("definition".into());
  }
  if capability_present(capabilities.get("documentFormattingProvider")) {
    values.push("formatting".into());
  }
  if capability_present(capabilities.get("textDocumentSync")) {
    values.push("diagnostics".into());
  }
  values
}

fn capability_present(value: Option<&Value>) -> bool {
  match value {
    Some(Value::Bool(flag)) => *flag,
    Some(Value::Null) | None => false,
    Some(_) => true,
  }
}

fn extract_hover_text(result: &Value) -> Option<String> {
  let contents = result.get("contents")?;

  if let Some(text) = contents.as_str() {
    return Some(text.to_string());
  }

  if let Some(value) = contents.get("value").and_then(Value::as_str) {
    return Some(value.to_string());
  }

  if let Some(items) = contents.as_array() {
    let joined = items
      .iter()
      .filter_map(|item| {
        item
          .as_str()
          .map(|value| value.to_string())
          .or_else(|| item.get("value").and_then(Value::as_str).map(|value| value.to_string()))
      })
      .collect::<Vec<_>>()
      .join("\n\n");
    if !joined.is_empty() {
      return Some(joined);
    }
  }

  None
}

struct DefinitionLocation {
  path: PathBuf,
  line: u32,
  character: u32,
}

fn extract_definition_location(result: &Value) -> Option<DefinitionLocation> {
  let location = if let Some(items) = result.as_array() {
    items.first()?
  } else {
    result
  };

  let uri = location.get("uri").and_then(Value::as_str)?;
  let range = location
    .get("range")
    .or_else(|| location.get("targetSelectionRange"))
    .or_else(|| location.get("targetRange"))?;
  let start = range.get("start")?;

  Some(DefinitionLocation {
    path: uri_to_path(uri)?,
    line: start.get("line").and_then(Value::as_u64).unwrap_or(0) as u32,
    character: start.get("character").and_then(Value::as_u64).unwrap_or(0) as u32,
  })
}

fn extract_diagnostics(params: Option<&Value>) -> Vec<SmlspDiagnostic> {
  params
    .and_then(|value| value.get("diagnostics"))
    .and_then(Value::as_array)
    .map(|diagnostics| {
      diagnostics
        .iter()
        .filter_map(|diagnostic| {
          let range = diagnostic.get("range")?;
          let start = range.get("start")?;
          let end = range.get("end")?;
          Some(SmlspDiagnostic {
            severity: diagnostic_severity_label(diagnostic.get("severity")),
            code: diagnostic
              .get("code")
              .map(|value| match value {
                Value::String(text) => text.clone(),
                other => other.to_string(),
              }),
            message: diagnostic
              .get("message")
              .and_then(Value::as_str)
              .unwrap_or("unnamed diagnostic")
              .to_string(),
            line: start.get("line").and_then(Value::as_u64).unwrap_or(0) as u32,
            character: start.get("character").and_then(Value::as_u64).unwrap_or(0) as u32,
            end_line: end.get("line").and_then(Value::as_u64).unwrap_or(0) as u32,
            end_character: end.get("character").and_then(Value::as_u64).unwrap_or(0) as u32,
          })
        })
        .collect::<Vec<_>>()
    })
    .unwrap_or_default()
}

fn diagnostic_severity_label(value: Option<&Value>) -> String {
  match value.and_then(Value::as_u64) {
    Some(1) => "error".into(),
    Some(2) => "warning".into(),
    Some(3) => "info".into(),
    Some(4) => "hint".into(),
    _ => "unknown".into(),
  }
}

fn apply_formatting_result(content: &str, result: &Value) -> Result<Option<String>, String> {
  let Some(edits) = result.as_array() else {
    return Ok(None);
  };
  if edits.is_empty() {
    return Ok(None);
  }

  let mut normalized = edits
    .iter()
    .filter_map(|edit| {
      Some(TextEdit {
        start_line: edit.get("range")?.get("start")?.get("line")?.as_u64()? as u32,
        start_character: edit
          .get("range")?
          .get("start")?
          .get("character")?
          .as_u64()? as u32,
        end_line: edit.get("range")?.get("end")?.get("line")?.as_u64()? as u32,
        end_character: edit
          .get("range")?
          .get("end")?
          .get("character")?
          .as_u64()? as u32,
        new_text: edit.get("newText")?.as_str()?.to_string(),
      })
    })
    .collect::<Vec<_>>();

  normalized.sort_by(|left, right| {
    right
      .start_line
      .cmp(&left.start_line)
      .then(right.start_character.cmp(&left.start_character))
  });

  let mut updated = content.to_string();
  for edit in normalized {
    let start = position_to_offset(&updated, edit.start_line, edit.start_character)?;
    let end = position_to_offset(&updated, edit.end_line, edit.end_character)?;
    updated.replace_range(start..end, &edit.new_text);
  }

  Ok(Some(updated))
}

struct TextEdit {
  start_line: u32,
  start_character: u32,
  end_line: u32,
  end_character: u32,
  new_text: String,
}

fn position_to_offset(content: &str, line: u32, character: u32) -> Result<usize, String> {
  let mut current_line = 0u32;
  let mut current_character = 0u32;

  for (offset, ch) in content.char_indices() {
    if current_line == line && current_character == character {
      return Ok(offset);
    }

    if ch == '\n' {
      current_line += 1;
      current_character = 0;
    } else {
      current_character += 1;
    }
  }

  if current_line == line && current_character == character {
    return Ok(content.len());
  }

  Err(format!(
    "formatting edit references invalid position {line}:{character}"
  ))
}

fn file_uri(path: &Path) -> String {
  let normalized = path.to_string_lossy().replace('\\', "/");
  if normalized.starts_with('/') {
    format!("file://{normalized}")
  } else {
    format!("file:///{normalized}")
  }
}

fn uri_to_path(uri: &str) -> Option<PathBuf> {
  let value = uri.strip_prefix("file:///").or_else(|| uri.strip_prefix("file://"))?;
  Some(PathBuf::from(value.replace('/', "\\")))
}

#[cfg(test)]
mod tests {
  use super::{apply_formatting_result, extract_capabilities, file_uri, position_to_offset};
  use serde_json::json;
  use std::path::PathBuf;

  #[test]
  fn file_uri_uses_windows_compatible_format() {
    let uri = file_uri(&PathBuf::from(r"C:\repo\file.sm"));
    assert_eq!(uri, "file:///C:/repo/file.sm");
  }

  #[test]
  fn capabilities_are_summarized_from_initialize_result() {
    let values = extract_capabilities(Some(&json!({
      "hoverProvider": true,
      "definitionProvider": { "workDoneProgress": false },
      "documentFormattingProvider": true,
      "textDocumentSync": 2
    })));
    assert_eq!(values, vec!["hover", "definition", "formatting", "diagnostics"]);
  }

  #[test]
  fn formatting_edits_apply_to_document() {
    let result = apply_formatting_result(
      "fn main(){\nreturn;\n}\n",
      &json!([
        {
          "range": {
            "start": { "line": 0, "character": 9 },
            "end": { "line": 1, "character": 0 }
          },
          "newText": " {\n    "
        }
      ]),
    )
    .unwrap()
    .unwrap();

    assert_eq!(result, "fn main() {\n    return;\n}\n");
  }

  #[test]
  fn positions_resolve_to_offsets() {
    let offset = position_to_offset("alpha\nbeta\n", 1, 2).unwrap();
    assert_eq!(&"alpha\nbeta\n"[offset..], "ta\n");
  }
}
