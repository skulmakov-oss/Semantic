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
use semantic_language::runtime_core::ExecutionContext;

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

#[test]
fn runtime_validation_matrix_core_flow_is_deterministic_and_owner_clean() {
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
    assert_eq!(session.descriptor().context, ExecutionContext::KernelBound);
    assert!(session.descriptor().gate_registry_bound);
    assert_eq!(session.descriptor().capability_manifest, metadata);

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
    assert_eq!(advance.snapshot.state_epoch.0, 1);
    assert_eq!(advance.snapshot.active_rules, 1);
    assert_eq!(advance.agenda.entries().len(), 1);

    let activation = session
        .select_next_activation(&advance.agenda)
        .expect("activation");
    assert_eq!(activation.entry.rule_id.0, "rule.alpha");
    assert_eq!(activation.entry.salience.0, 9);
    assert_eq!(activation.remaining_rules, 0);

    session.record_session_started(&mut audit, "main");
    session.record_rule_activation(&mut audit, &activation);
    session.record_session_finished(&mut audit);

    // Intentional byte-shim compatibility coverage: this test protects the public run_verified_semcode* API surface.
    session
        .run_verified_semcode(&bytes)
        .expect("run verified runtime flow");

    let events = audit.events();
    assert_eq!(events.len(), 4);
    assert!(matches!(
        &events[0].kind,
        AuditEventKind::StateTransition {
            key,
            from_epoch,
            to_epoch
        } if key == "fact.alpha" && *from_epoch == 0 && *to_epoch == 1
    ));
    assert!(matches!(
        &events[1].kind,
        AuditEventKind::SessionStarted { entry } if entry == "main"
    ));
    assert!(matches!(
        &events[2].kind,
        AuditEventKind::RuleActivated { rule_id, salience }
            if rule_id == "rule.alpha" && *salience == 9
    ));
    assert!(matches!(&events[3].kind, AuditEventKind::SessionFinished));

    drop(session);
    assert_eq!(binding.writes(), &[(GateId::new(7, 4), AbiValue::I32(41))]);
}
