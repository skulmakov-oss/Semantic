//! Semantic UI action admission descriptor scaffold.
//!
//! This module records the checks that would be required before an action
//! candidate may become an admitted Semantic UI action. It does not execute
//! actions, perform admission, deny anything, request effects, call the VM,
//! call Host ABI, or mutate runtime state.

use crate::action_binding::{
    find_interaction_action_binding_by_action, InteractionActionBindingScope,
    InteractionActionEffectPolicy, InteractionActionName, InteractionActionTargetPolicy,
    InteractionActionTracePolicy,
};
use crate::action_binding_trace::{
    InteractionActionBindingTraceReport, InteractionActionBindingTraceStatus,
};
use crate::interaction::InteractionIntentKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InteractionActionAdmissionDescriptorId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionActionAdmissionTargetRequirement {
    Required,
    OptionalMayResolveLater,
    Ignored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionActionAdmissionTargetOwnershipRequirement {
    RequiredWhenTargetPresent,
    NotRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionActionAdmissionLifecycleRequirement {
    None,
    SessionActive,
    WindowAlive,
    SurfaceReady,
    FrameOpen,
    RuntimeNotQuarantined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionActionAdmissionCapabilityRequirement {
    None,
    DesktopSession,
    InputPoll,
    FrameEmit,
    FutureActionSpecific,
    FutureEffectSpecific,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionActionAdmissionTraceRequirement {
    Required,
    OptionalForDecorativeOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionActionAdmissionEffectRelationship {
    NoEffect,
    MayRequestEffectAfterAdmission,
    RequiresSeparateEffectAdmission,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionActionAdmissionDenialVisibility {
    Required,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionActionAdmissionPolicyGateNamespace {
    CoreUi,
    WorkbenchLocal,
    DiagnosticOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionActionAdmissionFutureResultShape {
    AdmitOrDenyWithReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionActionAdmissionDescriptor {
    pub id: InteractionActionAdmissionDescriptorId,
    pub action: InteractionActionName,
    pub source_intent: InteractionIntentKind,
    pub binding_id: crate::action_binding::InteractionActionBindingId,
    pub target_requirement: InteractionActionAdmissionTargetRequirement,
    pub target_ownership_requirement: InteractionActionAdmissionTargetOwnershipRequirement,
    pub lifecycle_requirement: InteractionActionAdmissionLifecycleRequirement,
    pub capability_requirement: InteractionActionAdmissionCapabilityRequirement,
    pub trace_requirement: InteractionActionAdmissionTraceRequirement,
    pub effect_relationship: InteractionActionAdmissionEffectRelationship,
    pub denial_visibility: InteractionActionAdmissionDenialVisibility,
    pub policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace,
    pub future_result_shape: InteractionActionAdmissionFutureResultShape,
}

pub fn describe_interaction_action_admission(
    binding_trace: &InteractionActionBindingTraceReport,
) -> Option<InteractionActionAdmissionDescriptor> {
    if !matches!(
        binding_trace.binding_status,
        InteractionActionBindingTraceStatus::Bound
    ) {
        return None;
    }

    let binding_id = binding_trace.binding_id?;
    let action = binding_trace.action?;
    let target_policy = binding_trace.target_policy?;
    let effect_policy = binding_trace.effect_policy?;
    let trace_policy = binding_trace.trace_policy?;

    let binding = find_interaction_action_binding_by_action(action)?;

    if binding.scope == InteractionActionBindingScope::DiagnosticOnly {
        return None;
    }

    Some(InteractionActionAdmissionDescriptor {
        id: InteractionActionAdmissionDescriptorId(binding_id.0),
        action,
        source_intent: binding_trace.intent_kind,
        binding_id,
        target_requirement: map_target_requirement(target_policy),
        target_ownership_requirement: map_target_ownership_requirement(target_policy),
        lifecycle_requirement: InteractionActionAdmissionLifecycleRequirement::SessionActive,
        capability_requirement: map_capability_requirement(action),
        trace_requirement: map_trace_requirement(trace_policy),
        effect_relationship: map_effect_relationship(effect_policy),
        denial_visibility: InteractionActionAdmissionDenialVisibility::Required,
        policy_gate_namespace: map_policy_gate_namespace(binding.scope),
        future_result_shape: InteractionActionAdmissionFutureResultShape::AdmitOrDenyWithReason,
    })
}

const fn map_target_requirement(
    policy: InteractionActionTargetPolicy,
) -> InteractionActionAdmissionTargetRequirement {
    match policy {
        InteractionActionTargetPolicy::RequiresTarget => {
            InteractionActionAdmissionTargetRequirement::Required
        }
        InteractionActionTargetPolicy::AllowsMissingTarget => {
            InteractionActionAdmissionTargetRequirement::OptionalMayResolveLater
        }
        InteractionActionTargetPolicy::IgnoresTarget => {
            InteractionActionAdmissionTargetRequirement::Ignored
        }
    }
}

const fn map_target_ownership_requirement(
    policy: InteractionActionTargetPolicy,
) -> InteractionActionAdmissionTargetOwnershipRequirement {
    match policy {
        InteractionActionTargetPolicy::RequiresTarget
        | InteractionActionTargetPolicy::AllowsMissingTarget => {
            InteractionActionAdmissionTargetOwnershipRequirement::RequiredWhenTargetPresent
        }
        InteractionActionTargetPolicy::IgnoresTarget => {
            InteractionActionAdmissionTargetOwnershipRequirement::NotRequired
        }
    }
}

const fn map_trace_requirement(
    policy: InteractionActionTracePolicy,
) -> InteractionActionAdmissionTraceRequirement {
    match policy {
        InteractionActionTracePolicy::Required => {
            InteractionActionAdmissionTraceRequirement::Required
        }
        InteractionActionTracePolicy::OptionalForDecorativeOnly => {
            InteractionActionAdmissionTraceRequirement::OptionalForDecorativeOnly
        }
    }
}

const fn map_effect_relationship(
    policy: InteractionActionEffectPolicy,
) -> InteractionActionAdmissionEffectRelationship {
    match policy {
        InteractionActionEffectPolicy::NoEffect => {
            InteractionActionAdmissionEffectRelationship::NoEffect
        }
        InteractionActionEffectPolicy::MayRequestEffect => {
            InteractionActionAdmissionEffectRelationship::MayRequestEffectAfterAdmission
        }
        InteractionActionEffectPolicy::RequiresSeparateEffectAdmission => {
            InteractionActionAdmissionEffectRelationship::RequiresSeparateEffectAdmission
        }
    }
}

const fn map_capability_requirement(
    action: InteractionActionName,
) -> InteractionActionAdmissionCapabilityRequirement {
    match action {
        InteractionActionName::CloseWindow => {
            InteractionActionAdmissionCapabilityRequirement::DesktopSession
        }
        InteractionActionName::SelectTraceEvent
        | InteractionActionName::FocusModule
        | InteractionActionName::OpenInspector
        | InteractionActionName::AcknowledgeError
        | InteractionActionName::OpenDenialReason
        | InteractionActionName::PinTrace => {
            InteractionActionAdmissionCapabilityRequirement::InputPoll
        }
        InteractionActionName::ExpandCapabilityGate
        | InteractionActionName::PrepareEffect
        | InteractionActionName::CommitEffect
        | InteractionActionName::RollbackEffect
        | InteractionActionName::QuarantineTarget => {
            InteractionActionAdmissionCapabilityRequirement::FutureActionSpecific
        }
        InteractionActionName::Unknown => {
            InteractionActionAdmissionCapabilityRequirement::FutureActionSpecific
        }
    }
}

const fn map_policy_gate_namespace(
    scope: InteractionActionBindingScope,
) -> InteractionActionAdmissionPolicyGateNamespace {
    match scope {
        InteractionActionBindingScope::CoreUi => {
            InteractionActionAdmissionPolicyGateNamespace::CoreUi
        }
        InteractionActionBindingScope::WorkbenchLocal => {
            InteractionActionAdmissionPolicyGateNamespace::WorkbenchLocal
        }
        InteractionActionBindingScope::DiagnosticOnly => {
            InteractionActionAdmissionPolicyGateNamespace::DiagnosticOnly
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action_binding::{
        InteractionActionAdmissionPolicy, InteractionActionBindingId,
        InteractionActionEffectPolicy, InteractionActionName, InteractionActionTargetPolicy,
        InteractionActionTracePolicy,
    };
    use crate::action_binding_trace::{
        InteractionActionBindingTraceReason, InteractionActionBindingTraceReport,
        InteractionActionBindingTraceStatus,
    };
    use crate::interaction::InteractionIntentKind;
    use crate::trace::InteractionIntentTraceStatus;

    fn close_binding_trace() -> InteractionActionBindingTraceReport {
        InteractionActionBindingTraceReport {
            intent_id: crate::interaction::InteractionIntentId(1),
            intent_kind: InteractionIntentKind::Close,
            intent_status: InteractionIntentTraceStatus::Classified,
            binding_status: InteractionActionBindingTraceStatus::Bound,
            reason: InteractionActionBindingTraceReason::BindingIgnoresTarget,
            binding_id: Some(InteractionActionBindingId(1)),
            action: Some(InteractionActionName::CloseWindow),
            target_policy: Some(InteractionActionTargetPolicy::IgnoresTarget),
            admission_policy: Some(InteractionActionAdmissionPolicy::Required),
            effect_policy: Some(InteractionActionEffectPolicy::NoEffect),
            trace_policy: Some(InteractionActionTracePolicy::Required),
        }
    }

    fn select_binding_trace() -> InteractionActionBindingTraceReport {
        InteractionActionBindingTraceReport {
            intent_id: crate::interaction::InteractionIntentId(2),
            intent_kind: InteractionIntentKind::Select,
            intent_status: InteractionIntentTraceStatus::Classified,
            binding_status: InteractionActionBindingTraceStatus::Bound,
            reason: InteractionActionBindingTraceReason::BindingRequiresTarget,
            binding_id: Some(InteractionActionBindingId(2)),
            action: Some(InteractionActionName::SelectTraceEvent),
            target_policy: Some(InteractionActionTargetPolicy::RequiresTarget),
            admission_policy: Some(InteractionActionAdmissionPolicy::Required),
            effect_policy: Some(InteractionActionEffectPolicy::NoEffect),
            trace_policy: Some(InteractionActionTracePolicy::Required),
        }
    }

    fn unbound_trace() -> InteractionActionBindingTraceReport {
        InteractionActionBindingTraceReport {
            intent_id: crate::interaction::InteractionIntentId(3),
            intent_kind: InteractionIntentKind::Activate,
            intent_status: InteractionIntentTraceStatus::Classified,
            binding_status: InteractionActionBindingTraceStatus::Unbound,
            reason: InteractionActionBindingTraceReason::NoBindingForIntent,
            binding_id: None,
            action: None,
            target_policy: None,
            admission_policy: None,
            effect_policy: None,
            trace_policy: None,
        }
    }

    #[test]
    fn bound_close_trace_produces_admission_descriptor() {
        let trace = close_binding_trace();

        let descriptor = describe_interaction_action_admission(&trace)
            .expect("bound close trace should produce descriptor");

        assert_eq!(descriptor.action, InteractionActionName::CloseWindow);
        assert_eq!(
            descriptor.capability_requirement,
            InteractionActionAdmissionCapabilityRequirement::DesktopSession
        );
        assert_eq!(
            descriptor.target_requirement,
            InteractionActionAdmissionTargetRequirement::Ignored
        );
    }

    #[test]
    fn bound_select_trace_requires_target_and_ownership() {
        let trace = select_binding_trace();

        let descriptor = describe_interaction_action_admission(&trace)
            .expect("bound select trace should produce descriptor");

        assert_eq!(descriptor.action, InteractionActionName::SelectTraceEvent);
        assert_eq!(
            descriptor.target_requirement,
            InteractionActionAdmissionTargetRequirement::Required
        );
        assert_eq!(
            descriptor.target_ownership_requirement,
            InteractionActionAdmissionTargetOwnershipRequirement::RequiredWhenTargetPresent
        );
    }

    #[test]
    fn unbound_trace_does_not_produce_admission_descriptor() {
        let trace = unbound_trace();

        let descriptor = describe_interaction_action_admission(&trace);

        assert!(descriptor.is_none());
    }

    #[test]
    fn descriptor_records_future_result_shape_only() {
        let trace = close_binding_trace();

        let descriptor = describe_interaction_action_admission(&trace)
            .expect("bound close trace should produce descriptor");

        assert_eq!(
            descriptor.future_result_shape,
            InteractionActionAdmissionFutureResultShape::AdmitOrDenyWithReason
        );
    }

    #[test]
    fn descriptor_preserves_no_effect_relationship() {
        let trace = close_binding_trace();

        let descriptor = describe_interaction_action_admission(&trace)
            .expect("bound close trace should produce descriptor");

        assert_eq!(
            descriptor.effect_relationship,
            InteractionActionAdmissionEffectRelationship::NoEffect
        );
    }

    #[test]
    fn descriptor_requires_denial_visibility() {
        let trace = select_binding_trace();

        let descriptor = describe_interaction_action_admission(&trace)
            .expect("bound select trace should produce descriptor");

        assert_eq!(
            descriptor.denial_visibility,
            InteractionActionAdmissionDenialVisibility::Required
        );
    }

    #[test]
    fn descriptor_generation_is_deterministic() {
        let trace = select_binding_trace();

        let first = describe_interaction_action_admission(&trace);
        let second = describe_interaction_action_admission(&trace);

        assert_eq!(first, second);
    }
}
