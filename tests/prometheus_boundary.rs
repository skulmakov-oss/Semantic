use semantic_language::frontend::{emit_ir_to_semcode, IrFunction, IrInstr};
use semantic_language::prom_abi::HostCallId;
use semantic_language::prom_abi::{AbiValue, RecordingHostAbi};
use semantic_language::prom_cap::{CapabilityKind, CapabilityManifest};
use semantic_language::semcode_vm::RuntimeError;

fn run_token_first_with_host(
    semcode: &[u8],
    host: &mut RecordingHostAbi,
    manifest: &CapabilityManifest,
) -> Result<(), RuntimeError> {
    let token = sm_verify::verify_semcode_token(semcode).expect("token admission");
    let entry_token = token.require_entry("main").expect("entry resolution");
    sm_vm::run_verified_entry_semcode_with_host_and_capabilities_and_config(
        &entry_token,
        host,
        manifest,
        semantic_language::runtime_core::ExecutionConfig::for_context(
            semantic_language::runtime_core::ExecutionContext::KernelBound,
        ),
    )
}

fn boundary_program() -> Vec<IrFunction> {
    vec![IrFunction {
        name: "main".to_string(),
        instrs: vec![
            IrInstr::GateRead {
                dst: 0,
                device_id: 7,
                port: 3,
            },
            IrInstr::GateWrite {
                device_id: 7,
                port: 4,
                src: 0,
            },
            IrInstr::PulseEmit {
                signal: "pulse.alpha".to_string(),
            },
            IrInstr::Ret { src: None },
        ],
        ownership_events: Vec::new(),
    }]
}

#[test]
fn host_effects_route_through_prometheus_boundary() {
    let bytes = emit_ir_to_semcode(&boundary_program(), false).expect("emit gate surface semcode");
    let mut host = RecordingHostAbi::with_read_value(AbiValue::I32(42));
    let manifest = CapabilityManifest::gate_surface();

    run_token_first_with_host(&bytes, &mut host, &manifest).expect("run through prom boundary");

    assert_eq!(host.reads, vec![(7, 3)]);
    assert_eq!(host.writes, vec![(7, 4, AbiValue::I32(42))]);
    assert_eq!(host.pulses, vec!["pulse.alpha".to_string()]);
}

#[test]
fn missing_capability_blocks_host_effect_before_dispatch() {
    let bytes = emit_ir_to_semcode(&boundary_program(), false).expect("emit gate surface semcode");
    let mut host = RecordingHostAbi::with_read_value(AbiValue::I32(42));
    let mut manifest = CapabilityManifest::new();
    manifest.allow(CapabilityKind::GateRead);
    manifest.allow(CapabilityKind::GateWrite);

    let err = run_token_first_with_host(&bytes, &mut host, &manifest)
        .expect_err("pulse capability should block dispatch");

    match err {
        RuntimeError::CapabilityDenied(denied) => {
            assert_eq!(denied.capability, CapabilityKind::PulseEmit);
            assert_eq!(denied.call, Some(HostCallId::PulseEmit));
            assert_eq!(denied.manifest.schema, "prom.cap.manifest");
        }
        other => panic!("expected capability denial, got {other:?}"),
    }

    assert_eq!(host.reads, vec![(7, 3)]);
    assert_eq!(host.writes, vec![(7, 4, AbiValue::I32(42))]);
    assert!(host.pulses.is_empty(), "pulse dispatch must be blocked");
}
