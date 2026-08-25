use semantic_language::frontend::{emit_ir_to_semcode, IrFunction, IrInstr};
use semantic_language::prom_abi::AbiValue;
use semantic_language::prom_audit::AuditEventKind;
use semantic_language::prom_cap::CapabilityManifest;
use semantic_language::prom_gates::{DeterministicGateMock, GateDescriptor, GateId, GateRegistry};
use semantic_language::prom_rules::{RuleCondition, RuleDefinition, RuleEngine};
use semantic_language::prom_runtime::GateExecutionSession;
use semantic_language::prom_state::{
    ContextWindow, FactResolution, FactValue, SemanticStateStore, StateUpdate,
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

fn gate_program() -> Vec<IrFunction> {
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
            IrInstr::Ret { src: None },
        ],
        ownership_events: Vec::new(),
        params: Vec::new(),
    }]
}

fn render_runtime_baseline() -> String {
    let bytes = emit_ir_to_semcode(&gate_program(), false).expect("emit");

    let mut registry = GateRegistry::new();
    registry
        .register(GateDescriptor::read_only(7, 3, "sensor.alpha"))
        .expect("register read gate");
    registry
        .register(GateDescriptor::read_write(7, 4, "actuator.beta"))
        .expect("register write gate");

    let manifest = CapabilityManifest::gate_surface();
    let metadata = manifest.metadata();
    let mut binding = DeterministicGateMock::new();
    binding.seed_read(GateId::new(7, 3), AbiValue::I32(41));

    let mut session =
        GateExecutionSession::kernel_bound(&registry, &mut binding, &manifest, metadata.clone());

    let mut state = SemanticStateStore::new();
    let mut rules = RuleEngine::new();
    rules
        .register(RuleDefinition::new(
            "rule.alpha",
            9,
            vec![RuleCondition::equals("fact.alpha", FactValue::Bool(true))],
        ))
        .expect("register rule");

    let mut audit = session.begin_audit_trail();
    let advance = session
        .apply_state_update_and_refresh_agenda(
            &mut state,
            StateUpdate::new(
                "fact.alpha",
                FactResolution::Certain(FactValue::Bool(true)),
                ContextWindow::new("root"),
                "seed alpha",
            ),
            &rules,
            &mut audit,
        )
        .expect("advance state");
    let activation = session
        .select_next_activation(&advance.agenda)
        .expect("activation");

    session.record_session_started(&mut audit, "main");
    session.record_rule_activation(&mut audit, &activation);
    session.record_session_finished(&mut audit);
    // Intentional byte-shim compatibility coverage: this test protects the public run_verified_semcode* API surface.
    session
        .run_verified_semcode(&bytes)
        .expect("run verified runtime flow");
    drop(session);

    let mut out = String::new();
    out.push_str(&format!(
        "context={:?}\nmanifest={}@{:?}\ngate_registry_bound={}\n",
        advance.snapshot.session.context,
        advance.snapshot.session.capability_manifest.schema,
        advance.snapshot.session.capability_manifest.version,
        advance.snapshot.session.gate_registry_bound
    ));
    out.push_str(&format!(
        "state_epoch={}\nactive_rules={}\nactivation={} salience={} remaining={}\n",
        advance.snapshot.state_epoch.0,
        advance.snapshot.active_rules,
        activation.entry.rule_id.0,
        activation.entry.salience.0,
        activation.remaining_rules
    ));
    out.push_str("events:\n");
    for event in audit.events() {
        match &event.kind {
            AuditEventKind::StateTransition {
                key,
                from_epoch,
                to_epoch,
            } => {
                out.push_str(&format!(
                    "  {} StateTransition {} {}->{}\n",
                    event.id.0, key, from_epoch, to_epoch
                ));
            }
            AuditEventKind::SessionStarted { entry } => {
                out.push_str(&format!("  {} SessionStarted {}\n", event.id.0, entry));
            }
            AuditEventKind::RuleActivated { rule_id, salience } => {
                out.push_str(&format!(
                    "  {} RuleActivated {} salience={}\n",
                    event.id.0, rule_id, salience
                ));
            }
            AuditEventKind::SessionFinished => {
                out.push_str(&format!("  {} SessionFinished\n", event.id.0));
            }
            other => panic!("unexpected audit event in baseline: {other:?}"),
        }
    }
    out.push_str("writes:\n");
    for (gate, value) in binding.writes() {
        out.push_str(&format!("  {}:{}={:?}\n", gate.device_id, gate.port, value));
    }
    out
}

#[test]
fn golden_semantic_runtime_flow_snapshot() {
    let got = render_runtime_baseline();
    assert_snapshot(
        "tests/golden_snapshots/runtime/semantic_runtime_flow.txt",
        &got,
    );
}
