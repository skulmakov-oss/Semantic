use semantic_language::frontend::{emit_ir_to_semcode, IrFunction, IrInstr};
use semantic_language::prom_abi::AbiValue;
use semantic_language::prom_cap::{CapabilityKind, CapabilityManifest};
use semantic_language::prom_gates::{
    DeterministicGateMock, GateDescriptor, GateHostAdapter, GateId, GateRegistry,
};
use semantic_language::semcode_vm::{
    run_verified_semcode_with_host_and_capabilities, RuntimeError,
};

fn gate_rw_program(write_port: u16) -> Vec<IrFunction> {
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
                port: write_port,
                src: 0,
            },
            IrInstr::Ret { src: None },
        ],
        ownership_events: Vec::new(),
        params: Vec::new(),
    }]
}

#[test]
fn gate_registry_adapter_routes_vm_gate_ops() {
    let bytes = emit_ir_to_semcode(&gate_rw_program(4), false).expect("emit gate semcode");

    let mut registry = GateRegistry::new();
    registry
        .register(GateDescriptor::read_only(7, 3, "sensor.alpha"))
        .expect("register read gate");
    registry
        .register(GateDescriptor::read_write(7, 4, "actuator.beta"))
        .expect("register write gate");

    let mut binding = DeterministicGateMock::new();
    binding.seed_read(GateId::new(7, 3), AbiValue::I32(55));
    let manifest = CapabilityManifest::gate_surface();
    {
        let mut host = GateHostAdapter::new(&registry, &mut binding);
        run_verified_semcode_with_host_and_capabilities(&bytes, &mut host, &manifest)
            .expect("run through gate adapter");
    }

    assert_eq!(binding.writes(), &[(GateId::new(7, 4), AbiValue::I32(55))]);
}

#[test]
fn gate_registry_blocks_invalid_write_before_backend_dispatch() {
    let bytes = emit_ir_to_semcode(&gate_rw_program(3), false).expect("emit gate semcode");

    let mut registry = GateRegistry::new();
    registry
        .register(GateDescriptor::read_only(7, 3, "sensor.alpha"))
        .expect("register read-only gate");

    let mut binding = DeterministicGateMock::new();
    binding.seed_read(GateId::new(7, 3), AbiValue::I32(55));
    let mut manifest = CapabilityManifest::new();
    manifest.allow(CapabilityKind::GateRead);
    manifest.allow(CapabilityKind::GateWrite);
    let err = {
        let mut host = GateHostAdapter::new(&registry, &mut binding);
        run_verified_semcode_with_host_and_capabilities(&bytes, &mut host, &manifest)
            .expect_err("read-only gate should reject write")
    };

    match err {
        RuntimeError::HostAbi(err) => {
            assert!(err.message.contains("does not allow writes"));
        }
        other => panic!("expected host abi error, got {other:?}"),
    }

    assert!(
        binding.writes().is_empty(),
        "backend write dispatch must be blocked"
    );
}
