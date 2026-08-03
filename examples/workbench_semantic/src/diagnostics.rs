//! Diagnostics parsing.
//!
//! Generic host responsibility -- turning real `smc` process output (JSON or
//! plain text) into structured entries. Knows nothing about job dispatch,
//! screens, or the ledger; callers own how origin_job_id is assigned and how
//! entries are stored.

#[derive(Debug, Clone)]
pub struct DiagnosticEntry {
    pub code: String,
    pub severity: String,
    pub family: String,
    pub message: String,
    pub file: String,
    pub line: i64,
    pub column: i64,
    pub origin_job_id: i32,
}

/// Real structured diagnostics from `smc 7hell <file> --json`.
pub fn parse_7hell_diagnostics(stdout: &str, origin_job_id: i32) -> Vec<DiagnosticEntry> {
    let mut out = Vec::new();
    let Ok(json) = serde_json::from_str::<serde_json::Value>(stdout) else {
        return out;
    };
    let Some(diags) = json.get("diagnostics").and_then(|d| d.as_array()) else {
        return out;
    };
    for d in diags {
        let code = d
            .get("code")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let severity = d
            .get("severity")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let family = d
            .get("stage")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let message = d
            .get("message_needle")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let source = d.get("source");
        let file = source
            .and_then(|s| s.get("file"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let line = source
            .and_then(|s| s.get("line"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let column = source
            .and_then(|s| s.get("column"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        out.push(DiagnosticEntry {
            code,
            severity,
            family,
            message,
            file,
            line,
            column,
            origin_job_id,
        });
    }
    out
}

/// Fallback diagnostics parser for plain-text compiler output such as:
/// `Error [E0000]: expected primary expression at line 1:1`
/// This is a best-effort real-text parse, not an invented format -- it only
/// extracts a diagnostic when the exact observed shape is present, and never
/// fabricates a line/column that was not in the source text.
pub fn parse_plain_diagnostics(
    text: &str,
    family: &str,
    origin_job_id: i32,
    fallback_file: &str,
) -> Vec<DiagnosticEntry> {
    let mut out = Vec::new();
    for raw_line in text.lines() {
        let line = raw_line.trim();
        let Some(rest) = line.strip_prefix("Error [") else {
            continue;
        };
        let Some(close) = rest.find(']') else {
            continue;
        };
        let code = rest[..close].to_string();
        let after_code = &rest[close + 1..];
        let Some(msg_start) = after_code.strip_prefix(':') else {
            continue;
        };
        let msg_start = msg_start.trim_start();

        let (message, loc_line, loc_col) = match msg_start.rfind(" at line ") {
            Some(idx) => {
                let message = msg_start[..idx].trim().to_string();
                let loc = &msg_start[idx + " at line ".len()..];
                let mut parts = loc.splitn(2, ':');
                let l = parts
                    .next()
                    .and_then(|s| s.trim().parse::<i64>().ok())
                    .unwrap_or(0);
                let c = parts
                    .next()
                    .and_then(|s| s.trim().parse::<i64>().ok())
                    .unwrap_or(0);
                (message, l, c)
            }
            None => (msg_start.to_string(), 0, 0),
        };

        out.push(DiagnosticEntry {
            code,
            severity: "error".to_string(),
            family: family.to_string(),
            message,
            file: fallback_file.to_string(),
            line: loc_line,
            column: loc_col,
            origin_job_id,
        });
    }
    out
}
