use semantic_language::frontend::{emit_ir_to_semcode, IrFunction, IrInstr};
use semantic_language::prom_abi::{AbiValue, HostCallId, RecordingHostAbi};
use semantic_language::prom_cap::{CapabilityKind, CapabilityManifest};
use semantic_language::prom_gates::{DeterministicGateMock, GateDescriptor, GateId, GateRegistry};
use semantic_language::prom_rules::{RuleCondition, RuleDefinition, RuleEngine};
use semantic_language::prom_runtime::GateExecutionSession;
use semantic_language::prom_state::{
    ContextWindow, FactResolution, FactValue, SemanticStateStore, StateUpdate,
};
use semantic_language::semcode_vm::{
    run_verified_semcode_with_host_and_capabilities, RuntimeError,
};

fn read_text(path: &str) -> String {
    let raw = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read '{}': {}", path, e));
    normalize_newlines(&raw)
}

fn write_text(path: &str, text: &str) {
    std::fs::write(path, text).unwrap_or_else(|e| panic!("write '{}': {}", path, e))
}

fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n")
}

fn update_mode() -> bool {
    std::env::var("SM_UPDATE_SNAPSHOTS")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn assert_snapshot(path: &str, got: &str) {
    if update_mode() {
        write_text(path, got);
        return;
    }
    let expected = normalize_newlines(&read_text(path));
    let got = normalize_newlines(got);
    assert_eq!(expected, got, "snapshot mismatch at {}", path);
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
        params: Vec::new(),
    }]
}

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

fn render_capability_denial_baseline() -> String {
    let bytes = emit_ir_to_semcode(&boundary_program(), false).expect("emit");
    let mut host = RecordingHostAbi::with_read_value(AbiValue::I32(42));
    let mut manifest = CapabilityManifest::new();
    manifest.allow(CapabilityKind::GateRead);
    manifest.allow(CapabilityKind::GateWrite);

    let err = run_verified_semcode_with_host_and_capabilities(&bytes, &mut host, &manifest)
        .expect_err("pulse capability should block dispatch");

    match err {
        RuntimeError::CapabilityDenied(denied) => {
            format!(
                "error=CapabilityDenied\ncapability={:?}\ncall={:?}\ncode={:?}\nschema={}\nversion={:?}\nreads={:?}\nwrites={:?}\npulses={:?}\n",
                denied.capability,
                denied.call.unwrap_or(HostCallId::PulseEmit),
                denied.code,
                denied.manifest.schema,
                denied.manifest.version,
                host.reads,
                host.writes,
                host.pulses
            )
        }
        other => panic!("expected capability denial, got {other:?}"),
    }
}

fn render_gate_write_denial_baseline() -> String {
    let bytes = emit_ir_to_semcode(&gate_rw_program(3), false).expect("emit");

    let mut registry = GateRegistry::new();
    registry
        .register(GateDescriptor::read_only(7, 3, "sensor.alpha"))
        .expect("register");

    let mut binding = DeterministicGateMock::new();
    binding.seed_read(GateId::new(7, 3), AbiValue::I32(55));
    let manifest = CapabilityManifest::gate_surface();

    let err = {
        let mut session = GateExecutionSession::kernel_bound(
            &registry,
            &mut binding,
            &manifest,
            manifest.metadata(),
        );
        session
            .run_verified_semcode(&bytes)
            .expect_err("read-only gate write must fail")
    };

    match err {
        RuntimeError::HostAbi(err) => format!(
            "error=HostAbi\ncall={:?}\nkind={:?}\nmessage={}\nwrites={:?}\n",
            err.call,
            err.kind,
            err.message,
            binding.writes()
        ),
        other => panic!("expected host abi error, got {other:?}"),
    }
}

fn render_state_validation_rejection_baseline() -> String {
    let manifest = CapabilityManifest::gate_surface();
    let metadata = manifest.metadata();
    let mut registry = GateRegistry::new();
    registry
        .register(GateDescriptor::read_write(7, 4, "gate.alpha"))
        .expect("register");
    let mut binding = DeterministicGateMock::new();
    binding.seed_read(GateId::new(7, 4), AbiValue::I32(1));

    let session =
        GateExecutionSession::kernel_bound(&registry, &mut binding, &manifest, metadata.clone());
    let mut state = SemanticStateStore::new();
    let mut rules = RuleEngine::new();
    rules
        .register(RuleDefinition::new(
            "rule.alpha",
            5,
            vec![RuleCondition::equals("fact.alpha", FactValue::Bool(true))],
        ))
        .expect("register");
    let mut audit = session.begin_audit_trail();

    let err = session
        .apply_state_update_and_refresh_agenda(
            &mut state,
            StateUpdate::new(
                "fact.alpha",
                FactResolution::Certain(FactValue::Bool(true)),
                ContextWindow::new(""),
                "",
            ),
            &rules,
            &mut audit,
        )
        .expect_err("invalid state update must reject");

    format!(
        "error=StateValidation\ncode={:?}\nmessage={}\nstate_epoch={}\naudit_events={}\nmanifest={}@{:?}\n",
        err.code,
        err.message,
        state.epoch().0,
        audit.events().len(),
        metadata.schema,
        metadata.version
    )
}

#[test]
fn golden_capability_denial_runtime_snapshot() {
    let got = render_capability_denial_baseline();
    assert_snapshot("tests/golden_snapshots/runtime/capability_denial.txt", &got);
}

#[test]
fn golden_gate_write_denial_runtime_snapshot() {
    let got = render_gate_write_denial_baseline();
    assert_snapshot("tests/golden_snapshots/runtime/gate_write_denial.txt", &got);
}

#[test]
fn golden_state_validation_rejection_runtime_snapshot() {
    let got = render_state_validation_rejection_baseline();
    assert_snapshot(
        "tests/golden_snapshots/runtime/state_validation_rejection.txt",
        &got,
    );
}
