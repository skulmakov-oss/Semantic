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

#[test]
fn runtime_composes_state_rules_and_audit_without_taking_ownership() {
    let manifest = CapabilityManifest::gate_surface();
    let metadata = manifest.metadata();

    let mut registry = GateRegistry::new();
    registry
        .register(GateDescriptor::read_write(7, 4, "gate.alpha"))
        .expect("register");
    let mut binding = DeterministicGateMock::new();
    binding.seed_read(GateId::new(7, 4), AbiValue::I32(1));

    let session = GateExecutionSession::kernel_bound(&registry, &mut binding, &manifest);

    let mut state = SemanticStateStore::new();
    state
        .apply(StateUpdate::new(
            "fact.alpha",
            FactResolution::Certain(FactValue::Bool(true)),
            ContextWindow::new("root"),
            "seed alpha",
        ))
        .expect("seed");

    let mut rules = RuleEngine::new();
    rules
        .register(RuleDefinition::new(
            "rule.alpha",
            7,
            vec![RuleCondition::equals("fact.alpha", FactValue::Bool(true))],
        ))
        .expect("register");

    let agenda = session.derive_agenda(&state, &rules);
    assert_eq!(agenda.entries().len(), 1);
    let activation = session.select_next_activation(&agenda).expect("activation");
    assert_eq!(activation.entry.rule_id.0, "rule.alpha");
    assert_eq!(activation.remaining_rules, 0);

    let mut audit = session.begin_audit_trail();
    session
        .record_session_started(&mut audit, "main")
        .expect("audit identity available");
    session
        .record_rule_activation(&mut audit, &activation)
        .expect("audit identity available");
    let transition = state.transitions().last().expect("transition");
    session
        .record_state_transition(&mut audit, transition)
        .expect("audit identity available");
    session
        .record_session_finished(&mut audit)
        .expect("audit identity available");
    let snapshot = session.integration_snapshot(&state, &agenda);

    assert_eq!(snapshot.session.context, ExecutionContext::KernelBound);
    assert_eq!(snapshot.session.capability_manifest, metadata);
    assert!(snapshot.session.gate_registry_bound);
    assert_eq!(snapshot.state_epoch.0, 1);
    assert_eq!(snapshot.active_rules, 1);
    assert_eq!(audit.replay_metadata().event_count, 4);
    assert!(matches!(
        &audit.events()[1].kind,
        AuditEventKind::RuleActivated { rule_id, salience }
            if rule_id == "rule.alpha" && *salience == 7
    ));
    assert!(matches!(
        &audit.events()[2].kind,
        AuditEventKind::StateTransition {
            key,
            from_epoch,
            to_epoch
        } if key == "fact.alpha" && *from_epoch == 0 && *to_epoch == 1
    ));
}

#[test]
fn runtime_applies_state_update_and_refreshes_agenda_through_owner_layers() {
    let manifest = CapabilityManifest::gate_surface();
    let metadata = manifest.metadata();

    let mut registry = GateRegistry::new();
    registry
        .register(GateDescriptor::read_write(7, 4, "gate.alpha"))
        .expect("register");
    let mut binding = DeterministicGateMock::new();
    binding.seed_read(GateId::new(7, 4), AbiValue::I32(1));

    let session = GateExecutionSession::kernel_bound(&registry, &mut binding, &manifest);

    let mut state = SemanticStateStore::new();
    let mut rules = RuleEngine::new();
    rules
        .register(RuleDefinition::new(
            "rule.alpha",
            7,
            vec![RuleCondition::equals("fact.alpha", FactValue::Bool(true))],
        ))
        .expect("register");

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
        .expect("advance");

    assert_eq!(advance.transition.key, "fact.alpha");
    assert_eq!(advance.snapshot.session.capability_manifest, metadata);
    assert_eq!(advance.snapshot.state_epoch.0, 1);
    assert_eq!(advance.snapshot.active_rules, 1);
    assert_eq!(advance.agenda.entries().len(), 1);
    assert!(matches!(
        &audit.events()[0].kind,
        AuditEventKind::StateTransition {
            key,
            from_epoch,
            to_epoch
        } if key == "fact.alpha" && *from_epoch == 0 && *to_epoch == 1
    ));
}
