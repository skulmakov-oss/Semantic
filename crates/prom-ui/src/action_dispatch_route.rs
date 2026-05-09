//! Semantic UI action dispatch route scaffold.
//!
//! This module records inert route metadata for a future dispatcher. It does
//! not implement dispatch execution, dispatch records, effect requests,
//! VM/Host ABI calls, or runtime mutation.

use crate::action_admission::{
    InteractionActionAdmissionEffectRelationship,
    InteractionActionAdmissionPolicyGateNamespace, InteractionActionAdmissionTraceRequirement,
};
use crate::action_binding::{InteractionActionBindingId, InteractionActionName};
use crate::admitted_action::{
    InteractionAdmittedSemanticAction, InteractionAdmittedSemanticActionEffectReadiness,
    InteractionAdmittedSemanticActionId,
};
use crate::interaction::InteractionIntentKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InteractionSemanticActionDispatchRouteId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionSemanticActionDispatchRouteKind {
    NoopSemanticRecord,
    LocalUiStateCandidate,
    EffectRequestCandidate,
    WorkbenchLocalCandidate,
    DiagnosticOnlyCandidate,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionSemanticActionDispatchEffectEligibility {
    NoEffect,
    RequiresFutureEffectBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionSemanticActionDispatchRouteDescriptor {
    pub id: InteractionSemanticActionDispatchRouteId,
    pub admitted_action_id: InteractionAdmittedSemanticActionId,
    pub action: InteractionActionName,
    pub source_intent: InteractionIntentKind,
    pub binding_id: InteractionActionBindingId,
    pub route: InteractionSemanticActionDispatchRouteKind,
    pub trace_requirement: InteractionActionAdmissionTraceRequirement,
    pub effect_relationship: InteractionActionAdmissionEffectRelationship,
    pub policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace,
    pub effect_eligibility: InteractionSemanticActionDispatchEffectEligibility,
}

pub fn describe_interaction_semantic_action_dispatch_route(
    action: &InteractionAdmittedSemanticAction,
) -> InteractionSemanticActionDispatchRouteDescriptor {
    InteractionSemanticActionDispatchRouteDescriptor {
        id: InteractionSemanticActionDispatchRouteId(action.id.0),
        admitted_action_id: action.id,
        action: action.action,
        source_intent: action.source_intent,
        binding_id: action.binding_id,
        route: map_dispatch_route(
            action.action,
            action.policy_gate_namespace,
            action.effect_readiness,
        ),
        trace_requirement: action.trace_requirement,
        effect_relationship: action.effect_relationship,
        policy_gate_namespace: action.policy_gate_namespace,
        effect_eligibility: map_effect_eligibility(action.effect_readiness),
    }
}

const fn map_effect_eligibility(
    readiness: InteractionAdmittedSemanticActionEffectReadiness,
) -> InteractionSemanticActionDispatchEffectEligibility {
    match readiness {
        InteractionAdmittedSemanticActionEffectReadiness::NoEffect => {
            InteractionSemanticActionDispatchEffectEligibility::NoEffect
        }
        InteractionAdmittedSemanticActionEffectReadiness::DeferredEffectBoundary => {
            InteractionSemanticActionDispatchEffectEligibility::RequiresFutureEffectBoundary
        }
    }
}

const fn map_dispatch_route(
    action: InteractionActionName,
    policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace,
    effect_readiness: InteractionAdmittedSemanticActionEffectReadiness,
) -> InteractionSemanticActionDispatchRouteKind {
    match effect_readiness {
        InteractionAdmittedSemanticActionEffectReadiness::DeferredEffectBoundary => {
            InteractionSemanticActionDispatchRouteKind::EffectRequestCandidate
        }
        InteractionAdmittedSemanticActionEffectReadiness::NoEffect => match policy_gate_namespace {
            InteractionActionAdmissionPolicyGateNamespace::WorkbenchLocal => {
                InteractionSemanticActionDispatchRouteKind::WorkbenchLocalCandidate
            }
            InteractionActionAdmissionPolicyGateNamespace::DiagnosticOnly => {
                InteractionSemanticActionDispatchRouteKind::DiagnosticOnlyCandidate
            }
            InteractionActionAdmissionPolicyGateNamespace::CoreUi => {
                map_core_ui_no_effect_route(action)
            }
        },
    }
}

const fn map_core_ui_no_effect_route(
    action: InteractionActionName,
) -> InteractionSemanticActionDispatchRouteKind {
    match action {
        InteractionActionName::CloseWindow
        | InteractionActionName::SelectTraceEvent
        | InteractionActionName::FocusModule
        | InteractionActionName::AcknowledgeError
        | InteractionActionName::PinTrace
        | InteractionActionName::OpenDenialReason => {
            InteractionSemanticActionDispatchRouteKind::LocalUiStateCandidate
        }
        InteractionActionName::OpenInspector => {
            InteractionSemanticActionDispatchRouteKind::WorkbenchLocalCandidate
        }
        InteractionActionName::ExpandCapabilityGate
        | InteractionActionName::PrepareEffect
        | InteractionActionName::CommitEffect
        | InteractionActionName::RollbackEffect
        | InteractionActionName::QuarantineTarget => {
            InteractionSemanticActionDispatchRouteKind::EffectRequestCandidate
        }
        InteractionActionName::Unknown => InteractionSemanticActionDispatchRouteKind::Unknown,
    }
}

impl InteractionSemanticActionDispatchRouteDescriptor {
    pub const fn is_dispatch_record(&self) -> bool {
        false
    }

    pub const fn is_execution_authority(&self) -> bool {
        false
    }

    pub const fn is_effect_request(&self) -> bool {
        false
    }

    pub const fn is_runtime_mutation(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action_admission::{
        InteractionActionAdmissionDescriptorId, InteractionActionAdmissionPolicyGateNamespace,
        InteractionActionAdmissionTraceRequirement,
    };
    use crate::action_admission_result::InteractionActionAdmissionResultId;
    use crate::admitted_action::{
        InteractionAdmittedSemanticAction,
        InteractionAdmittedSemanticActionDispatchReadiness,
        InteractionAdmittedSemanticActionEffectReadiness,
        InteractionAdmittedSemanticActionId,
    };

    fn close_admitted_action() -> InteractionAdmittedSemanticAction {
        InteractionAdmittedSemanticAction {
            id: InteractionAdmittedSemanticActionId(1),
            admission_result_id: InteractionActionAdmissionResultId(1),
            descriptor_id: InteractionActionAdmissionDescriptorId(1),
            action: InteractionActionName::CloseWindow,
            source_intent: InteractionIntentKind::Close,
            binding_id: InteractionActionBindingId(1),
            trace_requirement: InteractionActionAdmissionTraceRequirement::Required,
            effect_relationship: InteractionActionAdmissionEffectRelationship::NoEffect,
            policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace::CoreUi,
            dispatch_readiness: InteractionAdmittedSemanticActionDispatchReadiness::Deferred,
            effect_readiness: InteractionAdmittedSemanticActionEffectReadiness::NoEffect,
        }
    }

    fn select_admitted_action() -> InteractionAdmittedSemanticAction {
        InteractionAdmittedSemanticAction {
            id: InteractionAdmittedSemanticActionId(2),
            admission_result_id: InteractionActionAdmissionResultId(2),
            descriptor_id: InteractionActionAdmissionDescriptorId(2),
            action: InteractionActionName::SelectTraceEvent,
            source_intent: InteractionIntentKind::Select,
            binding_id: InteractionActionBindingId(2),
            trace_requirement: InteractionActionAdmissionTraceRequirement::Required,
            effect_relationship: InteractionActionAdmissionEffectRelationship::NoEffect,
            policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace::CoreUi,
            dispatch_readiness: InteractionAdmittedSemanticActionDispatchReadiness::Deferred,
            effect_readiness: InteractionAdmittedSemanticActionEffectReadiness::NoEffect,
        }
    }

    #[test]
    fn close_window_maps_to_local_ui_state_candidate() {
        let action = close_admitted_action();

        let route = describe_interaction_semantic_action_dispatch_route(&action);

        assert_eq!(
            route.route,
            InteractionSemanticActionDispatchRouteKind::LocalUiStateCandidate
        );
        assert_eq!(
            route.effect_eligibility,
            InteractionSemanticActionDispatchEffectEligibility::NoEffect
        );
    }

    #[test]
    fn workbench_local_maps_to_workbench_candidate() {
        let mut action = select_admitted_action();
        action.policy_gate_namespace = InteractionActionAdmissionPolicyGateNamespace::WorkbenchLocal;

        let route = describe_interaction_semantic_action_dispatch_route(&action);

        assert_eq!(
            route.route,
            InteractionSemanticActionDispatchRouteKind::WorkbenchLocalCandidate
        );
    }

    #[test]
    fn deferred_effect_maps_to_effect_request_candidate() {
        let mut action = select_admitted_action();
        action.effect_readiness =
            InteractionAdmittedSemanticActionEffectReadiness::DeferredEffectBoundary;

        let route = describe_interaction_semantic_action_dispatch_route(&action);

        assert_eq!(
            route.route,
            InteractionSemanticActionDispatchRouteKind::EffectRequestCandidate
        );
        assert_eq!(
            route.effect_eligibility,
            InteractionSemanticActionDispatchEffectEligibility::RequiresFutureEffectBoundary
        );
    }

    #[test]
    fn route_descriptor_preserves_identity() {
        let action = close_admitted_action();

        let route = describe_interaction_semantic_action_dispatch_route(&action);

        assert_eq!(route.admitted_action_id, action.id);
        assert_eq!(route.action, action.action);
        assert_eq!(route.source_intent, action.source_intent);
        assert_eq!(route.binding_id, action.binding_id);
    }

    #[test]
    fn route_descriptor_is_not_authority() {
        let action = close_admitted_action();

        let route = describe_interaction_semantic_action_dispatch_route(&action);

        assert!(!route.is_dispatch_record());
        assert!(!route.is_execution_authority());
        assert!(!route.is_effect_request());
        assert!(!route.is_runtime_mutation());
    }

    #[test]
    fn route_generation_is_deterministic() {
        let action = select_admitted_action();

        let first = describe_interaction_semantic_action_dispatch_route(&action);
        let second = describe_interaction_semantic_action_dispatch_route(&action);

        assert_eq!(first, second);
    }
}
