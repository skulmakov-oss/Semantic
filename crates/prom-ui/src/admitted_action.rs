//! Semantic UI admitted semantic action object scaffold.
//!
//! This module records a typed inert representation of an admitted UI-level
//! operation. It does not implement dispatch, execution, effect requests,
//! VM/Host ABI calls, or runtime mutation.

use crate::action_admission::{
    InteractionActionAdmissionDescriptorId, InteractionActionAdmissionEffectRelationship,
    InteractionActionAdmissionPolicyGateNamespace, InteractionActionAdmissionTraceRequirement,
};
use crate::action_admission_result::{
    InteractionActionAdmissionResult, InteractionActionAdmissionResultId,
};
use crate::action_binding::{InteractionActionBindingId, InteractionActionName};
use crate::interaction::InteractionIntentKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InteractionAdmittedSemanticActionId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionAdmittedSemanticActionDispatchReadiness {
    Deferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionAdmittedSemanticActionEffectReadiness {
    NoEffect,
    DeferredEffectBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionAdmittedSemanticAction {
    pub id: InteractionAdmittedSemanticActionId,
    pub admission_result_id: InteractionActionAdmissionResultId,
    pub descriptor_id: InteractionActionAdmissionDescriptorId,
    pub action: InteractionActionName,
    pub source_intent: InteractionIntentKind,
    pub binding_id: InteractionActionBindingId,
    pub trace_requirement: InteractionActionAdmissionTraceRequirement,
    pub effect_relationship: InteractionActionAdmissionEffectRelationship,
    pub policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace,
    pub dispatch_readiness: InteractionAdmittedSemanticActionDispatchReadiness,
    pub effect_readiness: InteractionAdmittedSemanticActionEffectReadiness,
}

pub fn build_interaction_admitted_semantic_action(
    result: &InteractionActionAdmissionResult,
) -> Option<InteractionAdmittedSemanticAction> {
    if !result.is_admitted() {
        return None;
    }

    Some(InteractionAdmittedSemanticAction {
        id: InteractionAdmittedSemanticActionId(result.id.0),
        admission_result_id: result.id,
        descriptor_id: result.descriptor_id,
        action: result.action,
        source_intent: result.source_intent,
        binding_id: result.binding_id,
        trace_requirement: result.trace_requirement,
        effect_relationship: result.effect_relationship,
        policy_gate_namespace: result.policy_gate_namespace,
        dispatch_readiness: InteractionAdmittedSemanticActionDispatchReadiness::Deferred,
        effect_readiness: map_effect_readiness(result.effect_relationship),
    })
}

const fn map_effect_readiness(
    relationship: InteractionActionAdmissionEffectRelationship,
) -> InteractionAdmittedSemanticActionEffectReadiness {
    match relationship {
        InteractionActionAdmissionEffectRelationship::NoEffect => {
            InteractionAdmittedSemanticActionEffectReadiness::NoEffect
        }
        InteractionActionAdmissionEffectRelationship::MayRequestEffectAfterAdmission
        | InteractionActionAdmissionEffectRelationship::RequiresSeparateEffectAdmission => {
            InteractionAdmittedSemanticActionEffectReadiness::DeferredEffectBoundary
        }
    }
}

impl InteractionAdmittedSemanticAction {
    pub const fn is_dispatch_authority(&self) -> bool {
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
        InteractionActionAdmissionCapabilityRequirement,
        InteractionActionAdmissionDenialVisibility, InteractionActionAdmissionDescriptor,
        InteractionActionAdmissionDescriptorId, InteractionActionAdmissionEffectRelationship,
        InteractionActionAdmissionFutureResultShape,
        InteractionActionAdmissionLifecycleRequirement,
        InteractionActionAdmissionPolicyGateNamespace,
        InteractionActionAdmissionTargetOwnershipRequirement,
        InteractionActionAdmissionTargetRequirement, InteractionActionAdmissionTraceRequirement,
    };
    use crate::action_admission_result::{
        record_interaction_action_admitted_result, record_interaction_action_denied_result,
        InteractionActionAdmissionDenialReason, InteractionActionAdmissionMissingRequirement,
    };
    use crate::action_binding::{InteractionActionBindingId, InteractionActionName};
    use crate::interaction::InteractionIntentKind;

    fn close_descriptor() -> InteractionActionAdmissionDescriptor {
        InteractionActionAdmissionDescriptor {
            id: InteractionActionAdmissionDescriptorId(1),
            action: InteractionActionName::CloseWindow,
            source_intent: InteractionIntentKind::Close,
            binding_id: InteractionActionBindingId(1),
            target_requirement: InteractionActionAdmissionTargetRequirement::Ignored,
            target_ownership_requirement:
                InteractionActionAdmissionTargetOwnershipRequirement::NotRequired,
            lifecycle_requirement: InteractionActionAdmissionLifecycleRequirement::SessionActive,
            capability_requirement: InteractionActionAdmissionCapabilityRequirement::DesktopSession,
            trace_requirement: InteractionActionAdmissionTraceRequirement::Required,
            effect_relationship: InteractionActionAdmissionEffectRelationship::NoEffect,
            denial_visibility: InteractionActionAdmissionDenialVisibility::Required,
            policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace::CoreUi,
            future_result_shape: InteractionActionAdmissionFutureResultShape::AdmitOrDenyWithReason,
        }
    }

    fn select_descriptor() -> InteractionActionAdmissionDescriptor {
        InteractionActionAdmissionDescriptor {
            id: InteractionActionAdmissionDescriptorId(2),
            action: InteractionActionName::SelectTraceEvent,
            source_intent: InteractionIntentKind::Select,
            binding_id: InteractionActionBindingId(2),
            target_requirement: InteractionActionAdmissionTargetRequirement::Required,
            target_ownership_requirement:
                InteractionActionAdmissionTargetOwnershipRequirement::RequiredWhenTargetPresent,
            lifecycle_requirement: InteractionActionAdmissionLifecycleRequirement::SessionActive,
            capability_requirement: InteractionActionAdmissionCapabilityRequirement::InputPoll,
            trace_requirement: InteractionActionAdmissionTraceRequirement::Required,
            effect_relationship: InteractionActionAdmissionEffectRelationship::NoEffect,
            denial_visibility: InteractionActionAdmissionDenialVisibility::Required,
            policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace::CoreUi,
            future_result_shape: InteractionActionAdmissionFutureResultShape::AdmitOrDenyWithReason,
        }
    }

    #[test]
    fn admitted_result_builds_admitted_action_object() {
        let descriptor = close_descriptor();
        let result = record_interaction_action_admitted_result(&descriptor);

        let action = build_interaction_admitted_semantic_action(&result)
            .expect("admitted result should build action object");

        assert_eq!(action.admission_result_id, result.id);
        assert_eq!(action.descriptor_id, result.descriptor_id);
        assert_eq!(action.action, result.action);
        assert_eq!(action.source_intent, result.source_intent);
        assert_eq!(action.binding_id, result.binding_id);
    }

    #[test]
    fn denied_result_does_not_build_admitted_action_object() {
        let descriptor = select_descriptor();

        let result = record_interaction_action_denied_result(
            &descriptor,
            InteractionActionAdmissionDenialReason::MissingTarget,
            InteractionActionAdmissionMissingRequirement::Target,
        );

        let action = build_interaction_admitted_semantic_action(&result);

        assert!(action.is_none());
    }

    #[test]
    fn admitted_action_object_is_not_authority() {
        let descriptor = close_descriptor();
        let result = record_interaction_action_admitted_result(&descriptor);

        let action = build_interaction_admitted_semantic_action(&result)
            .expect("admitted result should build action object");

        assert!(!action.is_dispatch_authority());
        assert!(!action.is_execution_authority());
        assert!(!action.is_effect_request());
        assert!(!action.is_runtime_mutation());
    }

    #[test]
    fn admitted_action_dispatch_readiness_is_deferred() {
        let descriptor = close_descriptor();
        let result = record_interaction_action_admitted_result(&descriptor);

        let action = build_interaction_admitted_semantic_action(&result)
            .expect("admitted result should build action object");

        assert_eq!(
            action.dispatch_readiness,
            InteractionAdmittedSemanticActionDispatchReadiness::Deferred
        );
    }

    #[test]
    fn no_effect_relationship_maps_to_no_effect_readiness() {
        let descriptor = close_descriptor();
        let result = record_interaction_action_admitted_result(&descriptor);

        let action = build_interaction_admitted_semantic_action(&result)
            .expect("admitted result should build action object");

        assert_eq!(
            action.effect_readiness,
            InteractionAdmittedSemanticActionEffectReadiness::NoEffect
        );
    }

    #[test]
    fn effect_capable_relationship_maps_to_deferred_effect_boundary() {
        let mut descriptor = select_descriptor();
        descriptor.effect_relationship =
            InteractionActionAdmissionEffectRelationship::MayRequestEffectAfterAdmission;

        let result = record_interaction_action_admitted_result(&descriptor);

        let action = build_interaction_admitted_semantic_action(&result)
            .expect("admitted result should build action object");

        assert_eq!(
            action.effect_readiness,
            InteractionAdmittedSemanticActionEffectReadiness::DeferredEffectBoundary
        );
        assert!(!action.is_effect_request());
    }

    #[test]
    fn admitted_action_object_generation_is_deterministic() {
        let descriptor = select_descriptor();
        let result = record_interaction_action_admitted_result(&descriptor);

        let first = build_interaction_admitted_semantic_action(&result);
        let second = build_interaction_admitted_semantic_action(&result);

        assert_eq!(first, second);
    }
}
