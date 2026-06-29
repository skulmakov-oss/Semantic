#![cfg(feature = "vm-profile")]

use sm_emit::{compile_program_to_semcode, Opcode};
use sm_runtime_core::{ExecutionConfig, ExecutionContext};
use sm_verify::verify_semcode_token;
use sm_vm::{run_verified_entry_semcode_with_profile, VmOpcodeProfile};

const FIXTURE_DIR: &str = "fixtures/profiling";

fn profile_source_fixture(source: &str) -> VmOpcodeProfile {
    let bytes = compile_program_to_semcode(source).expect("compile fixture");
    let token = verify_semcode_token(&bytes).expect("verify fixture");
    let entry = token.require_entry("main").expect("main entry");

    run_verified_entry_semcode_with_profile(
        &entry,
        ExecutionConfig::for_context(ExecutionContext::VerifiedLocal),
    )
    .expect("profiled execution")
}

fn quad_logic_count(profile: &VmOpcodeProfile) -> u64 {
    profile.count(Opcode::QAnd)
        + profile.count(Opcode::QOr)
        + profile.count(Opcode::QNot)
        + profile.count(Opcode::QImpl)
}

fn quad_family_count(profile: &VmOpcodeProfile) -> u64 {
    profile.count(Opcode::LoadQ)
        + profile.count(Opcode::QAnd)
        + profile.count(Opcode::QOr)
        + profile.count(Opcode::QNot)
        + profile.count(Opcode::QImpl)
        + profile.count(Opcode::CmpEq)
        + profile.count(Opcode::CmpNe)
}

fn control_flow_count(profile: &VmOpcodeProfile) -> u64 {
    profile.count(Opcode::Jmp) + profile.count(Opcode::JmpIf)
}

fn print_summary(name: &str, profile: &VmOpcodeProfile) {
    println!("fixture={name}");
    println!("{}", profile.summary_top_n(12));
}

fn fixture_path(name: &str) -> String {
    format!("{FIXTURE_DIR}/{name}")
}

#[test]
fn profile_quad_logic_storm() {
    let profile = profile_source_fixture(include_str!("fixtures/profiling/quad_logic_storm.sm"));

    assert!(profile.total_instructions() > 0);
    assert!(profile.count(Opcode::LoadQ) > 0);
    assert!(quad_logic_count(&profile) > 0);
    assert!(profile.count(Opcode::CmpEq) > 0);
    assert!(profile.count(Opcode::CmpNe) > 0);
    print_summary(&fixture_path("quad_logic_storm.sm"), &profile);
}

#[test]
fn profile_quad_match_dispatch() {
    let profile = profile_source_fixture(include_str!("fixtures/profiling/quad_match_dispatch.sm"));

    assert!(profile.total_instructions() > 0);
    assert!(profile.count(Opcode::LoadQ) > 0);
    assert!(control_flow_count(&profile) > 0 || profile.count(Opcode::CmpEq) > 0);
    assert!(profile.count(Opcode::Ret) > 0);
    print_summary(&fixture_path("quad_match_dispatch.sm"), &profile);
}

#[test]
fn profile_fact_merge_kernel() {
    let profile = profile_source_fixture(include_str!("fixtures/profiling/fact_merge_kernel.sm"));

    assert!(profile.total_instructions() > 0);
    assert!(profile.count(Opcode::LoadQ) > 0);
    assert!(profile.count(Opcode::QOr) > 0);
    assert!(control_flow_count(&profile) > 0 || profile.count(Opcode::CmpEq) > 0);
    print_summary(&fixture_path("fact_merge_kernel.sm"), &profile);
}

#[test]
fn profile_fact_intersect_kernel() {
    let profile =
        profile_source_fixture(include_str!("fixtures/profiling/fact_intersect_kernel.sm"));

    assert!(profile.total_instructions() > 0);
    assert!(profile.count(Opcode::LoadQ) > 0);
    assert!(profile.count(Opcode::QAnd) > 0);
    assert!(control_flow_count(&profile) > 0 || profile.count(Opcode::CmpEq) > 0);
    print_summary(&fixture_path("fact_intersect_kernel.sm"), &profile);
}

#[test]
fn profile_delta_like_kernel() {
    let profile = profile_source_fixture(include_str!("fixtures/profiling/delta_like_kernel.sm"));

    assert!(profile.total_instructions() > 0);
    assert!(profile.count(Opcode::LoadQ) > 0);
    assert!(quad_family_count(&profile) > 0);
    assert!(control_flow_count(&profile) > 0 || profile.count(Opcode::CmpEq) > 0);
    print_summary(&fixture_path("delta_like_kernel.sm"), &profile);
}

#[test]
fn profile_andromeda_fact_wave_64() {
    let profile =
        profile_source_fixture(include_str!("fixtures/profiling/andromeda_fact_wave_64.sm"));

    assert!(profile.total_instructions() > 0);
    assert!(profile.count(Opcode::LoadQ) > 0);
    assert!(quad_family_count(&profile) > 0);
    assert!(control_flow_count(&profile) > 0);
    print_summary(&fixture_path("andromeda_fact_wave_64.sm"), &profile);
}

#[test]
#[ignore]
fn profile_andromeda_fact_wave_256_local() {
    let profile = profile_source_fixture(include_str!(
        "fixtures/profiling/andromeda_fact_wave_256.sm"
    ));

    assert!(profile.total_instructions() > 0);
    assert!(profile.count(Opcode::LoadQ) > 0);
    assert!(quad_family_count(&profile) > 0);
    assert!(control_flow_count(&profile) > 0);
    print_summary(&fixture_path("andromeda_fact_wave_256.sm"), &profile);
}
