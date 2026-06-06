use std::fs;
use std::path::PathBuf;

use semantic_language::{
    frontend::{compile_program_to_ir, compile_program_to_semcode},
    semantics::check_source,
    semcode_verify::verify_semcode,
};
use sm_verify::verify_semcode_token;
use sm_vm::run_verified_entry_semcode;
use smc_cli::parse_package_manifest_baseline;

const REPLAY_COUNT: usize = 5;
const UPDATE_ENV: &str = "SM_UPDATE_CTF_E3_TAXONOMY";

#[derive(Clone, Copy)]
struct TaxonomyCase {
    taxonomy_id: &'static str,
    pcc: &'static str,
    surface: &'static str,
    source_fixture: &'static str,
    artifact_file: &'static str,
    failure_layer: &'static str,
    expected_status: &'static str,
    stable_error_code: Option<&'static str>,
    stable_message_needle: &'static str,
    trap_class: Option<&'static str>,
    candidate_class: Option<&'static str>,
    boundary: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RunSummary {
    run_index: usize,
    failure_layer: String,
    observed_status: String,
    source_hash: String,
    ir_shape_hash: Option<String>,
    semcode_hash: Option<String>,
    stable_error_code: Option<String>,
    stable_message_needle: String,
    trap_class: Option<String>,
    candidate_class: Option<String>,
}

const TAXONOMY_CASES: &[TaxonomyCase] = &[
    TaxonomyCase {
        taxonomy_id: "CTF-E3-PCC7-SEQUENCE-OOB-VM-TRAP-001",
        pcc: "PCC-7",
        surface: "Sequence out-of-bounds",
        source_fixture: "tests/fixtures/pcc7_collections_diagnostics/negative_sequence_index_out_of_bounds.sm",
        artifact_file: "CTF-E3-PCC7-SEQUENCE-OOB-VM-TRAP-001.taxonomy.json",
        failure_layer: "vm-trap",
        expected_status: "trap",
        stable_error_code: None,
        stable_message_needle: "SEQUENCE_GET index out of bounds",
        trap_class: None,
        candidate_class: Some("sequence-index-out-of-bounds"),
        boundary: "runtime sequence trap candidate only; not a frozen VM trap class",
    },
    TaxonomyCase {
        taxonomy_id: "CTF-E3-PCC7-SEQUENCE-EMPTY-POP-VM-TRAP-001",
        pcc: "PCC-7",
        surface: "Sequence empty pop",
        source_fixture: "tests/fixtures/pcc7_collections_diagnostics/negative_sequence_pop_empty.sm",
        artifact_file: "CTF-E3-PCC7-SEQUENCE-EMPTY-POP-VM-TRAP-001.taxonomy.json",
        failure_layer: "vm-trap",
        expected_status: "trap",
        stable_error_code: None,
        stable_message_needle: "SEQUENCE_POP source must be non-empty",
        trap_class: None,
        candidate_class: Some("sequence-pop-empty"),
        boundary: "runtime sequence trap candidate only; collection quota policy remains open",
    },
    TaxonomyCase {
        taxonomy_id: "CTF-E3-PCC8-ASSERT-FALSE-VM-TRAP-001",
        pcc: "PCC-8",
        surface: "Stdlib assert(false)",
        source_fixture: "tests/fixtures/pcc8_stdlib_diagnostics/negative_assert_false_trap.sm",
        artifact_file: "CTF-E3-PCC8-ASSERT-FALSE-VM-TRAP-001.taxonomy.json",
        failure_layer: "vm-trap",
        expected_status: "trap",
        stable_error_code: None,
        stable_message_needle: "assertion failed",
        trap_class: Some("Assertion failure"),
        candidate_class: None,
        boundary: "runtime assertion failure only; no host effect or capability widening",
    },
    TaxonomyCase {
        taxonomy_id: "CTF-E3-PCC8-TO-TEXT-UNSUPPORTED-DIAGNOSTIC-001",
        pcc: "PCC-8",
        surface: "Unsupported to_text(record)",
        source_fixture: "tests/fixtures/pcc8_stdlib_diagnostics/negative_to_text_record.sm",
        artifact_file: "CTF-E3-PCC8-TO-TEXT-UNSUPPORTED-DIAGNOSTIC-001.taxonomy.json",
        failure_layer: "check-diagnostic",
        expected_status: "reject",
        stable_error_code: Some("E0201"),
        stable_message_needle: "builtin 'to_text' does not yet support record type 'Probe'",
        trap_class: None,
        candidate_class: None,
        boundary: "compile/check-time diagnostic only; not a VM trap; no universal reflection",
    },
    TaxonomyCase {
        taxonomy_id: "CTF-E3-PCC9-MANIFEST-MISSING-FIELD-DIAGNOSTIC-001",
        pcc: "PCC-9",
        surface: "Project manifest missing package directive",
        source_fixture: "tests/fixtures/pcc9_project_model_diagnostics/negative_manifest_missing_package/Semantic.package",
        artifact_file: "CTF-E3-PCC9-MANIFEST-MISSING-FIELD-DIAGNOSTIC-001.taxonomy.json",
        failure_layer: "project-diagnostic",
        expected_status: "reject",
        stable_error_code: Some("MissingPackageDirective"),
        stable_message_needle: "explicit package directive",
        trap_class: None,
        candidate_class: None,
        boundary: "project-adjacent manifest diagnostic only; not project-root execution; no semantic.toml",
    },
];

fn repo_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn read_text(rel: &str) -> String {
    let raw = fs::read_to_string(repo_path(rel))
        .unwrap_or_else(|err| panic!("read failed for {rel}: {err}"));
    normalize_newlines(&raw)
}

fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n")
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;

    let mut hash = OFFSET;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

fn hash_hex(text: &str) -> String {
    format!("{:016x}", fnv1a64(text.as_bytes()))
}

fn hash_hex_bytes(bytes: &[u8]) -> String {
    format!("{:016x}", fnv1a64(bytes))
}

fn taxonomy_dir() -> PathBuf {
    repo_path("tests/fixtures/core_trust_freeze/trap_taxonomy/ctf_e3")
}

fn manifest_path() -> PathBuf {
    taxonomy_dir().join("manifest.md")
}

fn artifact_path(case: &TaxonomyCase) -> PathBuf {
    taxonomy_dir().join(case.artifact_file)
}

fn update_mode() -> bool {
    std::env::var(UPDATE_ENV)
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn write_text(path: &PathBuf, text: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|err| panic!("create_dir_all failed for {}: {err}", parent.display()));
    }
    fs::write(path, text)
        .unwrap_or_else(|err| panic!("write failed for {}: {err}", path.display()));
}

fn assert_text_file(path: &PathBuf, got: &str) {
    let got = normalize_newlines(got);
    if update_mode() {
        write_text(path, &got);
        return;
    }
    let expected = normalize_newlines(
        &fs::read_to_string(path)
            .unwrap_or_else(|err| panic!("read failed for {}: {err}", path.display())),
    );
    assert_eq!(
        expected,
        got,
        "taxonomy artifact drifted at {}",
        path.display()
    );
}

fn render_manifest() -> String {
    let mut out = String::new();
    out.push_str("# CTF-E3 Trap Taxonomy Manifest\n");
    out.push_str("Status: checked-in taxonomy selection\n");
    out.push_str("Owner: language maturity / execution contract\n");
    out.push_str("Scope: selected PCC failure-surface taxonomy evidence only\n");
    out.push_str("Non-goal: trap-class promotion, Map policy widening, or release freeze\n\n");
    out.push_str("| taxonomy_id | pcc | surface | source_fixture | artifact | expected_status |\n");
    out.push_str("| --- | --- | --- | --- | --- | --- |\n");
    for case in TAXONOMY_CASES {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            case.taxonomy_id,
            case.pcc,
            case.surface,
            case.source_fixture,
            case.artifact_file,
            case.expected_status
        ));
    }
    out
}

fn run_vm_trap_case(case: &TaxonomyCase) -> Vec<RunSummary> {
    let mut runs = Vec::with_capacity(REPLAY_COUNT);
    for run_index in 1..=REPLAY_COUNT {
        let src = read_text(case.source_fixture);
        let _sema = check_source(&src).expect("semantic check");
        let ir = compile_program_to_ir(&src).expect("compile ir");
        let semcode = compile_program_to_semcode(&src).expect("compile semcode");
        verify_semcode(&semcode).expect("verify semcode");
        let token = verify_semcode_token(&semcode).expect("token admission");
        let entry_token = token.require_entry("main").expect("entry resolution");
        let err = run_verified_entry_semcode(&entry_token).expect_err("must trap");
        let rendered = err.to_string();
        assert!(
            rendered.contains(case.stable_message_needle),
            "expected runtime error containing '{}' for {}, got: {}",
            case.stable_message_needle,
            case.taxonomy_id,
            rendered
        );

        let source_hash = hash_hex(&src);
        let mut ir_signatures = Vec::with_capacity(ir.len());
        for func in &ir {
            ir_signatures.push(format!("{}:{}", func.name, func.instrs.len()));
        }
        ir_signatures.sort();
        let ir_shape_hash = hash_hex(&ir_signatures.join("|"));
        let semcode_hash = hash_hex_bytes(&semcode);
        runs.push(RunSummary {
            run_index,
            failure_layer: case.failure_layer.to_string(),
            observed_status: "trap".to_string(),
            source_hash,
            ir_shape_hash: Some(ir_shape_hash),
            semcode_hash: Some(semcode_hash),
            stable_error_code: None,
            stable_message_needle: case.stable_message_needle.to_string(),
            trap_class: case.trap_class.map(|value| value.to_string()),
            candidate_class: case.candidate_class.map(|value| value.to_string()),
        });
    }
    runs
}

fn run_check_diagnostic_case(case: &TaxonomyCase) -> Vec<RunSummary> {
    let mut runs = Vec::with_capacity(REPLAY_COUNT);
    for run_index in 1..=REPLAY_COUNT {
        let src = read_text(case.source_fixture);
        let err = check_source(&src).expect_err("must reject");
        let rendered = err.to_string();
        assert!(
            rendered.contains(case.stable_message_needle),
            "expected check diagnostic containing '{}' for {}, got: {}",
            case.stable_message_needle,
            case.taxonomy_id,
            rendered
        );
        if let Some(code) = case.stable_error_code {
            assert!(
                rendered.contains(code),
                "expected stable code {} for {}, got: {}",
                code,
                case.taxonomy_id,
                rendered
            );
        }

        runs.push(RunSummary {
            run_index,
            failure_layer: case.failure_layer.to_string(),
            observed_status: "reject".to_string(),
            source_hash: hash_hex(&src),
            ir_shape_hash: None,
            semcode_hash: None,
            stable_error_code: case.stable_error_code.map(|value| value.to_string()),
            stable_message_needle: case.stable_message_needle.to_string(),
            trap_class: None,
            candidate_class: None,
        });
    }
    runs
}

fn run_project_diagnostic_case(case: &TaxonomyCase) -> Vec<RunSummary> {
    let mut runs = Vec::with_capacity(REPLAY_COUNT);
    for run_index in 1..=REPLAY_COUNT {
        let source = read_text(case.source_fixture);
        let err = parse_package_manifest_baseline(&source).expect_err("must reject");
        let rendered = format!("{err:?}");
        assert!(
            rendered.contains(case.stable_message_needle),
            "expected project diagnostic containing '{}' for {}, got: {}",
            case.stable_message_needle,
            case.taxonomy_id,
            rendered
        );
        if let Some(code) = case.stable_error_code {
            assert!(
                rendered.contains(code),
                "expected stable code {} for {}, got: {}",
                code,
                case.taxonomy_id,
                rendered
            );
        }

        runs.push(RunSummary {
            run_index,
            failure_layer: case.failure_layer.to_string(),
            observed_status: "reject".to_string(),
            source_hash: hash_hex(&source),
            ir_shape_hash: None,
            semcode_hash: None,
            stable_error_code: case.stable_error_code.map(|value| value.to_string()),
            stable_message_needle: case.stable_message_needle.to_string(),
            trap_class: None,
            candidate_class: None,
        });
    }
    runs
}

fn runs_all_equal(runs: &[RunSummary]) -> bool {
    if runs.is_empty() {
        return true;
    }
    runs.iter().all(|run| {
        run.failure_layer == runs[0].failure_layer
            && run.observed_status == runs[0].observed_status
            && run.stable_error_code == runs[0].stable_error_code
            && run.stable_message_needle == runs[0].stable_message_needle
            && run.trap_class == runs[0].trap_class
            && run.candidate_class == runs[0].candidate_class
    })
}

fn render_runs(runs: &[RunSummary]) -> String {
    let mut out = String::new();
    out.push_str("[\n");
    for (idx, run) in runs.iter().enumerate() {
        out.push_str("  {\n");
        out.push_str(&format!("    \"run_index\": {},\n", run.run_index));
        out.push_str(&format!(
            "    \"failure_layer\": \"{}\",\n",
            run.failure_layer
        ));
        out.push_str(&format!(
            "    \"observed_status\": \"{}\",\n",
            run.observed_status
        ));
        out.push_str(&format!("    \"source_hash\": \"{}\",\n", run.source_hash));
        if let Some(ir_shape_hash) = &run.ir_shape_hash {
            out.push_str(&format!("    \"ir_shape_hash\": \"{}\",\n", ir_shape_hash));
        } else {
            out.push_str("    \"ir_shape_hash\": null,\n");
        }
        if let Some(semcode_hash) = &run.semcode_hash {
            out.push_str(&format!("    \"semcode_hash\": \"{}\",\n", semcode_hash));
        } else {
            out.push_str("    \"semcode_hash\": null,\n");
        }
        if let Some(code) = &run.stable_error_code {
            out.push_str(&format!("    \"stable_error_code\": \"{}\",\n", code));
        } else {
            out.push_str("    \"stable_error_code\": null,\n");
        }
        out.push_str(&format!(
            "    \"stable_message_needle\": \"{}\",\n",
            run.stable_message_needle
        ));
        if let Some(trap_class) = &run.trap_class {
            out.push_str(&format!("    \"trap_class\": \"{}\",\n", trap_class));
        } else {
            out.push_str("    \"trap_class\": null,\n");
        }
        if let Some(candidate_class) = &run.candidate_class {
            out.push_str(&format!(
                "    \"candidate_class\": \"{}\"\n",
                candidate_class
            ));
        } else {
            out.push_str("    \"candidate_class\": null\n");
        }
        out.push_str("  }");
        if idx + 1 != runs.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("]\n");
    out
}

fn render_artifact(case: &TaxonomyCase) -> String {
    let runs = match case.failure_layer {
        "vm-trap" => run_vm_trap_case(case),
        "check-diagnostic" => run_check_diagnostic_case(case),
        "project-diagnostic" => run_project_diagnostic_case(case),
        other => panic!("unsupported failure layer {other}"),
    };
    assert!(
        runs_all_equal(&runs),
        "taxonomy drifted for {}",
        case.taxonomy_id
    );
    let summary_hash = hash_hex(&render_runs(&runs));
    let stable_error_code = case
        .stable_error_code
        .map(|value| format!("\"{}\"", value))
        .unwrap_or_else(|| "null".to_string());
    let trap_class = case
        .trap_class
        .map(|value| format!("\"{}\"", value))
        .unwrap_or_else(|| "null".to_string());
    let candidate_class = case
        .candidate_class
        .map(|value| format!("\"{}\"", value))
        .unwrap_or_else(|| "null".to_string());

    format!(
        "{{\n  \"taxonomy_id\": \"{}\",\n  \"pcc\": \"{}\",\n  \"surface\": \"{}\",\n  \"source_fixture\": \"{}\",\n  \"failure_layer\": \"{}\",\n  \"expected_status\": \"{}\",\n  \"stable_error_code\": {},\n  \"stable_message_needle\": \"{}\",\n  \"trap_class\": {},\n  \"candidate_class\": {},\n  \"summary_hash\": \"{}\",\n  \"boundary\": \"{}\",\n  \"all_runs_equal\": true,\n  \"replay_count\": {},\n  \"runs\": {}\n}}\n",
        case.taxonomy_id,
        case.pcc,
        case.surface,
        case.source_fixture,
        case.failure_layer,
        case.expected_status,
        stable_error_code,
        case.stable_message_needle,
        trap_class,
        candidate_class,
        summary_hash,
        case.boundary,
        REPLAY_COUNT,
        render_runs(&runs),
    )
}

#[test]
fn ctf_e3_trap_taxonomy_regression_matches_checked_in_selection() {
    assert_text_file(&manifest_path(), &render_manifest());
    for case in TAXONOMY_CASES {
        let artifact = render_artifact(case);
        assert_text_file(&artifact_path(case), &artifact);
    }
}
