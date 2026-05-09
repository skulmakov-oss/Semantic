//! Semantic UI action denial trace scaffold.
//!
//! This module records a visible denial trace for a denied admission result.
//! It does not implement audit authority, execution authority, dispatchers,
//! effect bridges, or runtime mutation.

use crate::action_admission::{
    InteractionActionAdmissionDescriptorId, InteractionActionAdmissionEffectRelationship,
    InteractionActionAdmissionPolicyGateNamespace, InteractionActionAdmissionTraceRequirement,
};
use crate::action_admission_result::{
    InteractionActionAdmissionDenialReason, InteractionActionAdmissionMissingRequirement,
    InteractionActionAdmissionResult, InteractionActionAdmissionResultId,
};
use crate::action_binding::{InteractionActionBindingId, InteractionActionName};
use crate::interaction::InteractionIntentKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InteractionActionDenialTraceId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionActionDenialRetryHint {
    NotUseful,
    UsefulAfterTargetChange,
    UsefulAfterLifecycleChange,
    UsefulAfterCapabilityChange,
    UsefulAfterPolicyChange,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionActionDenialTrace {
    pub id: InteractionActionDenialTraceId,
    pub result_id: InteractionActionAdmissionResultId,
    pub descriptor_id: InteractionActionAdmissionDescriptorId,
    pub action: InteractionActionName,
    pub source_intent: InteractionIntentKind,
    pub binding_id: InteractionActionBindingId,
    pub denial_reason: InteractionActionAdmissionDenialReason,
    pub missing_requirement: InteractionActionAdmissionMissingRequirement,
    pub trace_requirement: InteractionActionAdmissionTraceRequirement,
    pub effect_relationship: InteractionActionAdmissionEffectRelationship,
    pub policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace,
    pub retry_hint: InteractionActionDenialRetryHint,
}

pub fn trace_interaction_action_denial(
    result: &InteractionActionAdmissionResult,
) -> Option<InteractionActionDenialTrace> {
    if !result.is_denied() {
        return None;
    }

    let denial_reason = result.denial_reason?;

    Some(InteractionActionDenialTrace {
        id: InteractionActionDenialTraceId(result.id.0),
        result_id: result.id,
        descriptor_id: result.descriptor_id,
        action: result.action,
        source_intent: result.source_intent,
        binding_id: result.binding_id,
        denial_reason,
        missing_requirement: result.missing_requirement,
        trace_requirement: result.trace_requirement,
        effect_relationship: result.effect_relationship,
        policy_gate_namespace: result.policy_gate_namespace,
        retry_hint: retry_hint_for_missing_requirement(result.missing_requirement),
    })
}

const fn retry_hint_for_missing_requirement(
    missing: InteractionActionAdmissionMissingRequirement,
) -> InteractionActionDenialRetryHint {
    match missing {
        InteractionActionAdmissionMissingRequirement::Target
        | InteractionActionAdmissionMissingRequirement::TargetOwnership => {
            InteractionActionDenialRetryHint::UsefulAfterTargetChange
        }
        InteractionActionAdmissionMissingRequirement::Lifecycle => {
            InteractionActionDenialRetryHint::UsefulAfterLifecycleChange
        }
        InteractionActionAdmissionMissingRequirement::Capability => {
            InteractionActionDenialRetryHint::UsefulAfterCapabilityChange
        }
        InteractionActionAdmissionMissingRequirement::Policy => {
            InteractionActionDenialRetryHint::UsefulAfterPolicyChange
        }
        InteractionActionAdmissionMissingRequirement::EffectBoundary
        | InteractionActionAdmissionMissingRequirement::None => {
            InteractionActionDenialRetryHint::NotUseful
        }
        InteractionActionAdmissionMissingRequirement::Unknown => {
            InteractionActionDenialRetryHint::Unknown
        }
    }
}

impl InteractionActionDenialTrace {
    pub const fn is_audit_authority(&self) -> bool {
        false
    }

    pub const fn is_execution_authority(&self) -> bool {
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
        InteractionActionAdmissionDenialVisibility,
        InteractionActionAdmissionDescriptor,
        InteractionActionAdmissionDescriptorId,
        InteractionActionAdmissionEffectRelationship,
        InteractionActionAdmissionFutureResultShape,
        InteractionActionAdmissionLifecycleRequirement,
        InteractionActionAdmissionPolicyGateNamespace,
        InteractionActionAdmissionTargetOwnershipRequirement,
        InteractionActionAdmissionTargetRequirement,
        InteractionActionAdmissionTraceRequirement,
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
            capability_requirement:
                InteractionActionAdmissionCapabilityRequirement::DesktopSession,
            trace_requirement: InteractionActionAdmissionTraceRequirement::Required,
            effect_relationship: InteractionActionAdmissionEffectRelationship::NoEffect,
            denial_visibility: InteractionActionAdmissionDenialVisibility::Required,
            policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace::CoreUi,
            future_result_shape:
                InteractionActionAdmissionFutureResultShape::AdmitOrDenyWithReason,
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
            capability_requirement:
                InteractionActionAdmissionCapabilityRequirement::InputPoll,
            trace_requirement: InteractionActionAdmissionTraceRequirement::Required,
            effect_relationship: InteractionActionAdmissionEffectRelationship::NoEffect,
            denial_visibility: InteractionActionAdmissionDenialVisibility::Required,
            policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace::CoreUi,
            future_result_shape:
                InteractionActionAdmissionFutureResultShape::AdmitOrDenyWithReason,
        }
    }

    #[test]
    fn denied_result_produces_denial_trace() {
        let descriptor = select_descriptor();

        let result = record_interaction_action_denied_result(
            &descriptor,
            InteractionActionAdmissionDenialReason::MissingTarget,
            InteractionActionAdmissionMissingRequirement::Target,
        );

        let trace = trace_interaction_action_denial(&result)
            .expect("denied result should produce denial trace");

        assert_eq!(trace.result_id, result.id);
        assert_eq!(trace.descriptor_id, result.descriptor_id);
        assert_eq!(trace.action, result.action);
        assert_eq!(trace.denial_reason, InteractionActionAdmissionDenialReason::MissingTarget);
    }

    #[test]
    fn admitted_result_does_not_produce_denial_trace() {
        let descriptor = close_descriptor();

        let result = record_interaction_action_admitted_result(&descriptor);

        let trace = trace_interaction_action_denial(&result);

        assert!(trace.is_none());
    }

    #[test]
    fn missing_target_maps_to_target_retry_hint() {
        let descriptor = select_descriptor();

        let result = record_interaction_action_denied_result(
            &descriptor,
            InteractionActionAdmissionDenialReason::MissingTarget,
            InteractionActionAdmissionMissingRequirement::Target,
        );

        let trace = trace_interaction_action_denial(&result)
            .expect("denied result should produce denial trace");

        assert_eq!(
            trace.retry_hint,
            InteractionActionDenialRetryHint::UsefulAfterTargetChange
        );
    }

    #[test]
    fn capability_missing_maps_to_capability_retry_hint() {
        let descriptor = select_descriptor();

        let result = record_interaction_action_denied_result(
            &descriptor,
            InteractionActionAdmissionDenialReason::CapabilityMissing,
            InteractionActionAdmissionMissingRequirement::Capability,
        );

        let trace = trace_interaction_action_denial(&result)
            .expect("denied result should produce denial trace");

        assert_eq!(
            trace.retry_hint,
            InteractionActionDenialRetryHint::UsefulAfterCapabilityChange
        );
    }

    #[test]
    fn denial_trace_is_not_authority() {
        let descriptor = select_descriptor();

        let result = record_interaction_action_denied_result(
            &descriptor,
            InteractionActionAdmissionDenialReason::PolicyDenied,
            InteractionActionAdmissionMissingRequirement::Policy,
        );

        let trace = trace_interaction_action_denial(&result)
            .expect("denied result should produce denial trace");

        assert!(!trace.is_audit_authority());
        assert!(!trace.is_execution_authority());
        assert!(!trace.is_runtime_mutation());
    }

    #[test]
    fn denial_trace_preserves_effect_relationship_without_requesting_effect() {
        let descriptor = select_descriptor();

        let result = record_interaction_action_denied_result(
            &descriptor,
            InteractionActionAdmissionDenialReason::EffectBoundaryRequired,
            InteractionActionAdmissionMissingRequirement::EffectBoundary,
        );

        let trace = trace_interaction_action_denial(&result)
            .expect("denied result should produce denial trace");

        assert_eq!(trace.effect_relationship, result.effect_relationship);
        assert!(!trace.is_execution_authority());
    }

    #[test]
    fn denial_trace_generation_is_deterministic() {
        let descriptor = select_descriptor();

        let result = record_interaction_action_denied_result(
            &descriptor,
            InteractionActionAdmissionDenialReason::MissingTarget,
            InteractionActionAdmissionMissingRequirement::Target,
        );

        let first = trace_interaction_action_denial(&result);
        let second = trace_interaction_action_denial(&result);

        assert_eq!(first, second);
    }
}
