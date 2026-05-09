//! Semantic UI action admission result scaffold.
//!
//! This module records a future decision outcome for an action admission
//! descriptor. It does not implement denial traces, admitted actions,
//! dispatchers, effect bridges, or runtime mutation.

use crate::action_admission::{
    InteractionActionAdmissionDescriptor, InteractionActionAdmissionDescriptorId,
    InteractionActionAdmissionEffectRelationship,
    InteractionActionAdmissionPolicyGateNamespace,
    InteractionActionAdmissionTraceRequirement,
};
use crate::action_binding::{InteractionActionBindingId, InteractionActionName};
use crate::interaction::InteractionIntentKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InteractionActionAdmissionResultId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionActionAdmissionDecisionStatus {
    Admitted,
    Denied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionActionAdmissionDenialReason {
    MissingTarget,
    InvalidTargetOwnership,
    LifecycleBlocked,
    CapabilityMissing,
    PolicyDenied,
    EffectBoundaryRequired,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionActionAdmissionMissingRequirement {
    None,
    Target,
    TargetOwnership,
    Lifecycle,
    Capability,
    Policy,
    EffectBoundary,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionActionAdmissionResult {
    pub id: InteractionActionAdmissionResultId,
    pub descriptor_id: InteractionActionAdmissionDescriptorId,
    pub action: InteractionActionName,
    pub source_intent: InteractionIntentKind,
    pub binding_id: InteractionActionBindingId,
    pub status: InteractionActionAdmissionDecisionStatus,
    pub denial_reason: Option<InteractionActionAdmissionDenialReason>,
    pub missing_requirement: InteractionActionAdmissionMissingRequirement,
    pub trace_requirement: InteractionActionAdmissionTraceRequirement,
    pub effect_relationship: InteractionActionAdmissionEffectRelationship,
    pub policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace,
}

pub fn record_interaction_action_admitted_result(
    descriptor: &InteractionActionAdmissionDescriptor,
) -> InteractionActionAdmissionResult {
    InteractionActionAdmissionResult {
        id: InteractionActionAdmissionResultId(descriptor.id.0),
        descriptor_id: descriptor.id,
        action: descriptor.action,
        source_intent: descriptor.source_intent,
        binding_id: descriptor.binding_id,
        status: InteractionActionAdmissionDecisionStatus::Admitted,
        denial_reason: None,
        missing_requirement: InteractionActionAdmissionMissingRequirement::None,
        trace_requirement: descriptor.trace_requirement,
        effect_relationship: descriptor.effect_relationship,
        policy_gate_namespace: descriptor.policy_gate_namespace,
    }
}

pub fn record_interaction_action_denied_result(
    descriptor: &InteractionActionAdmissionDescriptor,
    reason: InteractionActionAdmissionDenialReason,
    missing_requirement: InteractionActionAdmissionMissingRequirement,
) -> InteractionActionAdmissionResult {
    InteractionActionAdmissionResult {
        id: InteractionActionAdmissionResultId(descriptor.id.0),
        descriptor_id: descriptor.id,
        action: descriptor.action,
        source_intent: descriptor.source_intent,
        binding_id: descriptor.binding_id,
        status: InteractionActionAdmissionDecisionStatus::Denied,
        denial_reason: Some(reason),
        missing_requirement,
        trace_requirement: descriptor.trace_requirement,
        effect_relationship: descriptor.effect_relationship,
        policy_gate_namespace: descriptor.policy_gate_namespace,
    }
}

impl InteractionActionAdmissionResult {
    pub const fn is_admitted(&self) -> bool {
        matches!(self.status, InteractionActionAdmissionDecisionStatus::Admitted)
    }

    pub const fn is_denied(&self) -> bool {
        matches!(self.status, InteractionActionAdmissionDecisionStatus::Denied)
    }

    pub const fn requires_denial_trace(&self) -> bool {
        self.is_denied()
    }

    pub const fn is_execution_authority(&self) -> bool {
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
    fn admitted_result_preserves_descriptor_identity() {
        let descriptor = close_descriptor();

        let result = record_interaction_action_admitted_result(&descriptor);

        assert_eq!(result.status, InteractionActionAdmissionDecisionStatus::Admitted);
        assert_eq!(result.descriptor_id, descriptor.id);
        assert_eq!(result.action, descriptor.action);
        assert_eq!(result.source_intent, descriptor.source_intent);
        assert_eq!(result.binding_id, descriptor.binding_id);
    }

    #[test]
    fn admitted_result_has_no_denial_reason() {
        let descriptor = close_descriptor();

        let result = record_interaction_action_admitted_result(&descriptor);

        assert_eq!(result.denial_reason, None);
        assert_eq!(
            result.missing_requirement,
            InteractionActionAdmissionMissingRequirement::None
        );
        assert!(result.is_admitted());
        assert!(!result.is_denied());
    }

    #[test]
    fn denied_result_records_reason_and_missing_requirement() {
        let descriptor = select_descriptor();

        let result = record_interaction_action_denied_result(
            &descriptor,
            InteractionActionAdmissionDenialReason::MissingTarget,
            InteractionActionAdmissionMissingRequirement::Target,
        );

        assert_eq!(result.status, InteractionActionAdmissionDecisionStatus::Denied);
        assert_eq!(
            result.denial_reason,
            Some(InteractionActionAdmissionDenialReason::MissingTarget)
        );
        assert_eq!(
            result.missing_requirement,
            InteractionActionAdmissionMissingRequirement::Target
        );
        assert!(result.is_denied());
    }

    #[test]
    fn denied_result_requires_future_denial_trace() {
        let descriptor = select_descriptor();

        let result = record_interaction_action_denied_result(
            &descriptor,
            InteractionActionAdmissionDenialReason::CapabilityMissing,
            InteractionActionAdmissionMissingRequirement::Capability,
        );

        assert!(result.requires_denial_trace());
    }

    #[test]
    fn admission_result_is_not_execution_authority() {
        let descriptor = close_descriptor();

        let result = record_interaction_action_admitted_result(&descriptor);

        assert!(!result.is_execution_authority());
    }

    #[test]
    fn result_preserves_effect_relationship_without_requesting_effect() {
        let descriptor = close_descriptor();

        let result = record_interaction_action_admitted_result(&descriptor);

        assert_eq!(result.effect_relationship, descriptor.effect_relationship);
        assert!(!result.is_execution_authority());
    }

    #[test]
    fn admission_result_generation_is_deterministic() {
        let descriptor = select_descriptor();

        let first = record_interaction_action_admitted_result(&descriptor);
        let second = record_interaction_action_admitted_result(&descriptor);

        assert_eq!(first, second);
    }
}
