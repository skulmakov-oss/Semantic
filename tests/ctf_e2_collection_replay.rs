use std::fs;
use std::path::PathBuf;

use semantic_language::{
    frontend::{compile_program_to_ir, compile_program_to_semcode},
    semantics::check_source,
};

fn run_token_first_main(semcode: &[u8]) -> Result<(), sm_vm::RuntimeError> {
    let token = sm_verify::verify_semcode_token(semcode).expect("token admission");
    let entry_token = token.require_entry("main").expect("entry resolution");
    sm_vm::run_verified_entry_semcode(&entry_token)
}

const REPLAY_COUNT: usize = 5;
const UPDATE_ENV: &str = "SM_UPDATE_CTF_E2_REPLAYS";

#[derive(Clone, Copy)]
struct ReplayCase {
    replay_id: &'static str,
    pcc: &'static str,
    surface: &'static str,
    source_fixture: &'static str,
    artifact_file: &'static str,
    replay_class: &'static [&'static str],
    expected_status: &'static str,
    boundary: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReplayRun {
    run_index: usize,
    check_status: &'static str,
    source_hash: String,
    ir_shape_hash: String,
    semcode_hash: String,
    verifier_status: &'static str,
    vm_status: &'static str,
}

const TRACE_CASES: &[ReplayCase] = &[
    ReplayCase {
        replay_id: "CTF-E2-PCC7-SEQUENCE-INDEXING-REPLAY-001",
        pcc: "PCC-7",
        surface: "Sequence indexing",
        source_fixture: "tests/fixtures/pcc7_sequence/positive_sequence_indexing.sm",
        artifact_file: "CTF-E2-PCC7-SEQUENCE-INDEXING-REPLAY-001.replay.json",
        replay_class: &["check", "ir", "semcode", "verifier", "vm"],
        expected_status: "accept",
        boundary: "current admitted PCC-7 Sequence baseline only; no Map policy widening",
    },
    ReplayCase {
        replay_id: "CTF-E2-PCC7-SEQUENCE-ITERATION-REPLAY-001",
        pcc: "PCC-7",
        surface: "Sequence iteration",
        source_fixture: "tests/fixtures/pcc7_sequence/positive_sequence_iteration.sm",
        artifact_file: "CTF-E2-PCC7-SEQUENCE-ITERATION-REPLAY-001.replay.json",
        replay_class: &["check", "ir", "semcode", "verifier", "vm"],
        expected_status: "accept",
        boundary: "current admitted PCC-7 Sequence baseline only; no Map iteration policy widening",
    },
    ReplayCase {
        replay_id: "CTF-E2-PCC7-SEQUENCE-MUTATION-REPLAY-001",
        pcc: "PCC-7",
        surface: "Sequence mutation",
        source_fixture: "tests/fixtures/pcc7_sequence/positive_sequence_push_prepend.sm",
        artifact_file: "CTF-E2-PCC7-SEQUENCE-MUTATION-REPLAY-001.replay.json",
        replay_class: &["check", "ir", "semcode", "verifier", "vm"],
        expected_status: "accept",
        boundary: "current admitted PCC-7 Sequence baseline only; no collection quota widening",
    },
    ReplayCase {
        replay_id: "CTF-E2-PCC7-MAP-INSERT-LOOKUP-REPLAY-001",
        pcc: "PCC-7",
        surface: "Map insert/lookup",
        source_fixture: "tests/fixtures/pcc7_map/positive_map_basic_insert_lookup.sm",
        artifact_file: "CTF-E2-PCC7-MAP-INSERT-LOOKUP-REPLAY-001.replay.json",
        replay_class: &["check", "ir", "semcode", "verifier", "vm"],
        expected_status: "accept",
        boundary: "current admitted PCC-7 Map baseline only; missing-key and iteration remain open",
    },
    ReplayCase {
        replay_id: "CTF-E2-PCC7-MAP-PERSISTENT-UPDATE-REPLAY-001",
        pcc: "PCC-7",
        surface: "Map persistent update",
        source_fixture: "tests/fixtures/pcc7_map/positive_map_persistent_update.sm",
        artifact_file: "CTF-E2-PCC7-MAP-PERSISTENT-UPDATE-REPLAY-001.replay.json",
        replay_class: &["check", "ir", "semcode", "verifier", "vm"],
        expected_status: "accept",
        boundary: "current admitted PCC-7 Map baseline only; quota, iteration, and missing-key remain open",
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

fn replay_dir() -> PathBuf {
    repo_path("tests/fixtures/core_trust_freeze/replay/ctf_e2")
}

fn replay_manifest_path() -> PathBuf {
    replay_dir().join("manifest.md")
}

fn replay_artifact_path(case: &ReplayCase) -> PathBuf {
    replay_dir().join(case.artifact_file)
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
        "replay artifact drifted at {}",
        path.display()
    );
}

fn json_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 8);
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn render_manifest() -> String {
    let mut out = String::new();
    out.push_str("# CTF-E2 Collection Replay Manifest\n");
    out.push_str("Status: checked-in replay selection\n");
    out.push_str("Owner: language maturity / execution contract\n");
    out.push_str("Scope: admitted PCC-7 collection replay evidence only\n");
    out.push_str(
        "Non-goal: collection policy changes, project-root determinism, or release freeze\n\n",
    );
    out.push_str("| replay_id | pcc | surface | source_fixture | artifact | expected_status |\n");
    out.push_str("| --- | --- | --- | --- | --- | --- |\n");
    for case in TRACE_CASES {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            case.replay_id,
            case.pcc,
            case.surface,
            case.source_fixture,
            case.artifact_file,
            case.expected_status
        ));
    }
    out
}

fn run_case_once(case: &ReplayCase) -> ReplayRun {
    let src = read_text(case.source_fixture);
    let _sema = check_source(&src).expect("semantic check");
    let ir = compile_program_to_ir(&src).expect("compile ir");
    let semcode = compile_program_to_semcode(&src).expect("compile semcode");
    run_token_first_main(&semcode).expect("run verified semcode");

    let source_hash = hash_hex(&src);
    let mut ir_signatures = Vec::with_capacity(ir.len());
    for func in &ir {
        ir_signatures.push(format!("{}:{}", func.name, func.instrs.len()));
    }
    ir_signatures.sort();
    let ir_shape_hash = hash_hex(&ir_signatures.join("|"));
    let semcode_hash = hash_hex_bytes(&semcode);

    ReplayRun {
        run_index: 0,
        check_status: "accept",
        source_hash,
        ir_shape_hash,
        semcode_hash,
        verifier_status: "accept",
        vm_status: "ok",
    }
}

fn collect_runs(case: &ReplayCase) -> Vec<ReplayRun> {
    let mut runs = Vec::with_capacity(REPLAY_COUNT);
    for run_index in 1..=REPLAY_COUNT {
        let mut run = run_case_once(case);
        run.run_index = run_index;
        runs.push(run);
    }
    runs
}

fn runs_all_equal(runs: &[ReplayRun]) -> bool {
    if runs.is_empty() {
        return true;
    }
    runs.iter().all(|run| {
        run.check_status == runs[0].check_status
            && run.source_hash == runs[0].source_hash
            && run.ir_shape_hash == runs[0].ir_shape_hash
            && run.semcode_hash == runs[0].semcode_hash
            && run.verifier_status == runs[0].verifier_status
            && run.vm_status == runs[0].vm_status
    })
}

fn render_runs(runs: &[ReplayRun]) -> String {
    let mut out = String::new();
    out.push_str("[\n");
    for (idx, run) in runs.iter().enumerate() {
        out.push_str(&format!(
            "  {{\n    \"run_index\": {},\n    \"check_status\": \"{}\",\n    \"source_hash\": \"{}\",\n    \"ir_shape_hash\": \"{}\",\n    \"semcode_hash\": \"{}\",\n    \"verifier_status\": \"{}\",\n    \"vm_status\": \"{}\"\n  }}",
            run.run_index,
            run.check_status,
            run.source_hash,
            run.ir_shape_hash,
            run.semcode_hash,
            run.verifier_status,
            run.vm_status
        ));
        if idx + 1 != runs.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("]\n");
    out
}

fn render_replay_artifact(case: &ReplayCase) -> String {
    let src = read_text(case.source_fixture);
    let source_hash = hash_hex(&src);
    let runs = collect_runs(case);
    assert!(
        runs_all_equal(&runs),
        "replay drifted for {}",
        case.replay_id
    );
    let summary_hash = hash_hex(&render_runs(&runs));
    let trace_class = case
        .replay_class
        .iter()
        .map(|entry| format!("\"{}\"", json_escape(entry)))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "{{\n  \"replay_id\": \"{}\",\n  \"pcc\": \"{}\",\n  \"surface\": \"{}\",\n  \"source_fixture\": \"{}\",\n  \"replay_class\": [{}],\n  \"replay_count\": {},\n  \"expected_status\": \"{}\",\n  \"source_hash\": \"{}\",\n  \"summary_hash\": \"{}\",\n  \"verifier_status\": \"accept\",\n  \"vm_status\": \"ok\",\n  \"all_runs_equal\": true,\n  \"boundary\": \"{}\",\n  \"runs\": {}\n}}\n",
        json_escape(case.replay_id),
        json_escape(case.pcc),
        json_escape(case.surface),
        json_escape(case.source_fixture),
        trace_class,
        REPLAY_COUNT,
        case.expected_status,
        source_hash,
        summary_hash,
        json_escape(case.boundary),
        render_runs(&runs),
    )
}

#[test]
fn ctf_e2_collection_replay_matches_checked_in_selection() {
    assert_text_file(&replay_manifest_path(), &render_manifest());

    for case in TRACE_CASES {
        let artifact = render_replay_artifact(case);
        assert_text_file(&replay_artifact_path(case), &artifact);
    }
}
