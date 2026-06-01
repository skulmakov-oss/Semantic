use std::time::{Duration, Instant};

#[path = "support/executable_bundle_support.rs"]
mod executable_bundle_support;

use semantic_language::{
    frontend::{
        compile_program_to_ir, compile_program_to_semcode, lex, parse_program_with_profile,
        ParserProfile,
    },
    semantics::check_source_with_profile,
    semcode_verify::verify_semcode,
};
use sm_vm::run_verified_semcode;

const WARMUP_RUNS: usize = 1;
const MEASURED_RUNS: usize = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StageStats {
    min_us: u128,
    median_us: u128,
    max_us: u128,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PipelineSnapshot {
    token_count: usize,
    parsed_function_count: usize,
    sema_warning_count: usize,
    sema_arena_nodes: usize,
    ir_function_count: usize,
    ir_instruction_count: usize,
    semcode_bytes: usize,
    semcode_hash: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScenarioBaseline {
    label: &'static str,
    rel: &'static str,
    snapshot: PipelineSnapshot,
    lex: StageStats,
    parse: StageStats,
    sema: StageStats,
    ir: StageStats,
    emit: StageStats,
    verify: StageStats,
    runtime: StageStats,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct StageDurations {
    lex: Duration,
    parse: Duration,
    sema: Duration,
    ir: Duration,
    emit: Duration,
    verify: Duration,
    runtime: Duration,
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

fn measure_once(src: &str, profile: &ParserProfile) -> (PipelineSnapshot, StageDurations) {
    let t0 = Instant::now();
    let tokens = lex(src).expect("lex");
    let t1 = Instant::now();
    let parsed = parse_program_with_profile(src, profile).expect("parse");
    let t2 = Instant::now();
    let sema = check_source_with_profile(src, profile).expect("semantic check");
    let t3 = Instant::now();
    let ir = compile_program_to_ir(src).expect("compile ir");
    let t4 = Instant::now();
    let semcode = compile_program_to_semcode(src).expect("compile semcode");
    let t5 = Instant::now();
    verify_semcode(&semcode).expect("verify");
    let t6 = Instant::now();
    run_verified_semcode(&semcode).expect("verified run");
    let t7 = Instant::now();

    let snapshot = PipelineSnapshot {
        token_count: tokens.len(),
        parsed_function_count: parsed.functions.len(),
        sema_warning_count: sema.warnings.len(),
        sema_arena_nodes: sema.arena_nodes,
        ir_function_count: ir.len(),
        ir_instruction_count: ir.iter().map(|func| func.instrs.len()).sum(),
        semcode_bytes: semcode.len(),
        semcode_hash: fnv1a64(&semcode),
    };
    let durations = StageDurations {
        lex: t1 - t0,
        parse: t2 - t1,
        sema: t3 - t2,
        ir: t4 - t3,
        emit: t5 - t4,
        verify: t6 - t5,
        runtime: t7 - t6,
    };
    (snapshot, durations)
}

fn summarize(mut values: Vec<u128>) -> StageStats {
    values.sort_unstable();
    let mid = values.len() / 2;
    StageStats {
        min_us: values[0],
        median_us: values[mid],
        max_us: values[values.len() - 1],
    }
}

fn measure_scenario(label: &'static str, rel: &'static str) -> ScenarioBaseline {
    let src = executable_bundle_support::bundle_source(rel);
    let profile = ParserProfile::foundation_default();

    for _ in 0..WARMUP_RUNS {
        let _ = measure_once(&src, &profile);
    }

    let mut first_snapshot: Option<PipelineSnapshot> = None;
    let mut lex = Vec::with_capacity(MEASURED_RUNS);
    let mut parse = Vec::with_capacity(MEASURED_RUNS);
    let mut sema = Vec::with_capacity(MEASURED_RUNS);
    let mut ir = Vec::with_capacity(MEASURED_RUNS);
    let mut emit = Vec::with_capacity(MEASURED_RUNS);
    let mut verify = Vec::with_capacity(MEASURED_RUNS);
    let mut runtime = Vec::with_capacity(MEASURED_RUNS);

    for _ in 0..MEASURED_RUNS {
        let (snapshot, durations) = measure_once(&src, &profile);
        if let Some(expected) = &first_snapshot {
            assert_eq!(snapshot, *expected, "pipeline snapshot drifted for {rel}");
        } else {
            first_snapshot = Some(snapshot);
        }
        lex.push(durations.lex.as_micros());
        parse.push(durations.parse.as_micros());
        sema.push(durations.sema.as_micros());
        ir.push(durations.ir.as_micros());
        emit.push(durations.emit.as_micros());
        verify.push(durations.verify.as_micros());
        runtime.push(durations.runtime.as_micros());
    }

    let snapshot = first_snapshot.expect("measured snapshot");
    ScenarioBaseline {
        label,
        rel,
        snapshot,
        lex: summarize(lex),
        parse: summarize(parse),
        sema: summarize(sema),
        ir: summarize(ir),
        emit: summarize(emit),
        verify: summarize(verify),
        runtime: summarize(runtime),
    }
}

fn render_stage(name: &str, stats: StageStats) -> String {
    format!(
        "{name}_us=min:{} median:{} max:{}",
        stats.min_us, stats.median_us, stats.max_us
    )
}

fn render_suite(scenarios: &[ScenarioBaseline]) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "warmup_runs={WARMUP_RUNS}\nmeasured_runs={MEASURED_RUNS}\n\n"
    ));
    for scenario in scenarios {
        out.push_str(&format!("scenario={}\n", scenario.label));
        out.push_str(&format!("path={}\n", scenario.rel));
        out.push_str(&format!(
            "snapshot=tokens:{} parsed_functions:{} sema_warnings:{} sema_arena_nodes:{} ir_functions:{} ir_instructions:{} semcode_bytes:{} semcode_hash:{:016x}\n",
            scenario.snapshot.token_count,
            scenario.snapshot.parsed_function_count,
            scenario.snapshot.sema_warning_count,
            scenario.snapshot.sema_arena_nodes,
            scenario.snapshot.ir_function_count,
            scenario.snapshot.ir_instruction_count,
            scenario.snapshot.semcode_bytes,
            scenario.snapshot.semcode_hash,
        ));
        out.push_str(&format!("{}\n", render_stage("lex", scenario.lex)));
        out.push_str(&format!("{}\n", render_stage("parse", scenario.parse)));
        out.push_str(&format!("{}\n", render_stage("sema", scenario.sema)));
        out.push_str(&format!("{}\n", render_stage("ir", scenario.ir)));
        out.push_str(&format!("{}\n", render_stage("emit", scenario.emit)));
        out.push_str(&format!("{}\n", render_stage("verify", scenario.verify)));
        out.push_str(&format!(
            "{}\n\n",
            render_stage("runtime", scenario.runtime)
        ));
    }
    out
}

#[test]
fn g1_benchmark_baseline_collects_reproducible_pipeline_metrics() {
    let scenarios = vec![
        measure_scenario(
            "small_cli_core",
            "examples/qualification/g1_real_program_trial/cli_batch_core/src/main.sm",
        ),
        measure_scenario(
            "medium_rule_state",
            "examples/qualification/g1_real_program_trial/rule_state_decision/src/main.sm",
        ),
        measure_scenario(
            "record_iterable_data",
            "examples/qualification/g1_real_program_trial/data_audit_record_iterable/src/main.sm",
        ),
        measure_scenario(
            "module_helper_entry",
            "examples/qualification/executable_module_entry/wave2_local_helper_import/src/main.sm",
        ),
        measure_scenario(
            "selected_import_entry",
            "examples/qualification/executable_module_entry/positive_selected_import/src/main.sm",
        ),
    ];

    for scenario in &scenarios {
        assert!(scenario.lex.min_us <= scenario.lex.median_us);
        assert!(scenario.lex.median_us <= scenario.lex.max_us);
        assert!(scenario.parse.min_us <= scenario.parse.median_us);
        assert!(scenario.parse.median_us <= scenario.parse.max_us);
        assert!(scenario.sema.min_us <= scenario.sema.median_us);
        assert!(scenario.sema.median_us <= scenario.sema.max_us);
        assert!(scenario.ir.min_us <= scenario.ir.median_us);
        assert!(scenario.ir.median_us <= scenario.ir.max_us);
        assert!(scenario.emit.min_us <= scenario.emit.median_us);
        assert!(scenario.emit.median_us <= scenario.emit.max_us);
        assert!(scenario.verify.min_us <= scenario.verify.median_us);
        assert!(scenario.verify.median_us <= scenario.verify.max_us);
        assert!(scenario.runtime.min_us <= scenario.runtime.median_us);
        assert!(scenario.runtime.median_us <= scenario.runtime.max_us);
        assert!(scenario.snapshot.token_count > 0);
        assert!(scenario.snapshot.parsed_function_count > 0);
        assert!(scenario.snapshot.ir_function_count > 0);
        assert!(scenario.snapshot.ir_instruction_count > 0);
        assert!(scenario.snapshot.semcode_bytes > 0);
        assert_ne!(scenario.snapshot.semcode_hash, 0);
    }

    println!("{}", render_suite(&scenarios));
}
