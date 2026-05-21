use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SevenHellOutputMode {
    Human,
    Json,
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
    summary: &'static str,
    blocked_by: Option<&'static str>,
}

struct SevenHellReport {
    target_display: String,
    target_normalized: String,
    result: SevenHellResult,
    stages: [SevenHellStageReport; 7],
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
            Err(_) => build_failed_7hell_report(
                target_display,
                "single-file check failed; diagnostic wiring reserved for 7HELL-S3",
            ),
        },
        Err(_) => build_failed_7hell_report(
            target_display,
            "input file could not be read; single-file check skipped",
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
            ),
            stage_report(
                2,
                "Type Hell",
                "type",
                SevenHellStageStatus::Pass,
                "single-file check accepted",
                None,
            ),
            stage_report(
                3,
                "Lowering Hell",
                "lowering",
                SevenHellStageStatus::NotImplemented,
                "7HELL-S2 does not lower",
                None,
            ),
            stage_report(
                4,
                "Verifier Hell",
                "verifier",
                SevenHellStageStatus::NotImplemented,
                "7HELL-S2 does not verify",
                None,
            ),
            stage_report(
                5,
                "VM Hell",
                "vm",
                SevenHellStageStatus::NotImplemented,
                "7HELL-S2 does not run VM",
                None,
            ),
            stage_report(
                6,
                "Practical Hell",
                "practical",
                SevenHellStageStatus::NotImplemented,
                "7HELL-S2 does not execute practical stage",
                None,
            ),
            stage_report(
                7,
                "User Pain / Diagnostics Hell",
                "diagnostics",
                SevenHellStageStatus::NotImplemented,
                "7HELL-S2 does not wire diagnostics",
                None,
            ),
        ],
        boundary: "S2 single-file check only; no compile, verify, VM run, project-root, CI gate, release readiness, or CTF closure",
    }
}

fn build_failed_7hell_report(target_display: String, summary: &'static str) -> SevenHellReport {
    SevenHellReport {
        target_display: target_display.clone(),
        target_normalized: target_display,
        result: SevenHellResult::Fail,
        stages: [
            stage_report(
                1,
                "Syntax Hell",
                "syntax",
                SevenHellStageStatus::Fail,
                summary,
                None,
            ),
            stage_report(
                2,
                "Type Hell",
                "type",
                SevenHellStageStatus::Blocked,
                "blocked by syntax check failure",
                Some("syntax"),
            ),
            stage_report(
                3,
                "Lowering Hell",
                "lowering",
                SevenHellStageStatus::Blocked,
                "blocked by earlier stage failure",
                Some("type"),
            ),
            stage_report(
                4,
                "Verifier Hell",
                "verifier",
                SevenHellStageStatus::Blocked,
                "blocked by earlier stage failure",
                Some("lowering"),
            ),
            stage_report(
                5,
                "VM Hell",
                "vm",
                SevenHellStageStatus::Blocked,
                "blocked by earlier stage failure",
                Some("verifier"),
            ),
            stage_report(
                6,
                "Practical Hell",
                "practical",
                SevenHellStageStatus::Blocked,
                "blocked by earlier stage failure",
                Some("vm"),
            ),
            stage_report(
                7,
                "User Pain / Diagnostics Hell",
                "diagnostics",
                SevenHellStageStatus::NotImplemented,
                "diagnostic wiring reserved for 7HELL-S3",
                None,
            ),
        ],
        boundary: "S2 single-file check only; no compile, verify, VM run, project-root, CI gate, release readiness, or CTF closure",
    }
}

fn stage_report(
    index: usize,
    name: &'static str,
    key: &'static str,
    status: SevenHellStageStatus,
    summary: &'static str,
    blocked_by: Option<&'static str>,
) -> SevenHellStageReport {
    SevenHellStageReport {
        index,
        name,
        key,
        status,
        summary,
        blocked_by,
    }
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
        stages.push_str(&format!(
            "    {{\n      \"index\": {},\n      \"name\": \"{}\",\n      \"key\": \"{}\",\n      \"status\": \"{}\",\n      \"summary\": \"{}\",\n      \"diagnostic_ids\": [],\n      \"evidence_ids\": [],\n      \"blocked_by\": {}\n    }}",
            stage.index,
            json_escape(stage.name),
            json_escape(stage.key),
            stage.status.as_json(),
            json_escape(stage.summary),
            blocked_by
        ));
    }

    format!(
        "{{\n  \"schema\": \"semantic.7hell.report.v0\",\n  \"tool\": \"smc 7hell\",\n  \"target\": {{\n    \"kind\": \"single-file\",\n    \"display\": \"{}\",\n    \"normalized\": \"{}\"\n  }},\n  \"profile\": \"default\",\n  \"result\": \"{}\",\n  \"stages\": [\n{}\n  ],\n  \"diagnostics\": [],\n  \"evidence\": [],\n  \"ctf\": [],\n  \"boundaries\": [\n    {{\n      \"id\": \"B001\",\n      \"scope\": \"7hell-s2-single-file\",\n      \"status\": \"s2-single-file-check-only\",\n      \"reason\": \"{}\"\n    }}\n  ]\n}}\n",
        json_escape(&report.target_display),
        json_escape(&report.target_normalized),
        report.result.as_json(),
        stages,
        json_escape(report.boundary)
    )
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
        assert!(outcome.rendered.contains(
            "single-file check failed; diagnostic wiring reserved for 7HELL-S3"
        ));

        let _ = std::fs::remove_dir_all(&dir);
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
