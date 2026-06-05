use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use sm_emit::{compile_program_to_semcode_with_options_debug, CompileProfile, OptLevel};
use sm_front::FrontendError;
use sm_verify::{RejectReport, VerifiedProgram};
use sm_vm::RuntimeError as VmRuntimeError;
use smc_cli::{CliPipeline, ControlledObservationQualificationEnvelope};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SevenHellOutputMode {
    Human,
    Json,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SevenHellDiagnosticKind {
    SyntaxDiagnostic,
    CheckDiagnostic,
    LoweringDiagnostic,
    VerifierRejection,
    VmTrap,
    VmError,
    PracticalDiagnostic,
    BoundaryDenial,
}

impl SevenHellDiagnosticKind {
    fn as_human(self) -> &'static str {
        match self {
            Self::SyntaxDiagnostic => "syntax-diagnostic",
            Self::CheckDiagnostic => "check-diagnostic",
            Self::LoweringDiagnostic => "lowering-diagnostic",
            Self::VerifierRejection => "verifier-rejection",
            Self::VmTrap => "vm-trap",
            Self::VmError => "vm-error",
            Self::PracticalDiagnostic => "practical-diagnostic",
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
    boundary_status: &'static str,
    boundary: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DiagnosticsHellStatus {
    Pass,
    Fail,
    Incomplete,
}

struct DiagnosticsHellFinding {
    status: DiagnosticsHellStatus,
    summary: String,
    diagnostic_ids: Vec<String>,
}

struct SevenHellRenderOutput {
    rendered: String,
    success: bool,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if matches!(
        args.first().map(|s| s.as_str()),
        Some("7hell" | "seven-hell")
    ) {
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

fn execute_7hell_single_file(
    target: &str,
    output_mode: SevenHellOutputMode,
) -> SevenHellRenderOutput {
    let target_display = display_path_for_report(target);
    let report = match fs::read_to_string(Path::new(target)) {
        Ok(source) => {
            match smc_cli::CliPipeline::semantic_check_source(&source) {
                Ok(_) => {
                    match compile_program_to_semcode_with_options_debug(
                        &source,
                        CompileProfile::Auto,
                        OptLevel::O0,
                        false,
                    ) {
                        Ok(bytes) => {
                            match sm_verify::verify_semcode_token(&bytes) {
                                Ok(token) => {
                                    let verified = token.program().clone();
                                    let entry_result =
                                        token.require_entry("main").map_err(|e| match e {
                                            sm_verify::EntryResolutionError::MissingEntry {
                                                entry,
                                            } => VmRuntimeError::UnknownFunction(entry),
                                        });
                                    match entry_result {
                            Ok(entry_token) => match sm_vm::run_verified_entry_semcode(&entry_token) {
                                Ok(()) => match CliPipeline::qualify_controlled_observation_bytes(&bytes) {
                                    Ok(practical) => build_practical_passed_7hell_report(
                                        target_display,
                                        verified,
                                        practical,
                                    ),
                                    Err(error) => build_practical_failed_7hell_report(
                                        target_display.clone(),
                                        diagnostic_from_practical_error(&error, &target_display),
                                    ),
                                },
                                Err(error) => build_vm_failed_7hell_report(
                                    target_display.clone(),
                                    diagnostic_from_vm_error(&error, &target_display),
                                ),
                            },
                            Err(error) => build_vm_failed_7hell_report(
                                target_display.clone(),
                                diagnostic_from_vm_error(&error, &target_display),
                            ),
                        }
                                }
                                Err(report) => build_verifier_failed_7hell_report(
                                    target_display.clone(),
                                    diagnostic_from_verifier_error(&report, &target_display),
                                ),
                            }
                        }
                        Err(error) => build_lowering_failed_7hell_report(
                            target_display.clone(),
                            diagnostic_from_compile_error(&error, &target_display),
                        ),
                    }
                }
                Err(error_text) => {
                    let diagnostic = diagnostic_from_check_error(&error_text, &target_display);
                    build_check_failed_7hell_report(target_display, diagnostic)
                }
            }
        }
        Err(_) => build_check_failed_7hell_report(
            target_display.clone(),
            boundary_denial_diagnostic(&target_display),
        ),
    };
    let report = apply_diagnostics_hell_report_quality(report);
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

#[cfg(test)]
fn build_verified_7hell_report(
    target_display: String,
    verified: VerifiedProgram,
) -> SevenHellReport {
    build_practical_passed_7hell_report(
        target_display,
        verified,
        ControlledObservationQualificationEnvelope {
            capability_decision:
                prom_cap::hello_observation_capability::HelloObservationCapabilityDecision::Allow,
            audit_results: Vec::new(),
            observations: Vec::new(),
        },
    )
}

#[allow(dead_code)]
fn build_vm_passed_7hell_report(
    target_display: String,
    verified: VerifiedProgram,
) -> SevenHellReport {
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
                SevenHellStageStatus::Pass,
                "SemCode emitted for verifier admission",
                None,
                vec![],
            ),
            stage_report(
                4,
                "Verifier Hell",
                "verifier",
                SevenHellStageStatus::Pass,
                format!(
                    "SemCode verifier accepted program ({} function(s))",
                    verified.functions.len()
                ),
                None,
                vec![],
            ),
            stage_report(
                5,
                "VM Hell",
                "vm",
                SevenHellStageStatus::Pass,
                "VM executed verified SemCode successfully",
                None,
                vec![],
            ),
            stage_report(
                6,
                "Practical Hell",
                "practical",
                SevenHellStageStatus::Blocked,
                "host-visible effects and practical qualification are outside 7HELL-S7",
                Some("vm"),
                vec![],
            ),
            stage_report(
                7,
                "User Pain / Diagnostics Hell",
                "diagnostics",
                SevenHellStageStatus::NotImplemented,
                "7HELL-S7 does not wire diagnostics",
                None,
                vec![],
            ),
        ],
        diagnostics: Vec::new(),
        boundary_status: "s7-practical-qualification-only",
        boundary: "S7 non-rendering Practical qualification only; no raw observation text, host-visible output, project-root, cache route, temp .smc, CI gate, release readiness, or CTF closure",
    }
}

fn build_practical_passed_7hell_report(
    target_display: String,
    verified: VerifiedProgram,
    practical: ControlledObservationQualificationEnvelope,
) -> SevenHellReport {
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
                SevenHellStageStatus::Pass,
                "SemCode emitted for verifier admission",
                None,
                vec![],
            ),
            stage_report(
                4,
                "Verifier Hell",
                "verifier",
                SevenHellStageStatus::Pass,
                format!(
                    "SemCode verifier accepted program ({} function(s))",
                    verified.functions.len()
                ),
                None,
                vec![],
            ),
            stage_report(
                5,
                "VM Hell",
                "vm",
                SevenHellStageStatus::Pass,
                "VM executed verified SemCode successfully",
                None,
                vec![],
            ),
            stage_report(
                6,
                "Practical Hell",
                "practical",
                SevenHellStageStatus::Pass,
                format!(
                    "Practical qualification completed without host-visible rendering ({} observation(s), {} audit record(s))",
                    practical.observations.len(),
                    practical.audit_results.len()
                ),
                None,
                vec![],
            ),
            stage_report(
                7,
                "User Pain / Diagnostics Hell",
                "diagnostics",
                SevenHellStageStatus::NotImplemented,
                "7HELL-S7 does not wire diagnostics",
                None,
                vec![],
            ),
        ],
        diagnostics: Vec::new(),
        boundary_status: "s7-practical-qualification-only",
        boundary: "S7 non-rendering Practical qualification only; no raw observation text, host-visible output, project-root, cache route, temp .smc, CI gate, release readiness, or CTF closure",
    }
}

fn build_practical_failed_7hell_report(
    target_display: String,
    diagnostic: SevenHellDiagnostic,
) -> SevenHellReport {
    let diag_id = diagnostic.id.clone();
    SevenHellReport {
        target_display: target_display.clone(),
        target_normalized: target_display,
        result: SevenHellResult::Fail,
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
                SevenHellStageStatus::Pass,
                "SemCode emitted for verifier admission",
                None,
                vec![],
            ),
            stage_report(
                4,
                "Verifier Hell",
                "verifier",
                SevenHellStageStatus::Pass,
                "SemCode verifier accepted program",
                None,
                vec![],
            ),
            stage_report(
                5,
                "VM Hell",
                "vm",
                SevenHellStageStatus::Pass,
                "VM executed verified SemCode successfully",
                None,
                vec![],
            ),
            stage_report(
                6,
                "Practical Hell",
                "practical",
                SevenHellStageStatus::Fail,
                failure_stage_summary(&diagnostic),
                None,
                vec![diag_id.clone()],
            ),
            stage_report(
                7,
                "User Pain / Diagnostics Hell",
                "diagnostics",
                SevenHellStageStatus::NotImplemented,
                "7HELL-S7 does not wire diagnostics",
                None,
                vec![],
            ),
        ],
        diagnostics: vec![diagnostic],
        boundary_status: "s7-practical-qualification-only",
        boundary: "S7 non-rendering Practical qualification only; no raw observation text, host-visible output, project-root, cache route, temp .smc, CI gate, release readiness, or CTF closure",
    }
}

fn build_lowering_failed_7hell_report(
    target_display: String,
    diagnostic: SevenHellDiagnostic,
) -> SevenHellReport {
    let diag_id = diagnostic.id.clone();
    SevenHellReport {
        target_display: target_display.clone(),
        target_normalized: target_display,
        result: SevenHellResult::Fail,
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
                SevenHellStageStatus::Fail,
                failure_stage_summary(&diagnostic),
                None,
                vec![diag_id.clone()],
            ),
            stage_report(
                4,
                "Verifier Hell",
                "verifier",
                SevenHellStageStatus::Blocked,
                "blocked by lowering stage failure",
                Some("lowering"),
                vec![],
            ),
            stage_report(
                5,
                "VM Hell",
                "vm",
                SevenHellStageStatus::Blocked,
                "blocked by verifier stage failure",
                Some("verifier"),
                vec![],
            ),
            stage_report(
                6,
                "Practical Hell",
                "practical",
                SevenHellStageStatus::Blocked,
                "blocked by VM stage failure",
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
        diagnostics: vec![diagnostic],
        boundary_status: "s7-practical-qualification-only",
        boundary: "S7 non-rendering Practical qualification only; no raw observation text, host-visible output, project-root, cache route, temp .smc, CI gate, release readiness, or CTF closure",
    }
}

fn build_verifier_failed_7hell_report(
    target_display: String,
    diagnostic: SevenHellDiagnostic,
) -> SevenHellReport {
    let diag_id = diagnostic.id.clone();
    SevenHellReport {
        target_display: target_display.clone(),
        target_normalized: target_display,
        result: SevenHellResult::Fail,
        stages: [
            stage_report(
                1,
                "Syntax Hell",
                "syntax",
                SevenHellStageStatus::Pass,
                "single-file check accepted".to_string(),
                None,
                vec![],
            ),
            stage_report(
                2,
                "Type Hell",
                "type",
                SevenHellStageStatus::Pass,
                "single-file check accepted".to_string(),
                None,
                vec![],
            ),
            stage_report(
                3,
                "Lowering Hell",
                "lowering",
                SevenHellStageStatus::Pass,
                "SemCode emitted for verifier admission",
                None,
                vec![],
            ),
            stage_report(
                4,
                "Verifier Hell",
                "verifier",
                SevenHellStageStatus::Fail,
                failure_stage_summary(&diagnostic),
                None,
                vec![diag_id.clone()],
            ),
            stage_report(
                5,
                "VM Hell",
                "vm",
                SevenHellStageStatus::Blocked,
                "VM execution intentionally not implemented in 7HELL-S5",
                Some("vm"),
                vec![],
            ),
            stage_report(
                6,
                "Practical Hell",
                "practical",
                SevenHellStageStatus::Blocked,
                "blocked until VM execution is available",
                Some("vm"),
                vec![],
            ),
            stage_report(
                7,
                "User Pain / Diagnostics Hell",
                "diagnostics",
                SevenHellStageStatus::NotImplemented,
                "7HELL-S7 does not wire diagnostics",
                None,
                vec![],
            ),
        ],
        diagnostics: vec![diagnostic],
        boundary_status: "s7-practical-qualification-only",
        boundary: "S7 non-rendering Practical qualification only; no raw observation text, host-visible output, project-root, cache route, temp .smc, CI gate, release readiness, or CTF closure",
    }
}

fn build_vm_failed_7hell_report(
    target_display: String,
    diagnostic: SevenHellDiagnostic,
) -> SevenHellReport {
    let diag_id = diagnostic.id.clone();
    SevenHellReport {
        target_display: target_display.clone(),
        target_normalized: target_display,
        result: SevenHellResult::Fail,
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
                SevenHellStageStatus::Pass,
                "SemCode emitted for verifier admission",
                None,
                vec![],
            ),
            stage_report(
                4,
                "Verifier Hell",
                "verifier",
                SevenHellStageStatus::Pass,
                "SemCode verifier accepted program",
                None,
                vec![],
            ),
            stage_report(
                5,
                "VM Hell",
                "vm",
                SevenHellStageStatus::Fail,
                failure_stage_summary(&diagnostic),
                None,
                vec![diag_id.clone()],
            ),
            stage_report(
                6,
                "Practical Hell",
                "practical",
                SevenHellStageStatus::Blocked,
                "host-visible effects and practical qualification are outside 7HELL-S7",
                Some("vm"),
                vec![],
            ),
            stage_report(
                7,
                "User Pain / Diagnostics Hell",
                "diagnostics",
                SevenHellStageStatus::NotImplemented,
                "7HELL-S7 does not wire diagnostics",
                None,
                vec![],
            ),
        ],
        diagnostics: vec![diagnostic],
        boundary_status: "s7-practical-qualification-only",
        boundary: "S7 non-rendering Practical qualification only; no raw observation text, host-visible output, project-root, cache route, temp .smc, CI gate, release readiness, or CTF closure",
    }
}

fn build_check_failed_7hell_report(
    target_display: String,
    diagnostic: SevenHellDiagnostic,
) -> SevenHellReport {
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
        boundary_status: "s7-practical-qualification-only",
        boundary: "S7 non-rendering Practical qualification only; no raw observation text, host-visible output, project-root, cache route, temp .smc, CI gate, release readiness, or CTF closure",
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

fn apply_diagnostics_hell_report_quality(mut report: SevenHellReport) -> SevenHellReport {
    let finding = evaluate_7hell_report_quality(&report);
    let status = match finding.status {
        DiagnosticsHellStatus::Pass => SevenHellStageStatus::Pass,
        DiagnosticsHellStatus::Fail => SevenHellStageStatus::Fail,
        DiagnosticsHellStatus::Incomplete => SevenHellStageStatus::NotImplemented,
    };

    if matches!(finding.status, DiagnosticsHellStatus::Fail) {
        report.result = SevenHellResult::Fail;
    }

    report.stages[6] = stage_report(
        7,
        "User Pain / Diagnostics Hell",
        "diagnostics",
        status,
        finding.summary,
        None,
        finding.diagnostic_ids,
    );
    report
}

fn evaluate_7hell_report_quality(report: &SevenHellReport) -> DiagnosticsHellFinding {
    let mut failures = Vec::new();
    let mut incomplete = Vec::new();

    if report.stages[6].index != 7 || report.stages[6].key != "diagnostics" {
        incomplete.push("diagnostics stage slot is not recognizable".to_string());
    }
    if report.target_display.trim().is_empty() {
        failures.push("target display is empty".to_string());
    }
    if report.target_normalized.trim().is_empty() {
        failures.push("target normalized path is empty".to_string());
    }
    if report.boundary_status.trim().is_empty() {
        failures.push("boundary status is empty".to_string());
    }
    if report.boundary.trim().is_empty() {
        failures.push("boundary reason is empty".to_string());
    }

    for (idx, stage) in report.stages.iter().enumerate() {
        if stage.index != idx + 1 {
            failures.push(format!("stage {} has non-sequential index", idx + 1));
        }
        if stage.name.trim().is_empty() {
            failures.push(format!("stage {} name is empty", stage.index));
        }
        if stage.key.trim().is_empty() {
            failures.push(format!("stage {} key is empty", stage.index));
        }
        if stage.summary.trim().is_empty() {
            failures.push(format!("stage {} summary is empty", stage.index));
        }
        if matches!(stage.status, SevenHellStageStatus::Blocked) && stage.blocked_by.is_none() {
            failures.push(format!(
                "stage {} is blocked without blocked_by",
                stage.index
            ));
        }
        if matches!(stage.status, SevenHellStageStatus::Fail)
            && stage.diagnostic_ids.is_empty()
            && stage.summary.trim().is_empty()
        {
            failures.push(format!(
                "stage {} failed without diagnostic id or usable summary",
                stage.index
            ));
        }
        for diagnostic_id in &stage.diagnostic_ids {
            match report
                .diagnostics
                .iter()
                .find(|diagnostic| diagnostic.id == *diagnostic_id)
            {
                Some(diagnostic) if diagnostic.stage == stage.key => {}
                Some(_) => failures.push(format!(
                    "stage {} references diagnostic id {} from another stage",
                    stage.index, diagnostic_id
                )),
                None => failures.push(format!(
                    "stage {} references missing diagnostic id {}",
                    stage.index, diagnostic_id
                )),
            }
        }
        if contains_forbidden_readiness_claim(&stage.summary) {
            failures.push(format!(
                "stage {} summary contains forbidden readiness claim",
                stage.index
            ));
        }
        if contains_raw_output_leakage_indicator(&stage.summary) {
            failures.push(format!(
                "stage {} summary contains raw output leakage indicator",
                stage.index
            ));
        }
    }

    for diagnostic in &report.diagnostics {
        if diagnostic.id.trim().is_empty() {
            failures.push("diagnostic id is empty".to_string());
        }
        if diagnostic.stage.trim().is_empty() {
            failures.push(format!("diagnostic {} stage is empty", diagnostic.id));
        }
        if diagnostic.code.trim().is_empty() {
            failures.push(format!("diagnostic {} code is empty", diagnostic.id));
        }
        if diagnostic.category.trim().is_empty() {
            failures.push(format!("diagnostic {} category is empty", diagnostic.id));
        }
        if diagnostic.message_needle.trim().is_empty() {
            failures.push(format!(
                "diagnostic {} message needle is empty",
                diagnostic.id
            ));
        }
        if diagnostic.severity.trim().is_empty() {
            failures.push(format!("diagnostic {} severity is empty", diagnostic.id));
        }
        if diagnostic.source.file.trim().is_empty() {
            failures.push(format!("diagnostic {} source file is empty", diagnostic.id));
        }
        if !report
            .stages
            .iter()
            .any(|stage| stage.key == diagnostic.stage)
        {
            failures.push(format!(
                "diagnostic {} refers to unknown stage {}",
                diagnostic.id, diagnostic.stage
            ));
        }
        if report
            .diagnostics
            .iter()
            .filter(|candidate| candidate.id == diagnostic.id)
            .count()
            > 1
        {
            failures.push(format!("diagnostic id {} is duplicated", diagnostic.id));
        }
        if contains_forbidden_readiness_claim(&diagnostic.message_needle) {
            failures.push(format!(
                "diagnostic {} contains forbidden readiness claim",
                diagnostic.id
            ));
        }
        if contains_raw_output_leakage_indicator(&diagnostic.message_needle) {
            failures.push(format!(
                "diagnostic {} contains raw output leakage indicator",
                diagnostic.id
            ));
        }
    }

    if contains_forbidden_readiness_claim(report.boundary)
        || contains_forbidden_readiness_claim(report.boundary_status)
    {
        failures.push("boundary contains forbidden readiness claim".to_string());
    }
    if contains_raw_output_leakage_indicator(report.boundary)
        || contains_raw_output_leakage_indicator(report.boundary_status)
    {
        failures.push("boundary contains raw output leakage indicator".to_string());
    }

    if !failures.is_empty() {
        return DiagnosticsHellFinding {
            status: DiagnosticsHellStatus::Fail,
            summary: format!("report quality rejected: {}", failures.join("; ")),
            diagnostic_ids: vec![],
        };
    }

    if !incomplete.is_empty() {
        return DiagnosticsHellFinding {
            status: DiagnosticsHellStatus::Incomplete,
            summary: format!("report quality incomplete: {}", incomplete.join("; ")),
            diagnostic_ids: vec![],
        };
    }

    DiagnosticsHellFinding {
        status: DiagnosticsHellStatus::Pass,
        summary:
            "report quality accepted: stage/status/reason/diagnostic boundary is structurally usable"
                .to_string(),
        diagnostic_ids: vec![],
    }
}

fn contains_forbidden_readiness_claim(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    [
        "release ready",
        "release-ready",
        "release readiness claimed",
        "ctf closure claimed",
        "ctf closed",
        "ctf complete",
        "final pass",
        "\"result\": \"pass\"",
    ]
    .iter()
    .any(|needle| value.contains(needle))
}

fn contains_raw_output_leakage_indicator(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    [
        "rendered_lines",
        "raw_text",
        "stdout",
        "stderr",
        "hello, world!",
    ]
    .iter()
    .any(|needle| value.contains(needle))
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
        "{{\n  \"schema\": \"semantic.7hell.report.v0\",\n  \"tool\": \"smc 7hell\",\n  \"target\": {{\n    \"kind\": \"single-file\",\n    \"display\": \"{}\",\n    \"normalized\": \"{}\"\n  }},\n  \"profile\": \"default\",\n  \"result\": \"{}\",\n  \"stages\": [\n{}\n  ],\n  \"diagnostics\": {},\n  \"evidence\": [],\n  \"ctf\": [],\n  \"boundaries\": [\n    {{\n      \"id\": \"B001\",\n      \"scope\": \"7hell-single-file\",\n      \"status\": \"{}\",\n      \"reason\": \"{}\"\n    }}\n  ]\n}}\n",
        json_escape(&report.target_display),
        json_escape(&report.target_normalized),
        report.result.as_json(),
        stages,
        diagnostics,
        json_escape(report.boundary_status),
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
    let message_needle =
        extract_error_message(first_line).unwrap_or_else(|| "check failed".to_string());
    let (line, column) = extract_error_location(first_line);

    let (stage, kind, category) = match code.as_str() {
        "E0201" => ("type", SevenHellDiagnosticKind::CheckDiagnostic, "type"),
        "E0000" => (
            "syntax",
            SevenHellDiagnosticKind::SyntaxDiagnostic,
            "syntax",
        ),
        c if c.starts_with('E') => (
            "syntax",
            SevenHellDiagnosticKind::SyntaxDiagnostic,
            "syntax",
        ),
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

fn diagnostic_from_compile_error(
    error: &FrontendError,
    target_display: &str,
) -> SevenHellDiagnostic {
    SevenHellDiagnostic {
        id: "D001".to_string(),
        stage: "lowering",
        kind: SevenHellDiagnosticKind::LoweringDiagnostic,
        code: "E0300".to_string(),
        category: "lowering",
        message_needle: error.message.clone(),
        severity: "error",
        source: SevenHellDiagnosticSource {
            file: target_display.to_string(),
            line: None,
            column: None,
        },
    }
}

fn diagnostic_from_verifier_error(
    report: &RejectReport,
    target_display: &str,
) -> SevenHellDiagnostic {
    let diagnostic = report
        .diagnostics
        .first()
        .expect("verifier rejection diagnostic");
    SevenHellDiagnostic {
        id: "D001".to_string(),
        stage: "verifier",
        kind: SevenHellDiagnosticKind::VerifierRejection,
        code: format!("{:?}", diagnostic.code),
        category: "verifier",
        message_needle: diagnostic.message.clone(),
        severity: "error",
        source: SevenHellDiagnosticSource {
            file: target_display.to_string(),
            line: None,
            column: None,
        },
    }
}

fn diagnostic_from_vm_error(error: &VmRuntimeError, target_display: &str) -> SevenHellDiagnostic {
    match error {
        VmRuntimeError::Trap(trap) => SevenHellDiagnostic {
            id: "D001".to_string(),
            stage: "vm",
            kind: SevenHellDiagnosticKind::VmTrap,
            code: format!("{:?}", trap),
            category: "vm",
            message_needle: vm_trap_message_needle(*trap),
            severity: "error",
            source: SevenHellDiagnosticSource {
                file: target_display.to_string(),
                line: None,
                column: None,
            },
        },
        other => SevenHellDiagnostic {
            id: "D001".to_string(),
            stage: "vm",
            kind: SevenHellDiagnosticKind::VmError,
            code: vm_error_code(other),
            category: "vm",
            message_needle: other.to_string(),
            severity: "error",
            source: SevenHellDiagnosticSource {
                file: target_display.to_string(),
                line: None,
                column: None,
            },
        },
    }
}

fn diagnostic_from_practical_error(error_text: &str, target_display: &str) -> SevenHellDiagnostic {
    let first_line = error_text.lines().next().unwrap_or(error_text);
    SevenHellDiagnostic {
        id: "D001".to_string(),
        stage: "practical",
        kind: SevenHellDiagnosticKind::PracticalDiagnostic,
        code: "PracticalQualificationFailed".to_string(),
        category: "practical",
        message_needle: extract_error_message(first_line)
            .unwrap_or_else(|| first_line.trim().to_string()),
        severity: "error",
        source: SevenHellDiagnosticSource {
            file: target_display.to_string(),
            line: None,
            column: None,
        },
    }
}

fn vm_trap_message_needle(trap: sm_runtime_core::RuntimeTrap) -> String {
    match trap {
        sm_runtime_core::RuntimeTrap::AssertionFailed => "assertion failed".to_string(),
        sm_runtime_core::RuntimeTrap::BorrowWriteConflict => {
            "write path overlaps active borrow".to_string()
        }
        sm_runtime_core::RuntimeTrap::StackOverflow => "stack overflow".to_string(),
        sm_runtime_core::RuntimeTrap::StackUnderflow => "stack underflow".to_string(),
        sm_runtime_core::RuntimeTrap::TypeMismatch => "runtime type mismatch".to_string(),
        sm_runtime_core::RuntimeTrap::InvalidOpcode => "invalid opcode".to_string(),
        sm_runtime_core::RuntimeTrap::InvalidJump => "invalid jump".to_string(),
        sm_runtime_core::RuntimeTrap::DivisionByZero => "division by zero".to_string(),
        sm_runtime_core::RuntimeTrap::ArithmeticOverflow => "arithmetic overflow".to_string(),
        sm_runtime_core::RuntimeTrap::CapabilityDenied => "capability denied".to_string(),
        sm_runtime_core::RuntimeTrap::AbiViolation => "abi violation".to_string(),
        sm_runtime_core::RuntimeTrap::VerifierRejected => "verifier rejected".to_string(),
        sm_runtime_core::RuntimeTrap::QuotaExceeded(_) => "quota exceeded".to_string(),
    }
}

fn vm_error_code(error: &VmRuntimeError) -> String {
    match error {
        VmRuntimeError::BadHeader => "BadHeader".to_string(),
        VmRuntimeError::UnsupportedBytecodeVersion { .. } => {
            "UnsupportedBytecodeVersion".to_string()
        }
        VmRuntimeError::BadFormat(_) => "BadFormat".to_string(),
        VmRuntimeError::UnknownFunction(_) => "UnknownFunction".to_string(),
        VmRuntimeError::InvalidJumpAddress { .. } => "InvalidJumpAddress".to_string(),
        VmRuntimeError::TypeMismatchRuntime(_) => "TypeMismatchRuntime".to_string(),
        VmRuntimeError::StackUnderflow => "StackUnderflow".to_string(),
        VmRuntimeError::StackOverflow => "StackOverflow".to_string(),
        VmRuntimeError::QuotaExceeded(_) => "QuotaExceeded".to_string(),
        VmRuntimeError::VerifierRejected(_) => "VerifierRejected".to_string(),
        VmRuntimeError::UnknownVariable(_) => "UnknownVariable".to_string(),
        VmRuntimeError::InvalidStringId(_) => "InvalidStringId".to_string(),
        VmRuntimeError::HostAbi(_) => "HostAbi".to_string(),
        VmRuntimeError::CapabilityDenied(_) => "CapabilityDenied".to_string(),
        VmRuntimeError::UiCapabilityDenied(_) => "UiCapabilityDenied".to_string(),
        VmRuntimeError::Trap(trap) => format!("{:?}", trap),
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

    fn verified_program() -> VerifiedProgram {
        VerifiedProgram {
            header: sm_emit::SemcodeHeaderSpec {
                magic: *b"SEMC0000",
                epoch: 0,
                rev: 0,
                capabilities: 0,
            },
            functions: vec![sm_verify::VerifiedFunction {
                name: "main".to_string(),
                code_len: 0,
                string_count: 0,
                debug_symbol_count: 0,
            }],
        }
    }

    fn well_formed_7hell_report() -> SevenHellReport {
        build_verified_7hell_report("program.sm".to_string(), verified_program())
    }

    fn syntax_diagnostic() -> SevenHellDiagnostic {
        diagnostic_from_check_error("[E0000]: syntax failed at line 1:1", "program.sm")
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
    fn evaluator_passes_well_formed_report() {
        let report = well_formed_7hell_report();
        let finding = evaluate_7hell_report_quality(&report);

        assert!(matches!(finding.status, DiagnosticsHellStatus::Pass));
        assert!(finding.summary.contains("report quality accepted"));
        assert!(finding.diagnostic_ids.is_empty());
    }

    #[test]
    fn evaluator_fails_empty_stage_summary() {
        let mut report = well_formed_7hell_report();
        report.stages[0].summary.clear();

        let finding = evaluate_7hell_report_quality(&report);

        assert!(matches!(finding.status, DiagnosticsHellStatus::Fail));
        assert!(finding.summary.contains("stage 1 summary is empty"));
    }

    #[test]
    fn evaluator_fails_missing_blocked_by() {
        let mut report =
            build_check_failed_7hell_report("program.sm".to_string(), syntax_diagnostic());
        report.stages[2].blocked_by = None;

        let finding = evaluate_7hell_report_quality(&report);

        assert!(matches!(finding.status, DiagnosticsHellStatus::Fail));
        assert!(finding
            .summary
            .contains("stage 3 is blocked without blocked_by"));
    }

    #[test]
    fn evaluator_fails_missing_diagnostic_reference() {
        let mut report =
            build_check_failed_7hell_report("program.sm".to_string(), syntax_diagnostic());
        report.stages[0].diagnostic_ids = vec!["D404".to_string()];

        let finding = evaluate_7hell_report_quality(&report);

        assert!(matches!(finding.status, DiagnosticsHellStatus::Fail));
        assert!(finding
            .summary
            .contains("stage 1 references missing diagnostic id D404"));
    }

    #[test]
    fn evaluator_fails_raw_output_leakage_indicator() {
        let mut report = well_formed_7hell_report();
        report.stages[5].summary = "rendered_lines leaked through report".to_string();

        let finding = evaluate_7hell_report_quality(&report);

        assert!(matches!(finding.status, DiagnosticsHellStatus::Fail));
        assert!(finding
            .summary
            .contains("stage 6 summary contains raw output leakage indicator"));
    }

    #[test]
    fn evaluator_does_not_change_execution_result_to_pass() {
        let report = apply_diagnostics_hell_report_quality(well_formed_7hell_report());

        assert!(matches!(report.result, SevenHellResult::Incomplete));
        assert!(matches!(
            report.stages[6].status,
            SevenHellStageStatus::Pass
        ));

        let rendered = render_7hell_report(&report, SevenHellOutputMode::Json);
        assert!(rendered.contains("\"result\": \"incomplete\""));
        assert!(!rendered.contains("\"result\": \"pass\""));
    }

    #[test]
    fn renders_incomplete_human_s2_pass() {
        let report = apply_diagnostics_hell_report_quality(well_formed_7hell_report());
        let rendered = render_7hell_report(&report, SevenHellOutputMode::Human);
        assert!(rendered.contains("Semantic 7hell qualification"));
        assert!(rendered.contains("[1/7] Syntax Hell"));
        assert!(rendered.contains("[2/7] Type Hell"));
        assert!(rendered.contains("[3/7] Lowering Hell"));
        assert!(rendered.contains("[4/7] Verifier Hell"));
        assert!(rendered.contains("[5/7] VM Hell"));
        assert!(rendered.contains("[6/7] Practical Hell"));
        assert!(rendered.contains("PASS"));
        assert!(!rendered.contains("BLOCKED"));
        assert!(rendered.contains("[7/7] User Pain / Diagnostics Hell  PASS"));
        assert!(rendered.contains(
            "report quality accepted: stage/status/reason/diagnostic boundary is structurally usable"
        ));
        assert!(rendered.contains("Practical qualification completed without host-visible rendering (0 observation(s), 0 audit record(s))"));
        assert!(rendered.contains("result: INCOMPLETE"));
        assert!(rendered.contains(
            "S7 non-rendering Practical qualification only; no raw observation text, host-visible output, project-root, cache route, temp .smc, CI gate, release readiness, or CTF closure"
        ));
        assert!(!rendered.contains("result: PASS"));
    }

    #[test]
    fn renders_json_s2_pass_schema() {
        let report = apply_diagnostics_hell_report_quality(well_formed_7hell_report());
        let rendered = render_7hell_report(&report, SevenHellOutputMode::Json);
        assert!(rendered.contains("\"schema\": \"semantic.7hell.report.v0\""));
        assert!(rendered.contains("\"result\": \"incomplete\""));
        assert!(rendered.contains("\"key\": \"syntax\""));
        assert!(rendered.contains("\"key\": \"verifier\""));
        assert!(rendered.contains("\"key\": \"diagnostics\""));
        assert!(rendered.contains("\"status\": \"pass\""));
        assert!(!rendered.contains("\"status\": \"not_implemented\""));
        assert!(rendered.contains("\"scope\": \"7hell-single-file\""));
        assert!(rendered.contains("\"status\": \"s7-practical-qualification-only\""));
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

        let outcome =
            execute_7hell_single_file(&entry.to_string_lossy(), SevenHellOutputMode::Human);
        assert!(outcome.success);
        assert!(outcome.rendered.contains("Syntax Hell"));
        assert!(outcome.rendered.contains("PASS"));
        assert!(outcome.rendered.contains("[6/7] Practical Hell"));
        assert!(outcome.rendered.contains("Practical qualification completed without host-visible rendering (0 observation(s), 0 audit record(s))"));
        assert!(outcome
            .rendered
            .contains("VM executed verified SemCode successfully"));
        assert!(outcome.rendered.contains("result: INCOMPLETE"));
        assert!(!outcome.rendered.contains("result: PASS"));
        assert!(outcome.rendered.contains(
            "boundary: S7 non-rendering Practical qualification only; no raw observation text, host-visible output, project-root, cache route, temp .smc, CI gate, release readiness, or CTF closure"
        ));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn executes_single_file_check_and_reports_fail() {
        let dir = mk_temp_dir("smc_7hell_s2_fail");
        let entry = dir.join("program.sm");
        std::fs::write(&entry, "fn main(").expect("write source");

        let outcome =
            execute_7hell_single_file(&entry.to_string_lossy(), SevenHellOutputMode::Json);
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

        let outcome =
            execute_7hell_single_file(&entry.to_string_lossy(), SevenHellOutputMode::Human);
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
    fn verifier_reject_json_snapshot() {
        let target = "tests/fixtures/7hell_e1/verifier_reject.synthetic.sm".to_string();
        let report = sm_verify::RejectReport {
            diagnostics: vec![sm_verify::VerificationDiagnostic {
                code: sm_verify::VerificationCode::UnknownOpcode,
                function: Some("main".to_string()),
                offset: Some(0),
                message: "unknown opcode 0xff".to_string(),
            }],
        };
        let diagnostic = diagnostic_from_verifier_error(&report, &target);
        let seven = apply_diagnostics_hell_report_quality(build_verifier_failed_7hell_report(
            target.clone(),
            diagnostic,
        ));
        let rendered = render_7hell_report(&seven, SevenHellOutputMode::Json);

        assert!(rendered.contains("\"kind\": \"verifier-rejection\""));
        assert!(rendered.contains("\"stage\": \"verifier\""));
        assert!(rendered.contains("\"code\": \"UnknownOpcode\""));
        assert!(!rendered.contains("\"kind\": \"vm-trap\""));
        assert!(!rendered.contains("\"kind\": \"project-diagnostic\""));
        assert!(!rendered.contains("\"result\": \"pass\""));

        let snapshot = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/7hell_e1/snapshots/verifier_reject_json.json");
        let expected = std::fs::read_to_string(&snapshot)
            .unwrap_or_else(|err| panic!("read {}: {}", snapshot.display(), err));
        assert_eq!(
            expected.replace("\r\n", "\n"),
            rendered.replace("\r\n", "\n")
        );
    }

    #[test]
    fn practical_failure_json_snapshot() {
        let target = "tests/fixtures/7hell_e1/practical_failure.synthetic.sm".to_string();
        let diagnostic =
            diagnostic_from_practical_error("controlled observation capability denied", &target);
        let report = apply_diagnostics_hell_report_quality(build_practical_failed_7hell_report(
            target.clone(),
            diagnostic,
        ));
        let rendered = render_7hell_report(&report, SevenHellOutputMode::Json);

        assert!(rendered.contains("\"stage\": \"practical\""));
        assert!(rendered.contains("\"kind\": \"practical-diagnostic\""));
        assert!(rendered.contains("\"code\": \"PracticalQualificationFailed\""));
        assert!(rendered.contains("\"category\": \"practical\""));
        assert!(rendered.contains("\"result\": \"fail\""));
        assert!(rendered.contains("\"key\": \"practical\""));
        assert!(rendered.contains("\"status\": \"fail\""));
        assert!(rendered.contains("\"key\": \"vm\""));
        assert!(rendered.contains("\"status\": \"pass\""));
        assert!(rendered.contains("\"key\": \"diagnostics\""));
        assert!(rendered.contains(
            "report quality accepted: stage/status/reason/diagnostic boundary is structurally usable"
        ));
        assert!(!rendered.contains("\"kind\": \"vm-trap\""));
        assert!(!rendered.contains("\"kind\": \"verifier-rejection\""));
        assert!(!rendered.contains("\"kind\": \"project-diagnostic\""));
        assert!(!rendered.contains("rendered_lines"));
        assert!(!rendered.contains("raw_text"));
        assert!(!rendered.contains("stdout"));
        assert!(!rendered.contains("Hello, World!"));
        assert!(!rendered.contains("\"result\": \"pass\""));

        let snapshot = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/7hell_e1/snapshots/practical_failure_json.json");
        let expected = std::fs::read_to_string(&snapshot)
            .unwrap_or_else(|err| panic!("read {}: {}", snapshot.display(), err));
        assert_eq!(
            expected.replace("\r\n", "\n"),
            rendered.replace("\r\n", "\n")
        );
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
