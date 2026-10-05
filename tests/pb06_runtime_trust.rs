//! PB-06 regressions across the runtime trust boundary.
//!
//! #1769: a public execution API that was given no host has no authority to
//! fabricate host semantics. Every host/effect opcode must fail with
//! `HostAbi(Unavailable)` carrying its exact `HostCallId`, on every public
//! no-host route, while host-free programs keep succeeding.

use prom_abi::{AbiFailureKind, HostCallId};
use sm_ir::{emit_ir_to_semcode, IrFunction, IrInstr};
use sm_vm::{
    run_semcode, run_semcode_collecting_hello_observations, run_verified_semcode, RuntimeError,
};

fn program(body: Vec<IrInstr>) -> Vec<u8> {
    let mut instrs = vec![IrInstr::LoadI32 { dst: 1, val: 7 }];
    instrs.extend(body);
    instrs.push(IrInstr::Ret { src: None });
    emit_ir_to_semcode(
        &[IrFunction {
            name: "main".to_string(),
            instrs,
            ownership_events: Vec::new(),
            params: Vec::new(),
        }],
        false,
    )
    .expect("emit")
}

fn effect_cases() -> Vec<(HostCallId, IrInstr)> {
    vec![
        (
            HostCallId::GateRead,
            IrInstr::GateRead {
                dst: 2,
                device_id: 1,
                port: 2,
            },
        ),
        (
            HostCallId::GateWrite,
            IrInstr::GateWrite {
                device_id: 1,
                port: 2,
                src: 1,
            },
        ),
        (
            HostCallId::PulseEmit,
            IrInstr::PulseEmit { signal: "s".into() },
        ),
        (
            HostCallId::StateQuery,
            IrInstr::StateQuery {
                dst: 2,
                key: "k".into(),
            },
        ),
        (
            HostCallId::StateUpdate,
            IrInstr::StateUpdate {
                key: "k".into(),
                src: 1,
            },
        ),
        (
            HostCallId::EventPost,
            IrInstr::EventPost { signal: "s".into() },
        ),
        (HostCallId::ClockRead, IrInstr::ClockRead { dst: 2 }),
    ]
}

fn assert_unavailable(route: &str, call: HostCallId, result: Result<(), RuntimeError>) {
    match result {
        Err(RuntimeError::HostAbi(err)) => {
            assert_eq!(err.kind, AbiFailureKind::Unavailable, "{route} {call:?}");
            assert_eq!(err.call, call, "{route}: wrong call identity");
        }
        other => panic!("{route} {call:?}: no-host execution must fail closed, got {other:?}"),
    }
}

#[test]
fn every_no_host_route_fails_closed_with_exact_call_identity() {
    for (call, instr) in effect_cases() {
        let bytes = program(vec![instr]);
        assert_unavailable("verified", call, run_verified_semcode(&bytes));
        assert_unavailable("raw", call, run_semcode(&bytes));
        assert_unavailable(
            "raw collecting",
            call,
            run_semcode_collecting_hello_observations(&bytes).map(|_| ()),
        );
    }
}

#[test]
fn host_free_programs_still_succeed_without_a_host() {
    let bytes = program(vec![
        IrInstr::LoadI32 { dst: 2, val: 5 },
        IrInstr::AddI32 {
            dst: 3,
            lhs: 1,
            rhs: 2,
        },
    ]);
    run_verified_semcode(&bytes).expect("pure arithmetic needs no host");
    run_semcode(&bytes).expect("raw pure arithmetic needs no host");
}

/// Removing synthetic host semantics must not reject host-free programs:
/// pure calls, deterministic PRNG and controlled `print` observation are VM
/// owned and keep working on the verified no-host route.
#[test]
fn verified_no_host_route_keeps_vm_owned_operations() {
    let src = r#"
fn add(a: i32, b: i32) -> i32 { return a + b; }
fn main() {
    let x: i32 = add(2, 3);
    random_seed(x);
    let r: i32 = random_next_i32(0, 10);
    print("observed");
    return;
}
"#;
    let bytes = sm_emit::compile_program_to_semcode(src).expect("compile");
    run_verified_semcode(&bytes).expect("host-free program needs no host");
    let events = run_semcode_collecting_hello_observations(&bytes).expect("collect");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].text, "observed");
    assert_eq!(events[0].sequence_index.0, 0);

    let quad = program(vec![
        IrInstr::LoadQ {
            dst: 2,
            val: sm_front::QuadVal::T,
        },
        IrInstr::LoadQ {
            dst: 3,
            val: sm_front::QuadVal::N,
        },
        IrInstr::QAnd {
            dst: 4,
            lhs: 2,
            rhs: 3,
        },
    ]);
    run_verified_semcode(&quad).expect("quad logic needs no host");
}

/// #1764/#1765: `sm-runtime-core` stays a vocabulary-only boundary. It must
/// not regain an executable observation route, nor any caller-forgeable
/// admission seam.
#[test]
fn runtime_core_hello_surface_is_vocabulary_only() {
    let lib = include_str!("../crates/sm-runtime-core/src/lib.rs");
    let sink = include_str!("../crates/sm-runtime-core/src/hello_observation_sink.rs");
    assert!(
        !lib.contains("hello_observation_route"),
        "runtime-core must not own a Hello route module"
    );
    assert!(
        !sink.contains("pub fn"),
        "runtime-core Hello vocabulary must not expose executable functions"
    );
    for forbidden in ["admitted", "NondeterministicOrder"] {
        assert!(
            !sink.contains(forbidden),
            "runtime-core Hello vocabulary contains {forbidden}"
        );
    }
}
