//! SSF-09 C6: the versioned external machine-readable diagnostic schema
//! (`semantic.diagnostics`, version 1), and the canonical human renderer
//! (C5). Both are explicit downstream projections of a
//! [`CanonicalCheckReport`]; neither is a `Serialize` derive on the internal
//! carrier, and neither adds, drops, reorders, or rewrites a diagnostic.
//!
//! The normative schema contract is
//! `docs/spec/diagnostics_machine_schema_v1.md`. Serialization is
//! hand-written so key order, whitespace, and escaping are fixed byte for
//! byte: identical reports always serialize to identical bytes.

use crate::canonical_check::{line_column, CanonicalCheckReport, CheckStatus, HostCheckFailure};
use sm_diagnostic::{
    Diagnostic, DiagnosticCause, DiagnosticFamily, DiagnosticSeverity, SourceContext,
};
use std::fmt::Write as _;

/// The schema identifier every document carries.
pub const DIAGNOSTIC_SCHEMA_NAME: &str = "semantic.diagnostics";
/// The schema version this build emits.
pub const DIAGNOSTIC_SCHEMA_VERSION: u32 = 1;

/// Serializes a completed check as a schema-v1 document.
pub fn render_json_report(report: &CanonicalCheckReport) -> String {
    let mut out = JsonOut::new();
    out.open_object();
    out.key("schema");
    out.string(DIAGNOSTIC_SCHEMA_NAME);
    out.key("schema_version");
    out.raw(&DIAGNOSTIC_SCHEMA_VERSION.to_string());
    out.key("status");
    out.string(match report.status {
        CheckStatus::Passed => "passed",
        CheckStatus::Failed => "failed",
    });
    out.key("sources");
    out.open_array();
    for (index, source) in report.sources.iter().enumerate() {
        out.item();
        out.open_object();
        out.key("index");
        out.raw(&index.to_string());
        out.key("path");
        out.opt_string(source.display_path.as_deref());
        out.key("identity");
        match &source.identity {
            Some(identity) => {
                out.open_object();
                out.key("package");
                out.string(&identity.package);
                out.key("module");
                out.string(&identity.module);
                out.close_object();
            }
            None => out.raw("null"),
        }
        out.close_object();
    }
    out.close_array();
    out.key("diagnostics");
    write_diagnostics(&mut out, report, &report.diagnostics);
    out.key("summary");
    out.open_object();
    out.key("errors");
    out.raw(&report.error_count().to_string());
    out.key("warnings");
    out.raw(&report.warning_count().to_string());
    out.close_object();
    out.key("failure");
    out.raw("null");
    out.close_object();
    out.finish()
}

/// Serializes a check that could not run as a schema-v1 document with
/// status `error`, no sources and no diagnostics.
pub fn render_json_failure(failure: &HostCheckFailure) -> String {
    let mut out = JsonOut::new();
    out.open_object();
    out.key("schema");
    out.string(DIAGNOSTIC_SCHEMA_NAME);
    out.key("schema_version");
    out.raw(&DIAGNOSTIC_SCHEMA_VERSION.to_string());
    out.key("status");
    out.string("error");
    out.key("sources");
    out.raw("[]");
    out.key("diagnostics");
    out.raw("[]");
    out.key("summary");
    out.open_object();
    out.key("errors");
    out.raw("0");
    out.key("warnings");
    out.raw("0");
    out.close_object();
    out.key("failure");
    out.open_object();
    out.key("message");
    out.string(&failure.message);
    out.close_object();
    out.close_object();
    out.finish()
}

fn write_diagnostics(out: &mut JsonOut, report: &CanonicalCheckReport, list: &[Diagnostic]) {
    out.open_array();
    for diagnostic in list {
        out.item();
        write_diagnostic(out, report, diagnostic);
    }
    out.close_array();
}

fn write_diagnostic(out: &mut JsonOut, report: &CanonicalCheckReport, d: &Diagnostic) {
    out.open_object();
    out.key("code");
    out.string(d.code.as_str());
    out.key("severity");
    out.string(severity_name(d.severity));
    out.key("family");
    out.string(family_name(d.family));
    out.key("message");
    out.string(d.message.as_str());
    write_context(out, report, d.source_context.as_ref());
    out.key("related");
    out.open_array();
    for related in &d.related_locations {
        out.item();
        out.open_object();
        write_context(out, report, Some(&related.context));
        out.key("message");
        out.opt_string(related.message.as_deref());
        out.close_object();
    }
    out.close_array();
    out.key("notes");
    out.open_array();
    for note in &d.notes {
        out.item();
        out.string(&note.message);
    }
    out.close_array();
    out.key("fix");
    out.opt_string(d.fix_proposal.as_ref().map(|f| f.message.as_str()));
    out.key("cause");
    match d.cause.as_deref() {
        None => out.raw("null"),
        Some(cause) => {
            let (kind, children): (&str, &[Diagnostic]) = match cause {
                DiagnosticCause::Diagnostic(child) => ("diagnostic", std::slice::from_ref(child)),
                DiagnosticCause::Report(children) => ("report", children),
            };
            out.open_object();
            out.key("kind");
            out.string(kind);
            out.key("diagnostics");
            write_diagnostics(out, report, children);
            out.close_object();
        }
    }
    out.close_object();
}

fn write_context(out: &mut JsonOut, report: &CanonicalCheckReport, ctx: Option<&SourceContext>) {
    let index = ctx.and_then(|ctx| report.source_index(ctx.source));
    out.key("source");
    match index {
        Some(index) => out.raw(&index.to_string()),
        None => out.raw("null"),
    }
    out.key("range");
    let located = ctx.and_then(|ctx| {
        let range = ctx.range?;
        let text = &report.source(ctx.source)?.text;
        Some((text, range))
    });
    match located {
        None => out.raw("null"),
        Some((text, range)) => {
            out.open_object();
            for (name, offset) in [("start", range.start()), ("end", range.end())] {
                let offset = offset.as_u64() as usize;
                let (line, column) = line_column(text, offset);
                out.key(name);
                out.open_object();
                out.key("offset");
                out.raw(&offset.to_string());
                out.key("line");
                out.raw(&line.to_string());
                out.key("column");
                out.raw(&column.to_string());
                out.close_object();
            }
            out.close_object();
        }
    }
}

pub(crate) fn severity_name(severity: DiagnosticSeverity) -> &'static str {
    match severity {
        DiagnosticSeverity::Error => "error",
        DiagnosticSeverity::Warning => "warning",
    }
}

pub(crate) fn family_name(family: DiagnosticFamily) -> &'static str {
    match family {
        DiagnosticFamily::Frontend => "frontend",
        DiagnosticFamily::Semantic => "semantic",
        DiagnosticFamily::Verification => "verification",
        DiagnosticFamily::Runtime => "runtime",
    }
}

/// Deterministic pretty JSON writer: two-space indentation, `\n` line
/// endings, one trailing newline, RFC 8259 string escaping with every
/// control character escaped.
struct JsonOut {
    buf: String,
    depth: usize,
    first: Vec<bool>,
}

impl JsonOut {
    fn new() -> Self {
        Self {
            buf: String::new(),
            depth: 0,
            first: Vec::new(),
        }
    }

    fn newline(&mut self) {
        self.buf.push('\n');
        for _ in 0..self.depth {
            self.buf.push_str("  ");
        }
    }

    fn separator(&mut self) {
        if let Some(first) = self.first.last_mut() {
            if *first {
                *first = false;
            } else {
                self.buf.push(',');
            }
        }
        self.newline();
    }

    fn open_object(&mut self) {
        self.buf.push('{');
        self.depth += 1;
        self.first.push(true);
    }

    fn close_object(&mut self) {
        self.close('}');
    }

    fn open_array(&mut self) {
        self.buf.push('[');
        self.depth += 1;
        self.first.push(true);
    }

    fn close_array(&mut self) {
        self.close(']');
    }

    fn close(&mut self, bracket: char) {
        self.depth -= 1;
        let empty = self.first.pop().unwrap_or(true);
        if !empty {
            self.newline();
        }
        self.buf.push(bracket);
    }

    fn key(&mut self, key: &str) {
        self.separator();
        self.push_string(key);
        self.buf.push_str(": ");
    }

    fn item(&mut self) {
        self.separator();
    }

    fn raw(&mut self, value: &str) {
        self.buf.push_str(value);
    }

    fn string(&mut self, value: &str) {
        self.push_string(value);
    }

    fn opt_string(&mut self, value: Option<&str>) {
        match value {
            Some(value) => self.push_string(value),
            None => self.buf.push_str("null"),
        }
    }

    fn push_string(&mut self, value: &str) {
        self.buf.push('"');
        for ch in value.chars() {
            match ch {
                '"' => self.buf.push_str("\\\""),
                '\\' => self.buf.push_str("\\\\"),
                '\n' => self.buf.push_str("\\n"),
                '\r' => self.buf.push_str("\\r"),
                '\t' => self.buf.push_str("\\t"),
                c if (c as u32) < 0x20 || c == '\u{7f}' => {
                    let _ = write!(self.buf, "\\u{:04x}", c as u32);
                }
                c => self.buf.push(c),
            }
        }
        self.buf.push('"');
    }

    fn finish(mut self) -> String {
        self.buf.push('\n');
        self.buf
    }
}

/// C5: canonical human rendering of a completed check. Plain text only:
/// no ANSI escapes, no absolute host paths, no terminal-width policy.
pub fn render_human_report(report: &CanonicalCheckReport) -> String {
    let mut out = String::new();
    for diagnostic in &report.diagnostics {
        render_human_diagnostic(&mut out, report, diagnostic, 0);
        out.push('\n');
    }
    let verdict = match report.status {
        CheckStatus::Passed => "check passed",
        CheckStatus::Failed => "check failed",
    };
    let _ = writeln!(
        out,
        "{verdict}: {} error(s), {} warning(s)",
        report.error_count(),
        report.warning_count()
    );
    out
}

/// C5 rendering of a check that could not run.
pub fn render_human_failure(failure: &HostCheckFailure) -> String {
    format!("check could not run: {}\n", failure.message)
}

fn render_human_diagnostic(
    out: &mut String,
    report: &CanonicalCheckReport,
    d: &Diagnostic,
    indent: usize,
) {
    let pad = " ".repeat(indent);
    let _ = writeln!(
        out,
        "{pad}{}[{}]: {}",
        severity_name(d.severity),
        d.code.as_str(),
        d.message.as_str()
    );
    if let Some(ctx) = &d.source_context {
        render_location(out, report, ctx, &pad);
    }
    for related in &d.related_locations {
        let label = related.message.as_deref().unwrap_or("related location");
        let _ = writeln!(out, "{pad}   = related: {label}");
        render_location(out, report, &related.context, &format!("{pad}   "));
    }
    for note in &d.notes {
        let _ = writeln!(out, "{pad}   = note: {}", note.message);
    }
    if let Some(fix) = &d.fix_proposal {
        let _ = writeln!(out, "{pad}   = proposal: {}", fix.message);
    }
    if let Some(cause) = d.cause.as_deref() {
        let _ = writeln!(out, "{pad}   = caused by:");
        let children: &[Diagnostic] = match cause {
            DiagnosticCause::Diagnostic(child) => std::slice::from_ref(child),
            DiagnosticCause::Report(children) => children,
        };
        for child in children {
            render_human_diagnostic(out, report, child, indent + 5);
        }
    }
}

fn render_location(
    out: &mut String,
    report: &CanonicalCheckReport,
    ctx: &SourceContext,
    pad: &str,
) {
    let Some(source) = report.source(ctx.source) else {
        return;
    };
    let name = match (&source.display_path, &source.identity) {
        (Some(path), _) => path.clone(),
        (None, Some(identity)) => format!("{}::{}", identity.package, identity.module),
        (None, None) => "(unnamed source)".to_string(),
    };
    let Some(range) = ctx.range else {
        let _ = writeln!(out, "{pad}  --> {name}");
        return;
    };
    let text = &source.text;
    let start = range.start().as_u64() as usize;
    let end = range.end().as_u64() as usize;
    let (line, column) = line_column(text, start);
    let _ = writeln!(out, "{pad}  --> {name}:{line}:{column}");
    let line_start = text[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = text[start..]
        .find('\n')
        .map(|i| start + i)
        .unwrap_or(text.len());
    let line_text = text[line_start..line_end].trim_end_matches('\r');
    let gutter = line.to_string();
    let blank = " ".repeat(gutter.len());
    let prefix: String = text[line_start..start]
        .chars()
        .map(|c| if c == '\t' { '\t' } else { ' ' })
        .collect();
    let underline_end = end.min(line_start + line_text.len()).max(start);
    let width = text[start..underline_end].chars().count().max(1);
    let _ = writeln!(out, "{pad} {blank} |");
    let _ = writeln!(out, "{pad} {gutter} | {line_text}");
    let _ = writeln!(out, "{pad} {blank} | {prefix}{}", "^".repeat(width));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_writer_escapes_every_control_character() {
        let mut out = JsonOut::new();
        out.string("a\"b\\c\nd\u{1}\u{7f}é");
        assert_eq!(out.buf, "\"a\\\"b\\\\c\\nd\\u0001\\u007fé\"");
    }

    #[test]
    fn json_writer_renders_empty_containers_compactly() {
        let mut out = JsonOut::new();
        out.open_object();
        out.key("a");
        out.open_array();
        out.close_array();
        out.close_object();
        assert_eq!(out.finish(), "{\n  \"a\": []\n}\n");
    }
}
