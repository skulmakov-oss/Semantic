#![cfg_attr(not(feature = "std"), no_std)]

use prom_ui::action_admission::{
    InteractionActionAdmissionDescriptorId, InteractionActionAdmissionEffectRelationship,
    InteractionActionAdmissionPolicyGateNamespace, InteractionActionAdmissionTraceRequirement,
};
use prom_ui::{
    InteractionActionAdmissionResultId, InteractionActionBindingId, InteractionActionName,
    InteractionAdmittedSemanticAction, InteractionAdmittedSemanticActionDispatchReadiness,
    InteractionAdmittedSemanticActionEffectReadiness, InteractionAdmittedSemanticActionId,
    InteractionIntentKind,
};
use prom_ui_runtime::state_update::{InertStateUpdater, RuntimeStateUpdater};

#[test]
fn inert_state_updater_consumes_admitted_action() {
    let updater = InertStateUpdater::new();

    let admitted_action = InteractionAdmittedSemanticAction {
        id: InteractionAdmittedSemanticActionId(1),
        admission_result_id: InteractionActionAdmissionResultId(1),
        descriptor_id: InteractionActionAdmissionDescriptorId(1),
        action: InteractionActionName::Unknown,
        source_intent: InteractionIntentKind::Unknown,
        binding_id: InteractionActionBindingId(1),
        trace_requirement: InteractionActionAdmissionTraceRequirement::Required,
        effect_relationship: InteractionActionAdmissionEffectRelationship::NoEffect,
        policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace::CoreUi,
        dispatch_readiness: InteractionAdmittedSemanticActionDispatchReadiness::Deferred,
        effect_readiness: InteractionAdmittedSemanticActionEffectReadiness::NoEffect,
    };

    let result = updater.execute_admitted_action(admitted_action);

    // Validates that the inert state updater successfully does nothing.
    assert_eq!(result, Ok(()));
}
