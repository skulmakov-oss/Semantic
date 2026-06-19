use prom_ui::effect_request::{
    InteractionEffectRequestDescriptor, InteractionEffectRequestDescriptorId,
    InteractionEffectRequestKind, InteractionEffectRequestTargetPolicy,
    InteractionEffectRequestUiCapability, InteractionEffectRequestRuntimeCapability,
    InteractionEffectRequestLifecyclePrecondition, InteractionEffectRequestDenialBehavior,
    InteractionEffectRequestPrepareCommitRelationship, InteractionEffectRequestScope,
};
use prom_ui::action_admission::{InteractionActionAdmissionPolicyGateNamespace, InteractionActionAdmissionTraceRequirement};
use prom_ui::action_binding::InteractionActionName;
use prom_ui::action_dispatch_record::InteractionSemanticActionDispatchRecordId;
use prom_ui::action_dispatch_route::InteractionSemanticActionDispatchRouteId;
use prom_ui::admitted_action::InteractionAdmittedSemanticActionId;
use prom_ui::interaction::InteractionIntentKind;

fn dummy_effect_request() -> InteractionEffectRequestDescriptor {
    InteractionEffectRequestDescriptor {
        id: InteractionEffectRequestDescriptorId(1),
        source_admitted_action_id: InteractionAdmittedSemanticActionId(1),
        dispatch_record_id: InteractionSemanticActionDispatchRecordId(1),
        dispatch_route_id: InteractionSemanticActionDispatchRouteId(1),
        action: InteractionActionName::PrepareEffect,
        source_intent: InteractionIntentKind::Submit,
        requested_effect: InteractionEffectRequestKind::PrepareEffect,
        target_policy: InteractionEffectRequestTargetPolicy::Required,
        required_ui_capability: InteractionEffectRequestUiCapability::EffectControl,
        required_runtime_capability: InteractionEffectRequestRuntimeCapability::EffectControl,
        lifecycle_precondition: InteractionEffectRequestLifecyclePrecondition::SessionActive,
        trace_requirement: InteractionActionAdmissionTraceRequirement::Required,
        denial_behavior: InteractionEffectRequestDenialBehavior::VisibleRequired,
        prepare_commit_relationship: InteractionEffectRequestPrepareCommitRelationship::PrepareThenCommit,
        policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace::CoreUi,
        scope: InteractionEffectRequestScope::CoreUi,
    }
}

#[test]
fn effect_request_is_not_execution() {
    let request = dummy_effect_request();

    // Effect requests are purely declarative metadata.
    // They do not execute anything. They are handed off to an Effect Boundary.
    assert!(!request.is_execution_authority());
    assert!(!request.is_runtime_mutation());
    assert!(!request.calls_vm_or_host_abi());

    // By design, `InteractionEffectRequestDescriptor` does not have `execute()` or `apply()` methods.
    // It is inert data waiting for an Effect Dispatcher.
}
