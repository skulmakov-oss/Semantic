use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SevenHellOutputMode {
    Human,
    Json,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SevenHellDiagnosticKind {
    SyntaxDiagnostic,
    CheckDiagnostic,
    BoundaryDenial,
}

impl SevenHellDiagnosticKind {
    fn as_human(self) -> &'static str {
        match self {
            Self::SyntaxDiagnostic => "syntax-diagnostic",
            Self::CheckDiagnostic => "check-diagnostic",
            Self::BoundaryDenial => "boundary-denial",
        }
    }

    fn as_json(self) -> &'static str {
        self.as_human()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SevenHellStageStatus {
    Pass,
    Fail,
    Blocked,
    NotImplemented,
}

impl SevenHellStageStatus {
    fn as_human(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::Blocked => "BLOCKED",
            Self::NotImplemented => "NOT_IMPLEMENTED",
        }
    }

    fn as_json(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Blocked => "blocked",
            Self::NotImplemented => "not_implemented",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SevenHellResult {
    Incomplete,
    Fail,
}

impl SevenHellResult {
    fn as_human(self) -> &'static str {
        match self {
            Self::Incomplete => "INCOMPLETE",
            Self::Fail => "FAIL",
        }
    }

    fn as_json(self) -> &'static str {
        match self {
            Self::Incomplete => "incomplete",
            Self::Fail => "fail",
        }
    }
}

struct SevenHellStageReport {
    index: usize,
    name: &'static str,
    key: &'static str,
    status: SevenHellStageStatus,
    summary: String,
    blocked_by: Option<&'static str>,
    diagnostic_ids: Vec<String>,
}

struct SevenHellDiagnosticSource {
    file: String,
    line: Option<u32>,
    column: Option<u32>,
}

struct SevenHellDiagnostic {
    id: String,
    stage: &'static str,
    kind: SevenHellDiagnosticKind,
    code: String,
    category: &'static str,
    message_needle: String,
    severity: &'static str,
    source: SevenHellDiagnosticSource,
}

struct SevenHellReport {
    target_display: String,
    target_normalized: String,
    result: SevenHellResult,
    stages: [SevenHellStageReport; 7],
    diagnostics: Vec<SevenHellDiagnostic>,
    boundary: &'static str,
}

struct SevenHellRenderOutput {
    rendered: String,
    success: bool,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if matches!(args.first().map(|s| s.as_str()), Some("7hell" | "seven-hell")) {
        return match run_7hell_command(&args[1..]) {
            Ok(code) => code,
            Err(e) => {
                eprintln!("{e}");
                ExitCode::from(1)
            }
        };
    }

    smc_cli::main_entry()
}

fn run_7hell_command(args: &[String]) -> Result<ExitCode, String> {
    let (target, output_mode) = parse_7hell_args(args)?;
    let outcome = execute_7hell_single_file(&target, output_mode);
    print!("{}", outcome.rendered);
    Ok(if outcome.success {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}

fn parse_7hell_args(args: &[String]) -> Result<(String, SevenHellOutputMode), String> {
    if args.is_empty() {
        return Err("usage: smc 7hell <input.sm> [--json]".to_string());
    }

    let mut output_mode = SevenHellOutputMode::Human;
    let mut target: Option<String> = None;

    for arg in args {
        match arg.as_str() {
            "--json" => output_mode = SevenHellOutputMode::Json,
            "--help" | "-h" => return Err("usage: smc 7hell <input.sm> [--json]".to_string()),
            value if value.starts_with('-') => return Err(format!("unknown flag '{}'", value)),
            value => {
                if target.is_some() {
                    return Err("usage: smc 7hell <input.sm> [--json]".to_string());
                }
                target = Some(value.to_string());
            }
        }
    }

    let target = target.ok_or_else(|| "usage: smc 7hell <input.sm> [--json]".to_string())?;
    Ok((target, output_mode))
}

fn execute_7hell_single_file(target: &str, output_mode: SevenHellOutputMode) -> SevenHellRenderOutput {
    let target_display = display_path_for_report(target);
    let report = match fs::read_to_string(Path::new(target)) {
        Ok(source) => match smc_cli::CliPipeline::semantic_check_source(&source) {
            Ok(_) => build_passed_7hell_report(target_display),
            Err(error_text) => {
                let diagnostic = diagnostic_from_check_error(&error_text, &target_display);
                build_failed_7hell_report(target_display, diagnostic)
            }
        },
        Err(_) => build_failed_7hell_report(
            target_display.clone(),
            boundary_denial_diagnostic(&target_display),
        ),
    };
    let success = matches!(report.result, SevenHellResult::Incomplete);
    SevenHellRenderOutput {
        rendered: render_7hell_report(&report, output_mode),
        success,
    }
}

fn display_path_for_report(path: &str) -> String {
    let path = Path::new(path);
    let candidate = if path.is_absolute() {
        match std::env::current_dir() {
            Ok(cwd) => path
                .strip_prefix(&cwd)
                .map(Path::to_path_buf)
                .unwrap_or_else(|_| {
                    path.file_name()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| PathBuf::from("."))
                }),
            Err(_) => path
                .file_name()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(".")),
        }
    } else {
        path.to_path_buf()
    };

    let raw = candidate.to_string_lossy().replace('\\', "/");
    if raw.is_empty() {
        ".".to_string()
    } else {
        raw
    }
}

fn build_passed_7hell_report(target_display: String) -> SevenHellReport {
    SevenHellReport {
        target_display: target_display.clone(),
        target_normalized: target_display,
        result: SevenHellResult::Incomplete,
        stages: [
            stage_report(
                1,
                "Syntax Hell",
                "syntax",
                SevenHellStageStatus::Pass,
                "single-file check accepted",
                None,
                vec![],
            ),
            stage_report(
                2,
                "Type Hell",
                "type",
                SevenHellStageStatus::Pass,
                "single-file check accepted",
                None,
                vec![],
            ),
            stage_report(
                3,
                "Lowering Hell",
                "lowering",
                SevenHellStageStatus::NotImplemented,
                "7HELL-S2 does not lower",
                None,
                vec![],
            ),
            stage_report(
                4,
                "Verifier Hell",
                "verifier",
                SevenHellStageStatus::NotImplemented,
                "7HELL-S2 does not verify",
                None,
                vec![],
            ),
            stage_report(
                5,
                "VM Hell",
                "vm",
                SevenHellStageStatus::NotImplemented,
                "7HELL-S2 does not run VM",
                None,
                vec![],
            ),
            stage_report(
                6,
                "Practical Hell",
                "practical",
                SevenHellStageStatus::NotImplemented,
                "7HELL-S2 does not execute practical stage",
                None,
                vec![],
            ),
            stage_report(
                7,
                "User Pain / Diagnostics Hell",
                "diagnostics",
                SevenHellStageStatus::NotImplemented,
                "7HELL-S2 does not wire diagnostics",
                None,
                vec![],
            ),
        ],
        diagnostics: Vec::new(),
        boundary: "S2 single-file check only; no compile, verify, VM run, project-root, CI gate, release readiness, or CTF closure",
    }
}

fn build_failed_7hell_report(target_display: String, diagnostic: SevenHellDiagnostic) -> SevenHellReport {
    let failure_on_type = diagnostic.stage == "type";
    let mut diagnostics = Vec::new();
    let diag_id = diagnostic.id.clone();
    diagnostics.push(diagnostic);
    SevenHellReport {
        target_display: target_display.clone(),
        target_normalized: target_display,
        result: SevenHellResult::Fail,
        stages: [
            stage_report(
                1,
                "Syntax Hell",
                "syntax",
                if failure_on_type {
                    SevenHellStageStatus::Pass
                } else {
                    SevenHellStageStatus::Fail
                },
                if failure_on_type {
                    "single-file check accepted".to_string()
                } else {
                    failure_stage_summary(&diagnostics[0])
                },
                None,
                if failure_on_type { vec![] } else { vec![diag_id.clone()] },
            ),
            stage_report(
                2,
                "Type Hell",
                "type",
                if failure_on_type {
                    SevenHellStageStatus::Fail
                } else {
                    SevenHellStageStatus::Blocked
                },
                if failure_on_type {
                    failure_stage_summary(&diagnostics[0])
                } else {
                    "blocked by syntax check failure".to_string()
                },
                if failure_on_type { None } else { Some("syntax") },
                if failure_on_type { vec![diag_id.clone()] } else { vec![] },
            ),
            stage_report(
                3,
                "Lowering Hell",
                "lowering",
                SevenHellStageStatus::Blocked,
                "blocked by earlier stage failure",
                Some("type"),
                vec![],
            ),
            stage_report(
                4,
                "Verifier Hell",
                "verifier",
                SevenHellStageStatus::Blocked,
                "blocked by earlier stage failure",
                Some("lowering"),
                vec![],
            ),
            stage_report(
                5,
                "VM Hell",
                "vm",
                SevenHellStageStatus::Blocked,
                "blocked by earlier stage failure",
                Some("verifier"),
                vec![],
            ),
            stage_report(
                6,
                "Practical Hell",
                "practical",
                SevenHellStageStatus::Blocked,
                "blocked by earlier stage failure",
                Some("vm"),
                vec![],
            ),
            stage_report(
                7,
                "User Pain / Diagnostics Hell",
                "diagnostics",
                SevenHellStageStatus::NotImplemented,
                "diagnostic wiring reserved for 7HELL-S3",
                None,
                vec![],
            ),
        ],
        diagnostics,
        boundary: "S2 single-file check only; no compile, verify, VM run, project-root, CI gate, release readiness, or CTF closure",
    }
}

fn stage_report(
    index: usize,
    name: &'static str,
    key: &'static str,
    status: SevenHellStageStatus,
    summary: impl Into<String>,
    blocked_by: Option<&'static str>,
    diagnostic_ids: Vec<String>,
) -> SevenHellStageReport {
    SevenHellStageReport {
        index,
        name,
        key,
        status,
        summary: summary.into(),
        blocked_by,
        diagnostic_ids,
    }
}

fn failure_stage_summary(diagnostic: &SevenHellDiagnostic) -> String {
    format!(
        "code: {}; category: {}; summary: {}",
        diagnostic.code, diagnostic.category, diagnostic.message_needle
    )
}

fn render_7hell_report(report: &SevenHellReport, output_mode: SevenHellOutputMode) -> String {
    match output_mode {
        SevenHellOutputMode::Human => render_human_7hell_report(report),
        SevenHellOutputMode::Json => render_json_7hell_report(report),
    }
}

fn render_human_7hell_report(report: &SevenHellReport) -> String {
    let mut out = String::new();
    out.push_str("Semantic 7hell qualification\n");
    out.push_str(&format!("target: {}\n", report.target_display));
    out.push_str("mode: single-file\n");
    out.push_str("profile: default\n\n");
    for stage in &report.stages {
        out.push_str(&format!(
            "[{}/7] {:<29} {}\n",
            stage.index,
            stage.name,
            stage.status.as_human()
        ));
        if let Some(blocked_by) = stage.blocked_by {
            out.push_str(&format!("  blocked_by: {}\n", blocked_by));
        }
        out.push_str(&format!("  summary: {}\n", stage.summary));
        if !stage.diagnostic_ids.is_empty() {
            out.push_str(&format!(
                "  diagnostic_ids: {}\n",
                stage.diagnostic_ids.join(", ")
            ));
        }
    }
    if !report.diagnostics.is_empty() {
        out.push_str("\ndiagnostics:\n");
        for diagnostic in &report.diagnostics {
            out.push_str(&format!(
                "  - {} stage={} kind={} code={} category={} summary={} source={}{}\n",
                diagnostic.id,
                diagnostic.stage,
                diagnostic.kind.as_human(),
                diagnostic.code,
                diagnostic.category,
                diagnostic.message_needle,
                diagnostic.source.file,
                match (diagnostic.source.line, diagnostic.source.column) {
                    (Some(line), Some(col)) => format!(":{}:{}", line, col),
                    _ => String::new(),
                }
            ));
        }
    }
    out.push_str(&format!("\nresult: {}\n", report.result.as_human()));
    out.push_str(&format!("boundary: {}\n", report.boundary));
    out
}

fn render_json_7hell_report(report: &SevenHellReport) -> String {
    let mut stages = String::new();
    for (idx, stage) in report.stages.iter().enumerate() {
        if idx > 0 {
            stages.push_str(",\n");
        }
        let blocked_by = match stage.blocked_by {
            Some(value) => format!("\"{}\"", json_escape(value)),
            None => "null".to_string(),
        };
        let diagnostic_ids = render_json_string_array(&stage.diagnostic_ids);
        stages.push_str(&format!(
            "    {{\n      \"index\": {},\n      \"name\": \"{}\",\n      \"key\": \"{}\",\n      \"status\": \"{}\",\n      \"summary\": \"{}\",\n      \"diagnostic_ids\": {},\n      \"evidence_ids\": [],\n      \"blocked_by\": {}\n    }}",
            stage.index,
            json_escape(stage.name),
            json_escape(stage.key),
            stage.status.as_json(),
            json_escape(&stage.summary),
            diagnostic_ids,
            blocked_by
        ));
    }

    let diagnostics = render_json_diagnostics(&report.diagnostics);

    format!(
        "{{\n  \"schema\": \"semantic.7hell.report.v0\",\n  \"tool\": \"smc 7hell\",\n  \"target\": {{\n    \"kind\": \"single-file\",\n    \"display\": \"{}\",\n    \"normalized\": \"{}\"\n  }},\n  \"profile\": \"default\",\n  \"result\": \"{}\",\n  \"stages\": [\n{}\n  ],\n  \"diagnostics\": {},\n  \"evidence\": [],\n  \"ctf\": [],\n  \"boundaries\": [\n    {{\n      \"id\": \"B001\",\n      \"scope\": \"7hell-s2-single-file\",\n      \"status\": \"s2-single-file-check-only\",\n      \"reason\": \"{}\"\n    }}\n  ]\n}}\n",
        json_escape(&report.target_display),
        json_escape(&report.target_normalized),
        report.result.as_json(),
        stages,
        diagnostics,
        json_escape(report.boundary)
    )
}

fn render_json_string_array(values: &[String]) -> String {
    let mut out = String::from("[");
    for (idx, value) in values.iter().enumerate() {
        if idx > 0 {
            out.push_str(", ");
        }
        out.push('"');
        out.push_str(&json_escape(value));
        out.push('"');
    }
    out.push(']');
    out
}

fn render_json_diagnostics(diagnostics: &[SevenHellDiagnostic]) -> String {
    if diagnostics.is_empty() {
        return "[]".to_string();
    }
    let mut out = String::from("[\n");
    for (idx, diagnostic) in diagnostics.iter().enumerate() {
        if idx > 0 {
            out.push_str(",\n");
        }
        let line = match diagnostic.source.line {
            Some(line) => line.to_string(),
            None => "null".to_string(),
        };
        let column = match diagnostic.source.column {
            Some(column) => column.to_string(),
            None => "null".to_string(),
        };
        out.push_str(&format!(
            "    {{\n      \"id\": \"{}\",\n      \"stage\": \"{}\",\n      \"kind\": \"{}\",\n      \"code\": \"{}\",\n      \"category\": \"{}\",\n      \"message_needle\": \"{}\",\n      \"severity\": \"{}\",\n      \"source\": {{\n        \"file\": \"{}\",\n        \"line\": {},\n        \"column\": {}\n      }}\n    }}",
            json_escape(&diagnostic.id),
            json_escape(diagnostic.stage),
            diagnostic.kind.as_json(),
            json_escape(&diagnostic.code),
            json_escape(diagnostic.category),
            json_escape(&diagnostic.message_needle),
            json_escape(diagnostic.severity),
            json_escape(&diagnostic.source.file),
            line,
            column
        ));
    }
    out.push_str("\n  ]");
    out
}

fn diagnostic_from_check_error(error_text: &str, target_display: &str) -> SevenHellDiagnostic {
    let first_line = error_text.lines().next().unwrap_or(error_text);
    let code = extract_error_code(first_line).unwrap_or_else(|| "unknown".to_string());
    let message_needle = extract_error_message(first_line).unwrap_or_else(|| "check failed".to_string());
    let (line, column) = extract_error_location(first_line);

    let (stage, kind, category) = match code.as_str() {
        "E0201" => ("type", SevenHellDiagnosticKind::CheckDiagnostic, "type"),
        "E0000" => ("syntax", SevenHellDiagnosticKind::SyntaxDiagnostic, "syntax"),
        c if c.starts_with('E') => ("syntax", SevenHellDiagnosticKind::SyntaxDiagnostic, "syntax"),
        _ => ("syntax", SevenHellDiagnosticKind::CheckDiagnostic, "check"),
    };

    SevenHellDiagnostic {
        id: "D001".to_string(),
        stage,
        kind,
        code,
        category,
        message_needle,
        severity: "error",
        source: SevenHellDiagnosticSource {
            file: target_display.to_string(),
            line,
            column,
        },
    }
}

fn boundary_denial_diagnostic(target_display: &str) -> SevenHellDiagnostic {
    SevenHellDiagnostic {
        id: "D001".to_string(),
        stage: "syntax",
        kind: SevenHellDiagnosticKind::BoundaryDenial,
        code: "E0239".to_string(),
        category: "boundary",
        message_needle: "input file could not be read".to_string(),
        severity: "error",
        source: SevenHellDiagnosticSource {
            file: target_display.to_string(),
            line: None,
            column: None,
        },
    }
}

fn extract_error_code(line: &str) -> Option<String> {
    let start = line.find('[')? + 1;
    let end = line[start..].find(']')? + start;
    Some(line[start..end].to_string())
}

fn extract_error_message(line: &str) -> Option<String> {
    let after_colon = line.find("]: ")? + 3;
    let tail = &line[after_colon..];
    if let Some(pos) = tail.rfind(" at line ") {
        Some(tail[..pos].trim().to_string())
    } else {
        Some(tail.trim().to_string())
    }
}

fn extract_error_location(line: &str) -> (Option<u32>, Option<u32>) {
    let Some(pos) = line.rfind(" at line ") else {
        return (None, None);
    };
    let loc = &line[(pos + " at line ".len())..];
    let mut split = loc.split(':');
    let line = split.next().and_then(|v| v.parse::<u32>().ok());
    let column = split.next().and_then(|v| v.parse::<u32>().ok());
    (line, column)
}

fn json_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn mk_temp_dir(prefix: &str) -> std::path::PathBuf {
        let base = std::env::temp_dir().join(format!(
            "{}_{}_{}",
            prefix,
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&base).expect("mkdir");
        base
    }

    #[test]
    fn parses_7hell_target() {
        let args = vec!["program.sm".to_string()];
        let (target, output_mode) = parse_7hell_args(&args).expect("parse");
        assert_eq!(target, "program.sm");
        assert!(matches!(output_mode, SevenHellOutputMode::Human));
    }

    #[test]
    fn rejects_project_flag() {
        let args = vec!["--project".to_string(), ".".to_string()];
        let err = parse_7hell_args(&args).expect_err("reject project");
        assert!(err.contains("unknown flag"));
    }

    #[test]
    fn renders_incomplete_human_s2_pass() {
        let report = build_passed_7hell_report("program.sm".to_string());
        let rendered = render_7hell_report(&report, SevenHellOutputMode::Human);
        assert!(rendered.contains("Semantic 7hell qualification"));
        assert!(rendered.contains("[1/7] Syntax Hell"));
        assert!(rendered.contains("[2/7] Type Hell"));
        assert!(rendered.contains("PASS"));
        assert!(rendered.contains("NOT_IMPLEMENTED"));
        assert!(rendered.contains("result: INCOMPLETE"));
        assert!(rendered.contains(
            "S2 single-file check only; no compile, verify, VM run, project-root, CI gate, release readiness, or CTF closure"
        ));
        assert!(!rendered.contains("result: PASS"));
    }

    #[test]
    fn renders_json_s2_pass_schema() {
        let report = build_passed_7hell_report("program.sm".to_string());
        let rendered = render_7hell_report(&report, SevenHellOutputMode::Json);
        assert!(rendered.contains("\"schema\": \"semantic.7hell.report.v0\""));
        assert!(rendered.contains("\"result\": \"incomplete\""));
        assert!(rendered.contains("\"key\": \"syntax\""));
        assert!(rendered.contains("\"key\": \"diagnostics\""));
        assert!(rendered.contains("\"status\": \"pass\""));
        assert!(rendered.contains("\"blocked_by\": null"));
        assert!(rendered.contains("\"scope\": \"7hell-s2-single-file\""));
        assert!(rendered.contains("\"diagnostics\": []"));
    }

    #[test]
    fn executes_single_file_check_and_reports_pass() {
        let dir = mk_temp_dir("smc_7hell_s2_pass");
        let entry = dir.join("program.sm");
        std::fs::write(
            &entry,
            r#"
fn main() {
    return;
}
"#,
        )
        .expect("write source");

        let outcome = execute_7hell_single_file(&entry.to_string_lossy(), SevenHellOutputMode::Human);
        assert!(outcome.success);
        assert!(outcome.rendered.contains("Syntax Hell"));
        assert!(outcome.rendered.contains("PASS"));
        assert!(outcome.rendered.contains("NOT_IMPLEMENTED"));
        assert!(outcome.rendered.contains("result: INCOMPLETE"));
        assert!(!outcome.rendered.contains("result: PASS"));
        assert!(outcome.rendered.contains(
            "boundary: S2 single-file check only; no compile, verify, VM run, project-root, CI gate, release readiness, or CTF closure"
        ));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn executes_single_file_check_and_reports_fail() {
        let dir = mk_temp_dir("smc_7hell_s2_fail");
        let entry = dir.join("program.sm");
        std::fs::write(&entry, "fn main(").expect("write source");

        let outcome = execute_7hell_single_file(&entry.to_string_lossy(), SevenHellOutputMode::Json);
        assert!(!outcome.success);
        assert!(outcome.rendered.contains("\"result\": \"fail\""));
        assert!(outcome.rendered.contains("\"status\": \"fail\""));
        assert!(outcome.rendered.contains("\"status\": \"blocked\""));
        assert!(outcome.rendered.contains("\"diagnostics\": ["));
        assert!(outcome.rendered.contains("\"id\": \"D001\""));
        assert!(outcome.rendered.contains("\"kind\": \"syntax-diagnostic\""));
        assert!(outcome.rendered.contains("\"category\": \"syntax\""));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn syntax_failure_includes_stable_diagnostic_summary() {
        let dir = mk_temp_dir("smc_7hell_s2_display");
        let entry = dir.join("program.sm");
        std::fs::write(&entry, "fn main(").expect("write source");

        let outcome = execute_7hell_single_file(&entry.to_string_lossy(), SevenHellOutputMode::Human);
        assert!(outcome.rendered.contains("Syntax Hell"));
        assert!(outcome.rendered.contains("FAIL"));
        assert!(outcome.rendered.contains("code: E0000"));
        assert!(outcome.rendered.contains("category: syntax"));
        assert!(outcome.rendered.contains("summary:"));
        assert!(!outcome.rendered.contains(&*entry.to_string_lossy()));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn type_failure_populates_check_diagnostic() {
        let outcome = execute_7hell_single_file(
            "tests/fixtures/pcc3_text/negative_to_text_record.sm",
            SevenHellOutputMode::Json,
        );
        assert!(!outcome.success);
        assert!(outcome.rendered.contains("\"stage\": \"type\""));
        assert!(outcome.rendered.contains("\"kind\": \"check-diagnostic\""));
        assert!(outcome.rendered.contains("\"code\": \"E0201\""));
        assert!(outcome.rendered.contains("\"category\": \"type\""));
        assert!(outcome.rendered.contains("\"result\": \"fail\""));
        assert!(outcome.rendered.contains("\"diagnostics\": ["));
    }

    #[test]
    fn display_path_never_reveals_absolute_temp_path() {
        let dir = mk_temp_dir("smc_7hell_s2_display");
        let entry = dir.join("program.sm");
        std::fs::write(&entry, "fn main() { return; }").expect("write source");

        let display = display_path_for_report(&entry.to_string_lossy());
        assert!(!display.contains(':'));
        assert!(!display.starts_with('/'));
        assert!(!display.starts_with('\\'));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
