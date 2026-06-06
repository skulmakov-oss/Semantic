use std::fs;
use std::path::PathBuf;

use semantic_language::{
    frontend::{compile_program_to_ir, compile_program_to_semcode},
    semantics::check_source,
    semcode_verify::verify_semcode,
};
use sm_verify::verify_semcode_token;
use sm_vm::run_verified_entry_semcode;

const UPDATE_ENV: &str = "SM_UPDATE_CTF_E1_TRACES";

#[derive(Clone, Copy)]
struct TraceCase {
    trace_id: &'static str,
    pcc: &'static str,
    pcc_owner: &'static str,
    surface: &'static str,
    source_fixture: &'static str,
    artifact_file: &'static str,
    trace_class: &'static [&'static str],
    stage: &'static str,
    expected_status: &'static str,
    runtime_config: &'static str,
    notes: &'static str,
}

const TRACE_CASES: &[TraceCase] = &[
    TraceCase {
        trace_id: "CTF-E1-PCC4-RECORD-POSITIVE-001",
        pcc: "PCC-4",
        pcc_owner: "PCC-4",
        surface: "Records",
        source_fixture: "examples/qualification/g1_real_program_trial/data_audit_record_iterable/src/main.sm",
        artifact_file: "CTF-E1-PCC4-RECORD-POSITIVE-001.trace.json",
        trace_class: &["syntax", "type", "ir", "semcode", "verifier", "vm"],
        stage: "selected",
        expected_status: "accept",
        runtime_config: "default",
        notes: "Selected representative record fixture only; no project-root, Map, or 7hell coverage.",
    },
    TraceCase {
        trace_id: "CTF-E1-PCC5-ADT-MATCH-POSITIVE-001",
        pcc: "PCC-5",
        pcc_owner: "PCC-5",
        surface: "ADT + basic match",
        source_fixture: "tests/fixtures/pcc5_match/positive_match_unit_enum_label.sm",
        artifact_file: "CTF-E1-PCC5-ADT-MATCH-POSITIVE-001.trace.json",
        trace_class: &["syntax", "type", "ir", "semcode", "verifier", "vm"],
        stage: "selected",
        expected_status: "accept",
        runtime_config: "default",
        notes: "Selected basic ADT/match trace only; no broader pattern-matching freeze.",
    },
    TraceCase {
        trace_id: "CTF-E1-PCC6-OPTION-POSITIVE-001",
        pcc: "PCC-6",
        pcc_owner: "PCC-6",
        surface: "Option",
        source_fixture: "tests/fixtures/pcc6_option/positive_option_some_match.sm",
        artifact_file: "CTF-E1-PCC6-OPTION-POSITIVE-001.trace.json",
        trace_class: &["syntax", "type", "ir", "semcode", "verifier", "vm"],
        stage: "selected",
        expected_status: "accept",
        runtime_config: "default",
        notes: "Selected standard-form Option trace only; Result remains covered by later evidence if needed.",
    },
    TraceCase {
        trace_id: "CTF-E1-PCC7-SEQUENCE-POSITIVE-001",
        pcc: "PCC-7",
        pcc_owner: "PCC-7",
        surface: "Sequence",
        source_fixture: "tests/fixtures/pcc7_sequence/positive_sequence_indexing.sm",
        artifact_file: "CTF-E1-PCC7-SEQUENCE-POSITIVE-001.trace.json",
        trace_class: &["syntax", "type", "ir", "semcode", "verifier", "vm"],
        stage: "selected",
        expected_status: "accept",
        runtime_config: "default",
        notes: "Selected Sequence baseline only; Map missing-key, iteration, and quota policy remain open.",
    },
    TraceCase {
        trace_id: "CTF-E1-PCC8-STDLIB-HELPER-001",
        pcc: "PCC-8",
        pcc_owner: "PCC-8",
        surface: "Stdlib helper boundary",
        source_fixture: "tests/fixtures/pcc8_stdlib/positive_assert_true.sm",
        artifact_file: "CTF-E1-PCC8-STDLIB-HELPER-001.trace.json",
        trace_class: &["syntax", "type", "ir", "semcode", "verifier", "vm"],
        stage: "selected",
        expected_status: "accept",
        runtime_config: "default",
        notes: "Selected assert helper boundary only; no universal to_text or debug_render widening.",
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

fn trace_dir() -> PathBuf {
    repo_path("tests/fixtures/core_trust_freeze/golden_traces/ctf_e1")
}

fn trace_manifest_path() -> PathBuf {
    trace_dir().join("manifest.md")
}

fn trace_artifact_path(case: &TraceCase) -> PathBuf {
    trace_dir().join(case.artifact_file)
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
        "trace artifact drifted at {}",
        path.display()
    );
}

fn render_trace_manifest() -> String {
    let mut out = String::new();
    out.push_str("# CTF-E1 Golden Trace Manifest\n");
    out.push_str("Status: checked-in trace selection\n");
    out.push_str("Owner: language maturity / execution contract\n");
    out.push_str("Scope: selected PCC fixture-backed golden traces only\n");
    out.push_str("Non-goal: exhaustive coverage, project-root traces, or release freeze\n\n");
    out.push_str("| trace_id | pcc | surface | source_fixture | artifact | expected_status |\n");
    out.push_str("| --- | --- | --- | --- | --- | --- |\n");
    for case in TRACE_CASES {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            case.trace_id,
            case.pcc,
            case.surface,
            case.source_fixture,
            case.artifact_file,
            case.expected_status
        ));
    }
    out
}

fn render_trace_artifact(case: &TraceCase) -> String {
    let src = read_text(case.source_fixture);
    let _sema = check_source(&src).expect("semantic check");
    let ir = compile_program_to_ir(&src).expect("compile ir");
    let semcode = compile_program_to_semcode(&src).expect("compile semcode");
    verify_semcode(&semcode).expect("verify semcode");
    let token = verify_semcode_token(&semcode).expect("token admission");
    let entry_token = token.require_entry("main").expect("entry resolution");
    run_verified_entry_semcode(&entry_token).expect("run verified semcode");

    let source_hash = hash_hex(&src);
    let mut ir_signatures = Vec::with_capacity(ir.len());
    for func in &ir {
        ir_signatures.push(format!("{}:{}", func.name, func.instrs.len()));
    }
    ir_signatures.sort();
    let ir_hash = hash_hex(&ir_signatures.join("|"));
    let semcode_hash = hash_hex_bytes(&semcode);
    let trace_body = format!(
        "{{\n  \"trace_id\": \"{}\",\n  \"pcc\": \"{}\",\n  \"pcc_owner\": \"{}\",\n  \"surface\": \"{}\",\n  \"source_fixture\": \"{}\",\n  \"trace_class\": [{}],\n  \"stage\": \"{}\",\n  \"expected_status\": \"{}\",\n  \"source_hash\": \"{}\",\n  \"ir_hash\": \"{}\",\n  \"semcode_hash\": \"{}\",\n  \"verifier_status\": \"accept\",\n  \"vm_status\": \"ok\",\n  \"runtime_config\": \"{}\",\n  \"notes\": \"{}\"\n}}\n",
        case.trace_id,
        case.pcc,
        case.pcc_owner,
        case.surface,
        case.source_fixture,
        case
            .trace_class
            .iter()
            .map(|entry| format!("\"{entry}\""))
            .collect::<Vec<_>>()
            .join(", "),
        case.stage,
        case.expected_status,
        source_hash,
        ir_hash,
        semcode_hash,
        case.runtime_config,
        case.notes,
    );
    let expected_output_hash = hash_hex(&trace_body);
    format!(
        "{{\n  \"trace_id\": \"{}\",\n  \"pcc\": \"{}\",\n  \"pcc_owner\": \"{}\",\n  \"surface\": \"{}\",\n  \"source_fixture\": \"{}\",\n  \"trace_class\": [{}],\n  \"stage\": \"{}\",\n  \"expected_status\": \"{}\",\n  \"source_hash\": \"{}\",\n  \"ir_hash\": \"{}\",\n  \"semcode_hash\": \"{}\",\n  \"verifier_status\": \"accept\",\n  \"vm_status\": \"ok\",\n  \"runtime_config\": \"{}\",\n  \"expected_output_hash\": \"{}\",\n  \"notes\": \"{}\"\n}}\n",
        case.trace_id,
        case.pcc,
        case.pcc_owner,
        case.surface,
        case.source_fixture,
        case
            .trace_class
            .iter()
            .map(|entry| format!("\"{entry}\""))
            .collect::<Vec<_>>()
            .join(", "),
        case.stage,
        case.expected_status,
        source_hash,
        ir_hash,
        semcode_hash,
        case.runtime_config,
        expected_output_hash,
        case.notes,
    )
}

#[test]
fn ctf_e1_golden_traces_match_checked_in_selection() {
    assert_text_file(&trace_manifest_path(), &render_trace_manifest());

    for case in TRACE_CASES {
        let artifact = render_trace_artifact(case);
        assert_text_file(&trace_artifact_path(case), &artifact);
    }
}
