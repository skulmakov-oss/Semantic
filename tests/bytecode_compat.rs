use semantic_language::frontend::{
    compile_program_to_semcode, compile_program_to_semcode_with_options_debug, emit_ir_to_semcode,
    CompileProfile, IrFunction, IrInstr, OptLevel,
};
use semantic_language::prom_abi::{AbiValue, RecordingHostAbi};
use semantic_language::prom_cap::{CapabilityKind, CapabilityManifest};
use semantic_language::semcode_format::{
    header_spec_from_magic, CAP_CLOCK_READ, CAP_CLOSURE_VALUES, CAP_EVENT_POST, CAP_F64_MATH,
    CAP_FX_MATH, CAP_FX_VALUES, CAP_GATE_SURFACE, CAP_SEQUENCE_VALUES, CAP_STATE_QUERY,
    CAP_STATE_UPDATE, CAP_TEXT_VALUES, MAGIC0, MAGIC1, MAGIC10, MAGIC2, MAGIC3, MAGIC4, MAGIC5,
    MAGIC6, MAGIC7, MAGIC8, MAGIC9,
};
use semantic_language::semcode_vm::{
    disasm_semcode, run_semcode, run_verified_semcode_with_host_and_capabilities, RuntimeError,
};
use sm_vm::run_verified_semcode;

fn first_function_code_offset(bytes: &[u8]) -> usize {
    let name_len = u16::from_le_bytes([bytes[8], bytes[9]]) as usize;
    8 + 2 + name_len + 4
}

fn compile_cli_default_semcode(src: &str) -> Vec<u8> {
    compile_program_to_semcode_with_options_debug(src, CompileProfile::Auto, OptLevel::O0, false)
        .expect("compile")
}

#[test]
fn compat_v0_header_and_run() {
    let src = "fn main() { return; }";
    let bytes = compile_program_to_semcode(src).expect("compile");
    assert_eq!(&bytes[0..8], &MAGIC0);
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[0..8]);
    let spec = header_spec_from_magic(&magic).expect("known header");
    assert_eq!(spec.epoch, 0);
    assert_eq!(spec.rev, 1);
    assert_eq!(spec.capabilities & CAP_F64_MATH, 0);
    assert_ne!(spec.capabilities & CAP_GATE_SURFACE, 0);
    run_verified_semcode(&bytes).expect("verified run");
}

#[test]
fn compat_v1_header_and_run() {
    let src = r#"
        fn main() {
            let x: f64 = 1.0 + 2.0;
            return;
        }
    "#;
    let bytes = compile_program_to_semcode(src).expect("compile");
    assert_eq!(&bytes[0..8], &MAGIC1);
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[0..8]);
    let spec = header_spec_from_magic(&magic).expect("known header");
    assert_eq!(spec.epoch, 0);
    assert_eq!(spec.rev, 2);
    assert_ne!(spec.capabilities & CAP_F64_MATH, 0);
    run_verified_semcode(&bytes).expect("verified run");
}

#[test]
fn compat_i32_value_path_runs_under_v0_header() {
    let src = r#"
        fn id(x: i32) -> i32 {
            return x;
        }

        fn main() {
            let x: i32 = 1;
            let y: i32 = id(2);
            if x != y { return; } else { return; }
        }
    "#;
    let bytes = compile_program_to_semcode(src).expect("compile");
    assert_eq!(&bytes[0..8], &MAGIC0);
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[0..8]);
    let spec = header_spec_from_magic(&magic).expect("known header");
    assert_eq!(spec.epoch, 0);
    assert_eq!(spec.rev, 1);
    assert_eq!(spec.capabilities & CAP_F64_MATH, 0);
    assert_eq!(spec.capabilities & CAP_FX_VALUES, 0);
    run_verified_semcode(&bytes).expect("verified run");
}

#[test]
fn compat_v2_header_and_run() {
    let src = r#"
        fn id(x: fx) -> fx {
            return x;
        }

        fn main() {
            let x: fx = 1.25;
            let y: fx = id(-2.0);
            if x == x { return; } else { return; }
        }
    "#;
    let bytes = compile_program_to_semcode(src).expect("compile");
    assert_eq!(&bytes[0..8], &MAGIC2);
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[0..8]);
    let spec = header_spec_from_magic(&magic).expect("known header");
    assert_eq!(spec.epoch, 0);
    assert_eq!(spec.rev, 3);
    assert_ne!(spec.capabilities & CAP_FX_VALUES, 0);
    run_verified_semcode(&bytes).expect("verified run");
}

#[test]
fn compat_v3_header_and_run() {
    let src = r#"
        fn main() {
            let a: fx = 2.5;
            let b: fx = 1.5;
            let c: fx = a + b;
            let expected: fx = 4.0;
            assert(c == expected);
            return;
        }
    "#;
    let bytes = compile_program_to_semcode(src).expect("compile");
    assert_eq!(&bytes[0..8], &MAGIC3);
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[0..8]);
    let spec = header_spec_from_magic(&magic).expect("known header");
    assert_eq!(spec.epoch, 0);
    assert_eq!(spec.rev, 4);
    assert_ne!(spec.capabilities & CAP_FX_VALUES, 0);
    assert_ne!(spec.capabilities & CAP_FX_MATH, 0);
    run_verified_semcode(&bytes).expect("verified run");
}

#[test]
fn compat_v4_header_and_state_query_run() {
    let bytes = emit_ir_to_semcode(
        &[IrFunction {
            name: "main".to_string(),
            instrs: vec![
                IrInstr::StateQuery {
                    dst: 0,
                    key: "decision.mode".to_string(),
                },
                IrInstr::LoadI32 { dst: 1, val: 123 },
                IrInstr::CmpEq {
                    dst: 2,
                    lhs: 0,
                    rhs: 1,
                },
                IrInstr::Assert { cond: 2 },
                IrInstr::Ret { src: None },
            ],
            ownership_events: Vec::new(),
        }],
        false,
    )
    .expect("emit");
    assert_eq!(&bytes[0..8], &MAGIC4);
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[0..8]);
    let spec = header_spec_from_magic(&magic).expect("known header");
    assert_eq!(spec.epoch, 0);
    assert_eq!(spec.rev, 5);
    assert_ne!(spec.capabilities & CAP_STATE_QUERY, 0);
    let mut manifest = CapabilityManifest::new();
    manifest.allow(CapabilityKind::StateQuery);
    let mut host = RecordingHostAbi::with_state_query_value(AbiValue::I32(123));
    run_verified_semcode_with_host_and_capabilities(&bytes, &mut host, &manifest)
        .expect("verified run");
    assert_eq!(host.state_queries, vec!["decision.mode".to_string()]);
}

#[test]
fn compat_v5_header_and_state_update_run() {
    let bytes = emit_ir_to_semcode(
        &[IrFunction {
            name: "main".to_string(),
            instrs: vec![
                IrInstr::LoadBool { dst: 0, val: true },
                IrInstr::StateUpdate {
                    key: "decision.mode".to_string(),
                    src: 0,
                },
                IrInstr::Ret { src: None },
            ],
            ownership_events: Vec::new(),
        }],
        false,
    )
    .expect("emit");
    assert_eq!(&bytes[0..8], &MAGIC5);
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[0..8]);
    let spec = header_spec_from_magic(&magic).expect("known header");
    assert_eq!(spec.epoch, 0);
    assert_eq!(spec.rev, 6);
    assert_ne!(spec.capabilities & CAP_STATE_UPDATE, 0);
    let mut manifest = CapabilityManifest::new();
    manifest.allow(CapabilityKind::StateUpdate);
    let mut host = RecordingHostAbi::default();
    run_verified_semcode_with_host_and_capabilities(&bytes, &mut host, &manifest)
        .expect("verified run");
    assert_eq!(
        host.state_updates,
        vec![("decision.mode".to_string(), AbiValue::Bool(true))]
    );
}

#[test]
fn compat_v6_header_and_event_post_run() {
    let bytes = emit_ir_to_semcode(
        &[IrFunction {
            name: "main".to_string(),
            instrs: vec![
                IrInstr::EventPost {
                    signal: "alert.raised".to_string(),
                },
                IrInstr::Ret { src: None },
            ],
            ownership_events: Vec::new(),
        }],
        false,
    )
    .expect("emit");
    assert_eq!(&bytes[0..8], &MAGIC6);
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[0..8]);
    let spec = header_spec_from_magic(&magic).expect("known header");
    assert_eq!(spec.epoch, 0);
    assert_eq!(spec.rev, 7);
    assert_ne!(spec.capabilities & CAP_EVENT_POST, 0);
    let mut manifest = CapabilityManifest::new();
    manifest.allow(CapabilityKind::EventPost);
    let mut host = RecordingHostAbi::default();
    run_verified_semcode_with_host_and_capabilities(&bytes, &mut host, &manifest)
        .expect("verified run");
    assert_eq!(host.event_posts, vec!["alert.raised".to_string()]);
}

#[test]
fn compat_v7_header_and_clock_read_run() {
    let bytes = emit_ir_to_semcode(
        &[IrFunction {
            name: "main".to_string(),
            instrs: vec![
                IrInstr::ClockRead { dst: 0 },
                IrInstr::LoadU32 { dst: 1, val: 42 },
                IrInstr::CmpEq {
                    dst: 2,
                    lhs: 0,
                    rhs: 1,
                },
                IrInstr::Assert { cond: 2 },
                IrInstr::Ret { src: None },
            ],
            ownership_events: Vec::new(),
        }],
        false,
    )
    .expect("emit");
    assert_eq!(&bytes[0..8], &MAGIC7);
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[0..8]);
    let spec = header_spec_from_magic(&magic).expect("known header");
    assert_eq!(spec.epoch, 0);
    assert_eq!(spec.rev, 8);
    assert_ne!(spec.capabilities & CAP_CLOCK_READ, 0);
    let mut manifest = CapabilityManifest::new();
    manifest.allow(CapabilityKind::ClockRead);
    let mut host = RecordingHostAbi::with_clock_read_value(42);
    run_verified_semcode_with_host_and_capabilities(&bytes, &mut host, &manifest)
        .expect("verified run");
    assert_eq!(host.clock_reads, 1);
}

#[test]
fn compat_v8_header_and_text_run() {
    let src = r#"
        fn echo(x: text) -> text { return x; }

        fn main() {
            let left: text = "alpha";
            let right: text = echo("alpha");
            assert(left == right);
            assert(left != "beta");
            return;
        }
    "#;
    let bytes = compile_program_to_semcode(src).expect("compile");
    assert_eq!(&bytes[0..8], &MAGIC8);
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[0..8]);
    let spec = header_spec_from_magic(&magic).expect("known header");
    assert_eq!(spec.epoch, 0);
    assert_eq!(spec.rev, 9);
    assert_ne!(spec.capabilities & CAP_TEXT_VALUES, 0);
    run_verified_semcode(&bytes).expect("verified run");
}

#[test]
fn compat_v9_header_and_sequence_run() {
    let src = r#"
        fn main() {
            let values: Sequence(i32) = [1, 2, 3];
            let head: i32 = values[0];
            assert(head == 1);
            assert(values == [1, 2, 3]);
            return;
        }
    "#;
    let bytes = compile_program_to_semcode(src).expect("compile");
    assert_eq!(&bytes[0..8], &MAGIC9);
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[0..8]);
    let spec = header_spec_from_magic(&magic).expect("known header");
    assert_eq!(spec.epoch, 0);
    assert_eq!(spec.rev, 10);
    assert_ne!(spec.capabilities & CAP_SEQUENCE_VALUES, 0);
    run_verified_semcode(&bytes).expect("verified run");
}

#[test]
fn compat_v10_header_and_closure_run() {
    let src = r#"
        fn main() {
            let offset: f64 = 1.0;
            let add: Closure(f64 -> f64) = (x => x + offset);
            let total: f64 = add(2.0);
            assert(total == 3.0);
            return;
        }
    "#;
    let bytes = compile_program_to_semcode(src).expect("compile");
    assert_eq!(&bytes[0..8], &MAGIC10);
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[0..8]);
    let spec = header_spec_from_magic(&magic).expect("known header");
    assert_eq!(spec.epoch, 0);
    assert_eq!(spec.rev, 11);
    assert_ne!(spec.capabilities & CAP_CLOSURE_VALUES, 0);
    run_verified_semcode(&bytes).expect("verified run");
}

#[test]
fn compat_cli_o0_v1_f64_arithmetic_runs_on_verified_path() {
    let src = r#"
        fn main() {
            let y: f64 = 1.0 + 2.0;
            return;
        }
    "#;
    let bytes = compile_cli_default_semcode(src);
    assert_eq!(&bytes[0..8], &MAGIC1);
    run_verified_semcode(&bytes).expect("verified run");
}

#[test]
fn compat_cli_o0_v1_builtin_call_runs_on_verified_path() {
    let src = r#"
        fn main() {
            let y: f64 = sqrt(16.0);
            return;
        }
    "#;
    let bytes = compile_cli_default_semcode(src);
    assert_eq!(&bytes[0..8], &MAGIC1);
    run_verified_semcode(&bytes).expect("verified run");
}

#[test]
fn compat_cli_o0_complex_semantic_stress_runs_on_verified_path() {
    let src = r#"
        fn sensor_a() -> f64 {
            return 1.0 + 2.0;
        }

        fn sensor_b() -> f64 {
            return sqrt(16.0);
        }

        fn normalize(v: f64) -> f64 {
            let delta: f64 = v - 3.0;
            return abs(delta);
        }

        fn fused_signal(a: f64, b: f64) -> f64 {
            let sum: f64 = a + b;
            let energy: f64 = pow(sum, 2.0);
            return sqrt(energy);
        }

        fn risk_score(signal: f64, bias: f64) -> f64 {
            let weighted: f64 = signal / bias;
            return normalize(weighted);
        }

        fn final_decision(score: f64) -> quad {
            if score == 0.0 {
                return T;
            } else {
                return N;
            }
            return N;
        }

        fn main() {
            let a: f64 = sensor_a();
            let b: f64 = sensor_b();
            let fused: f64 = fused_signal(a, b);
            let score: f64 = risk_score(fused, 7.0);
            let decision: quad = final_decision(score);
            if decision == T { return; } else { return; }
        }
    "#;
    let bytes = compile_cli_default_semcode(src);
    assert_eq!(&bytes[0..8], &MAGIC1);
    run_verified_semcode(&bytes).expect("verified run");
}

#[test]
fn compat_example_semantic_policy_overdrive_trace_runs_on_verified_path() {
    let src = include_str!("../examples/semantic_policy_overdrive_trace.sm");
    let bytes = compile_cli_default_semcode(src);
    assert_eq!(&bytes[0..8], &MAGIC1);
    run_verified_semcode(&bytes).expect("verified run");

    let disasm = disasm_semcode(&bytes).expect("disasm");
    assert!(disasm.contains("fn fusion_consensus_state:"));
    assert!(disasm.contains("fn policy_trace_guard:"));
    assert!(disasm.contains("fn policy_trace_quality:"));
    assert!(disasm.contains("fn policy_trace:"));
}

#[test]
fn compat_unsupported_version_has_migration_hint() {
    let src = "fn main() { return; }";
    let mut bytes = compile_program_to_semcode(src).expect("compile");
    bytes[7] = b'X';
    let err = run_semcode(&bytes).expect_err("must fail");
    match err {
        RuntimeError::UnsupportedBytecodeVersion { found, supported } => {
            assert!(found.starts_with("SEMCODE"));
            assert!(supported.contains("SEMCODE0"));
            assert!(supported.contains("SEMCODE1"));
            assert!(supported.contains("SEMCODE2"));
            assert!(supported.contains("SEMCODE3"));
            assert!(supported.contains("SEMCODE4"));
            assert!(supported.contains("SEMCODE5"));
            assert!(supported.contains("SEMCODE6"));
            assert!(supported.contains("SEMCODE7"));
            assert!(supported.contains("SEMCODE8"));
            assert!(supported.contains("SEMCODE9"));
            assert!(supported.contains("SEMCOD10"));
            assert!(supported.contains("SEMCOD11"));
            assert!(supported.contains("SEMCOD12"));
            assert!(supported.contains("SEMCOD13"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn compat_rejects_too_many_strings_in_function_section() {
    let src = "fn main() { return; }";
    let mut bytes = compile_program_to_semcode(src).expect("compile");
    let code_off = first_function_code_offset(&bytes);
    bytes[code_off] = 0x01;
    bytes[code_off + 1] = 0x20; // 8193 strings
    let err = run_semcode(&bytes).expect_err("must fail");
    match err {
        RuntimeError::BadFormat(msg) => assert!(msg.contains("too many strings")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn compat_rejects_too_many_debug_symbols_in_function_section() {
    let src = "fn main() { return; }";
    let mut bytes = compile_program_to_semcode_with_options_debug(
        src,
        CompileProfile::RustLike,
        OptLevel::O0,
        true,
    )
    .expect("compile");
    let code_off = first_function_code_offset(&bytes);
    let rel = bytes[code_off..]
        .windows(4)
        .position(|w| w == b"DBG0")
        .expect("DBG0 marker");
    let dbg_count_off = code_off + rel + 4;
    bytes[dbg_count_off] = 0x01;
    bytes[dbg_count_off + 1] = 0x20; // 8193 debug entries
    let err = run_semcode(&bytes).expect_err("must fail");
    match err {
        RuntimeError::BadFormat(msg) => assert!(msg.contains("too many debug symbols")),
        other => panic!("unexpected error: {other:?}"),
    }
}
