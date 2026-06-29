#![cfg(feature = "vm-profile")]

use std::fmt::Write;

use sm_emit::{compile_program_to_semcode, Opcode};
use sm_runtime_core::{ExecutionConfig, ExecutionContext};
use sm_verify::verify_semcode_token;
use sm_vm::{run_verified_entry_semcode_with_profile, VmOpcodeProfile};

const FIXTURE_DIR: &str = "fixtures/profiling";

#[derive(Debug, Clone, Copy)]
struct OpcodeFamilyMetric {
    name: &'static str,
    count: u64,
    ratio: f64,
}

#[derive(Debug, Clone)]
struct OpcodeFamilySummary {
    total_instructions: u64,
    quad_logic: OpcodeFamilyMetric,
    quad_family: OpcodeFamilyMetric,
    control_flow: OpcodeFamilyMetric,
    scalar_movement: OpcodeFamilyMetric,
    calls: OpcodeFamilyMetric,
    integer_ops: OpcodeFamilyMetric,
}

fn profile_source_fixture(name: &str, source: &str) -> VmOpcodeProfile {
    let bytes = compile_program_to_semcode(source).unwrap_or_else(|err| {
        panic!("{name}: compile failed: {err:?}");
    });
    let token = verify_semcode_token(&bytes).unwrap_or_else(|err| {
        panic!("{name}: verify failed: {err}");
    });
    let entry = token.require_entry("main").unwrap_or_else(|err| {
        panic!("{name}: entry resolution failed: {err:?}");
    });

    run_verified_entry_semcode_with_profile(
        &entry,
        ExecutionConfig::for_context(ExecutionContext::VerifiedLocal),
    )
    .unwrap_or_else(|err| panic!("{name}: profiled execution failed: {err}"))
}

fn ratio(count: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (count as f64 / total as f64) * 100.0
    }
}

fn family_summary(profile: &VmOpcodeProfile) -> OpcodeFamilySummary {
    let total = profile.total_instructions();

    let quad_logic_count = profile.count(Opcode::QAnd)
        + profile.count(Opcode::QOr)
        + profile.count(Opcode::QNot)
        + profile.count(Opcode::QImpl);

    let quad_family_count = profile.count(Opcode::LoadQ)
        + profile.count(Opcode::QAnd)
        + profile.count(Opcode::QOr)
        + profile.count(Opcode::QNot)
        + profile.count(Opcode::QImpl)
        + profile.count(Opcode::CmpEq)
        + profile.count(Opcode::CmpNe);

    let control_flow_count = profile.count(Opcode::Jmp) + profile.count(Opcode::JmpIf);
    let scalar_movement_count = profile.count(Opcode::LoadVar) + profile.count(Opcode::StoreVar);
    let calls_count = profile.count(Opcode::Call) + profile.count(Opcode::Ret);

    let integer_ops_count = profile.count(Opcode::LoadI32)
        + profile.count(Opcode::AddI32)
        + profile.count(Opcode::SubI32)
        + profile.count(Opcode::MulI32)
        + profile.count(Opcode::DivI32)
        + profile.count(Opcode::ModI32)
        + profile.count(Opcode::CmpI32Lt)
        + profile.count(Opcode::CmpI32Le);

    OpcodeFamilySummary {
        total_instructions: total,
        quad_logic: OpcodeFamilyMetric {
            name: "quad_logic",
            count: quad_logic_count,
            ratio: ratio(quad_logic_count, total),
        },
        quad_family: OpcodeFamilyMetric {
            name: "quad_family",
            count: quad_family_count,
            ratio: ratio(quad_family_count, total),
        },
        control_flow: OpcodeFamilyMetric {
            name: "control_flow",
            count: control_flow_count,
            ratio: ratio(control_flow_count, total),
        },
        scalar_movement: OpcodeFamilyMetric {
            name: "scalar_movement",
            count: scalar_movement_count,
            ratio: ratio(scalar_movement_count, total),
        },
        calls: OpcodeFamilyMetric {
            name: "calls",
            count: calls_count,
            ratio: ratio(calls_count, total),
        },
        integer_ops: OpcodeFamilyMetric {
            name: "integer_ops",
            count: integer_ops_count,
            ratio: ratio(integer_ops_count, total),
        },
    }
}

fn format_family_summary(summary: &OpcodeFamilySummary) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "families:");
    for metric in [
        summary.quad_logic,
        summary.quad_family,
        summary.control_flow,
        summary.scalar_movement,
        summary.calls,
        summary.integer_ops,
    ] {
        let _ = writeln!(
            out,
            "  {}: {} / {} = {:.2}%",
            metric.name, metric.count, summary.total_instructions, metric.ratio
        );
    }
    out
}

fn print_profile_report(name: &str, profile: &VmOpcodeProfile) {
    println!("fixture={name}");
    println!("{}", profile.summary_top_n(12));
    println!("{}", format_family_summary(&family_summary(profile)));
}

fn fixture_path(name: &str) -> String {
    format!("{FIXTURE_DIR}/{name}")
}

const QUAD_LOGIC_STORM: &str = include_str!("fixtures/profiling/quad_logic_storm.sm");
const QUAD_MATCH_DISPATCH: &str = include_str!("fixtures/profiling/quad_match_dispatch.sm");
const FACT_MERGE_KERNEL: &str = include_str!("fixtures/profiling/fact_merge_kernel.sm");
const FACT_INTERSECT_KERNEL: &str = include_str!("fixtures/profiling/fact_intersect_kernel.sm");
const DELTA_LIKE_KERNEL: &str = include_str!("fixtures/profiling/delta_like_kernel.sm");
const ANDROMEDA_FACT_WAVE_64: &str = include_str!("fixtures/profiling/andromeda_fact_wave_64.sm");
const ANDROMEDA_FACT_WAVE_256: &str =
    include_str!("fixtures/profiling/andromeda_fact_wave_256.sm");

#[test]
fn family_summary_empty_profile_is_zero() {
    let profile = VmOpcodeProfile::default();
    let summary = family_summary(&profile);

    assert!(profile.is_empty());
    assert_eq!(summary.total_instructions, 0);
    assert_eq!(summary.quad_logic.count, 0);
    assert_eq!(summary.quad_family.count, 0);
    assert_eq!(summary.control_flow.count, 0);
    assert_eq!(summary.scalar_movement.count, 0);
    assert_eq!(summary.calls.count, 0);
    assert_eq!(summary.integer_ops.count, 0);
    assert_eq!(summary.quad_logic.ratio, 0.0);
    assert_eq!(summary.quad_family.ratio, 0.0);
    assert_eq!(summary.control_flow.ratio, 0.0);
    assert_eq!(summary.scalar_movement.ratio, 0.0);
    assert_eq!(summary.calls.ratio, 0.0);
    assert_eq!(summary.integer_ops.ratio, 0.0);
}

#[test]
fn ratio_helper_formats_percentages() {
    assert_eq!(ratio(0, 0), 0.0);
    assert_eq!(ratio(3, 10), 30.0);
    assert_eq!(ratio(1, 4), 25.0);
}

#[test]
fn profile_quad_logic_storm() {
    let profile = profile_source_fixture("quad_logic_storm", QUAD_LOGIC_STORM);
    let summary = family_summary(&profile);

    print_profile_report(&fixture_path("quad_logic_storm.sm"), &profile);

    assert!(summary.total_instructions > 0);
    assert!(summary.quad_logic.count > 0);
    assert!(summary.quad_family.count > 0);
    assert!(summary.control_flow.count > 0);
}

#[test]
fn profile_quad_match_dispatch() {
    let profile = profile_source_fixture("quad_match_dispatch", QUAD_MATCH_DISPATCH);
    let summary = family_summary(&profile);

    print_profile_report(&fixture_path("quad_match_dispatch.sm"), &profile);

    assert!(summary.total_instructions > 0);
    assert!(summary.quad_family.count > 0);
    assert!(summary.control_flow.count > 0);
}

#[test]
fn profile_fact_merge_kernel() {
    let profile = profile_source_fixture("fact_merge_kernel", FACT_MERGE_KERNEL);
    let summary = family_summary(&profile);

    print_profile_report(&fixture_path("fact_merge_kernel.sm"), &profile);

    assert!(summary.total_instructions > 0);
    assert!(summary.quad_family.count > 0);
    assert!(profile.count(Opcode::QOr) > 0);
}

#[test]
fn profile_fact_intersect_kernel() {
    let profile = profile_source_fixture("fact_intersect_kernel", FACT_INTERSECT_KERNEL);
    let summary = family_summary(&profile);

    print_profile_report(&fixture_path("fact_intersect_kernel.sm"), &profile);

    assert!(summary.total_instructions > 0);
    assert!(summary.quad_family.count > 0);
    assert!(profile.count(Opcode::QAnd) > 0);
}

#[test]
fn profile_delta_like_kernel() {
    let profile = profile_source_fixture("delta_like_kernel", DELTA_LIKE_KERNEL);
    let summary = family_summary(&profile);

    print_profile_report(&fixture_path("delta_like_kernel.sm"), &profile);

    assert!(summary.total_instructions > 0);
    assert!(summary.quad_family.count > 0);
    assert!(summary.control_flow.count > 0);
    assert!(summary.scalar_movement.count > 0);
}

#[test]
fn profile_andromeda_fact_wave_64() {
    let profile = profile_source_fixture("andromeda_fact_wave_64", ANDROMEDA_FACT_WAVE_64);
    let summary = family_summary(&profile);

    print_profile_report(&fixture_path("andromeda_fact_wave_64.sm"), &profile);

    assert!(summary.total_instructions > 0);
    assert!(summary.quad_family.count > 0);
    assert!(summary.control_flow.count > 0);
    assert!(summary.scalar_movement.count > 0);
}

#[test]
#[ignore]
fn profile_andromeda_fact_wave_256_local() {
    let profile = profile_source_fixture("andromeda_fact_wave_256", ANDROMEDA_FACT_WAVE_256);
    let summary = family_summary(&profile);

    print_profile_report(&fixture_path("andromeda_fact_wave_256.sm"), &profile);

    assert!(summary.total_instructions > 0);
    assert!(summary.quad_family.count > 0);
    assert!(summary.control_flow.count > 0);
    assert!(summary.scalar_movement.count > 0);
}
