//! Semantic UI effect request descriptor scaffold.
//!
//! This module records an inert description of an effect request path derived
//! from admitted semantic UI action dispatch metadata. It does not perform
//! capability admission, execute effects, call the VM, call Host ABI, or
//! mutate runtime state.

use crate::action_admission::{
    InteractionActionAdmissionPolicyGateNamespace, InteractionActionAdmissionTraceRequirement,
};
use crate::action_binding::InteractionActionName;
use crate::action_dispatch_record::InteractionSemanticActionDispatchRecordId;
use crate::action_dispatch_route::{InteractionSemanticActionDispatchEffectEligibility, InteractionSemanticActionDispatchRouteId};
use crate::action_dispatch_trace::{
    InteractionSemanticActionDispatchTraceReport, InteractionSemanticActionDispatchTraceStatus,
};
use crate::admitted_action::InteractionAdmittedSemanticActionId;
use crate::interaction::InteractionIntentKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InteractionEffectRequestDescriptorId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionEffectRequestKind {
    PrepareEffect,
    CommitEffect,
    RollbackEffect,
    CloseWindow,
    QuarantineTarget,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionEffectRequestTargetPolicy {
    Required,
    Optional,
    Ignored,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionEffectRequestUiCapability {
    WindowLifecycle,
    EffectControl,
    TargetQuarantine,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionEffectRequestRuntimeCapability {
    WindowLifecycle,
    EffectControl,
    TargetQuarantine,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionEffectRequestLifecyclePrecondition {
    SessionActive,
    WindowAlive,
    TargetPresent,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionEffectRequestDenialBehavior {
    VisibleRequired,
    VisibleOptional,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionEffectRequestPrepareCommitRelationship {
    PrepareThenCommit,
    CommitAfterPrepare,
    RollbackAfterPrepare,
    Direct,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionEffectRequestScope {
    CoreUi,
    WorkbenchLocal,
    DiagnosticOnly,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionEffectRequestDescriptor {
    pub id: InteractionEffectRequestDescriptorId,
    pub source_admitted_action_id: InteractionAdmittedSemanticActionId,
    pub dispatch_record_id: InteractionSemanticActionDispatchRecordId,
    pub dispatch_route_id: InteractionSemanticActionDispatchRouteId,
    pub action: InteractionActionName,
    pub source_intent: InteractionIntentKind,
    pub requested_effect: InteractionEffectRequestKind,
    pub target_policy: InteractionEffectRequestTargetPolicy,
    pub required_ui_capability: InteractionEffectRequestUiCapability,
    pub required_runtime_capability: InteractionEffectRequestRuntimeCapability,
    pub lifecycle_precondition: InteractionEffectRequestLifecyclePrecondition,
    pub trace_requirement: InteractionActionAdmissionTraceRequirement,
    pub denial_behavior: InteractionEffectRequestDenialBehavior,
    pub prepare_commit_relationship: InteractionEffectRequestPrepareCommitRelationship,
    pub policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace,
    pub scope: InteractionEffectRequestScope,
}

pub fn describe_interaction_effect_request(
    trace: &InteractionSemanticActionDispatchTraceReport,
) -> Option<InteractionEffectRequestDescriptor> {
    if !matches!(
        trace.trace_status,
        InteractionSemanticActionDispatchTraceStatus::Recorded
    ) {
        return None;
    }

    if !matches!(
        trace.effect_eligibility,
        InteractionSemanticActionDispatchEffectEligibility::RequiresFutureEffectBoundary
    ) {
        return None;
    }

    Some(InteractionEffectRequestDescriptor {
        id: InteractionEffectRequestDescriptorId(trace.record_id.0),
        source_admitted_action_id: trace.admitted_action_id,
        dispatch_record_id: trace.record_id,
        dispatch_route_id: trace.route_id,
        action: trace.action,
        source_intent: trace.source_intent,
        requested_effect: map_requested_effect(trace.action),
        target_policy: map_target_policy(trace.action),
        required_ui_capability: map_ui_capability(trace.action),
        required_runtime_capability: map_runtime_capability(trace.action),
        lifecycle_precondition: map_lifecycle_precondition(trace.action),
        trace_requirement: trace.trace_requirement,
        denial_behavior: map_denial_behavior(trace.action),
        prepare_commit_relationship: map_prepare_commit_relationship(trace.action),
        policy_gate_namespace: trace.policy_gate_namespace,
        scope: map_scope(trace.policy_gate_namespace),
    })
}

const fn map_requested_effect(action: InteractionActionName) -> InteractionEffectRequestKind {
    match action {
        InteractionActionName::PrepareEffect => InteractionEffectRequestKind::PrepareEffect,
        InteractionActionName::CommitEffect => InteractionEffectRequestKind::CommitEffect,
        InteractionActionName::RollbackEffect => InteractionEffectRequestKind::RollbackEffect,
        InteractionActionName::CloseWindow => InteractionEffectRequestKind::CloseWindow,
        InteractionActionName::QuarantineTarget => InteractionEffectRequestKind::QuarantineTarget,
        InteractionActionName::OpenInspector
        | InteractionActionName::SelectTraceEvent
        | InteractionActionName::FocusModule
        | InteractionActionName::ExpandCapabilityGate
        | InteractionActionName::AcknowledgeError
        | InteractionActionName::OpenDenialReason
        | InteractionActionName::PinTrace
        | InteractionActionName::Unknown => InteractionEffectRequestKind::Unknown,
    }
}

const fn map_target_policy(action: InteractionActionName) -> InteractionEffectRequestTargetPolicy {
    match action {
        InteractionActionName::CloseWindow => InteractionEffectRequestTargetPolicy::Optional,
        InteractionActionName::PrepareEffect
        | InteractionActionName::CommitEffect
        | InteractionActionName::RollbackEffect
        | InteractionActionName::QuarantineTarget => InteractionEffectRequestTargetPolicy::Required,
        InteractionActionName::OpenInspector
        | InteractionActionName::SelectTraceEvent
        | InteractionActionName::FocusModule
        | InteractionActionName::ExpandCapabilityGate
        | InteractionActionName::AcknowledgeError
        | InteractionActionName::OpenDenialReason
        | InteractionActionName::PinTrace
        | InteractionActionName::Unknown => InteractionEffectRequestTargetPolicy::Ignored,
    }
}

const fn map_ui_capability(action: InteractionActionName) -> InteractionEffectRequestUiCapability {
    match action {
        InteractionActionName::CloseWindow => InteractionEffectRequestUiCapability::WindowLifecycle,
        InteractionActionName::PrepareEffect
        | InteractionActionName::CommitEffect
        | InteractionActionName::RollbackEffect => InteractionEffectRequestUiCapability::EffectControl,
        InteractionActionName::QuarantineTarget => {
            InteractionEffectRequestUiCapability::TargetQuarantine
        }
        InteractionActionName::OpenInspector
        | InteractionActionName::SelectTraceEvent
        | InteractionActionName::FocusModule
        | InteractionActionName::ExpandCapabilityGate
        | InteractionActionName::AcknowledgeError
        | InteractionActionName::OpenDenialReason
        | InteractionActionName::PinTrace
        | InteractionActionName::Unknown => InteractionEffectRequestUiCapability::Unknown,
    }
}

const fn map_runtime_capability(
    action: InteractionActionName,
) -> InteractionEffectRequestRuntimeCapability {
    match action {
        InteractionActionName::CloseWindow => {
            InteractionEffectRequestRuntimeCapability::WindowLifecycle
        }
        InteractionActionName::PrepareEffect
        | InteractionActionName::CommitEffect
        | InteractionActionName::RollbackEffect => {
            InteractionEffectRequestRuntimeCapability::EffectControl
        }
        InteractionActionName::QuarantineTarget => {
            InteractionEffectRequestRuntimeCapability::TargetQuarantine
        }
        InteractionActionName::OpenInspector
        | InteractionActionName::SelectTraceEvent
        | InteractionActionName::FocusModule
        | InteractionActionName::ExpandCapabilityGate
        | InteractionActionName::AcknowledgeError
        | InteractionActionName::OpenDenialReason
        | InteractionActionName::PinTrace
        | InteractionActionName::Unknown => InteractionEffectRequestRuntimeCapability::Unknown,
    }
}

const fn map_lifecycle_precondition(
    action: InteractionActionName,
) -> InteractionEffectRequestLifecyclePrecondition {
    match action {
        InteractionActionName::CloseWindow => InteractionEffectRequestLifecyclePrecondition::WindowAlive,
        InteractionActionName::PrepareEffect
        | InteractionActionName::CommitEffect
        | InteractionActionName::RollbackEffect => {
            InteractionEffectRequestLifecyclePrecondition::SessionActive
        }
        InteractionActionName::QuarantineTarget => {
            InteractionEffectRequestLifecyclePrecondition::TargetPresent
        }
        InteractionActionName::OpenInspector
        | InteractionActionName::SelectTraceEvent
        | InteractionActionName::FocusModule
        | InteractionActionName::ExpandCapabilityGate
        | InteractionActionName::AcknowledgeError
        | InteractionActionName::OpenDenialReason
        | InteractionActionName::PinTrace
        | InteractionActionName::Unknown => InteractionEffectRequestLifecyclePrecondition::Unknown,
    }
}

const fn map_denial_behavior(action: InteractionActionName) -> InteractionEffectRequestDenialBehavior {
    match action {
        InteractionActionName::CloseWindow
        | InteractionActionName::PrepareEffect
        | InteractionActionName::CommitEffect
        | InteractionActionName::RollbackEffect
        | InteractionActionName::QuarantineTarget => {
            InteractionEffectRequestDenialBehavior::VisibleRequired
        }
        InteractionActionName::OpenInspector
        | InteractionActionName::SelectTraceEvent
        | InteractionActionName::FocusModule
        | InteractionActionName::ExpandCapabilityGate
        | InteractionActionName::AcknowledgeError
        | InteractionActionName::OpenDenialReason
        | InteractionActionName::PinTrace
        | InteractionActionName::Unknown => InteractionEffectRequestDenialBehavior::Unknown,
    }
}

const fn map_prepare_commit_relationship(
    action: InteractionActionName,
) -> InteractionEffectRequestPrepareCommitRelationship {
    match action {
        InteractionActionName::CommitEffect => {
            InteractionEffectRequestPrepareCommitRelationship::CommitAfterPrepare
        }
        InteractionActionName::RollbackEffect => {
            InteractionEffectRequestPrepareCommitRelationship::RollbackAfterPrepare
        }
        InteractionActionName::CloseWindow
        | InteractionActionName::PrepareEffect
        | InteractionActionName::QuarantineTarget => {
            InteractionEffectRequestPrepareCommitRelationship::PrepareThenCommit
        }
        InteractionActionName::OpenInspector
        | InteractionActionName::SelectTraceEvent
        | InteractionActionName::FocusModule
        | InteractionActionName::ExpandCapabilityGate
        | InteractionActionName::AcknowledgeError
        | InteractionActionName::OpenDenialReason
        | InteractionActionName::PinTrace
        | InteractionActionName::Unknown => InteractionEffectRequestPrepareCommitRelationship::Unknown,
    }
}

const fn map_scope(
    namespace: InteractionActionAdmissionPolicyGateNamespace,
) -> InteractionEffectRequestScope {
    match namespace {
        InteractionActionAdmissionPolicyGateNamespace::CoreUi => InteractionEffectRequestScope::CoreUi,
        InteractionActionAdmissionPolicyGateNamespace::WorkbenchLocal => {
            InteractionEffectRequestScope::WorkbenchLocal
        }
        InteractionActionAdmissionPolicyGateNamespace::DiagnosticOnly => {
            InteractionEffectRequestScope::DiagnosticOnly
        }
    }
}

impl InteractionEffectRequestDescriptor {
    pub const fn is_capability_admission(&self) -> bool {
        false
    }

    pub const fn is_prepared_effect(&self) -> bool {
        false
    }

    pub const fn is_committed_effect(&self) -> bool {
        false
    }

    pub const fn is_execution_authority(&self) -> bool {
        false
    }

    pub const fn is_runtime_mutation(&self) -> bool {
        false
    }

    pub const fn is_audit_authority(&self) -> bool {
        false
    }

    pub const fn calls_vm_or_host_abi(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action_admission::{
        InteractionActionAdmissionPolicyGateNamespace, InteractionActionAdmissionTraceRequirement,
    };
    use crate::action_binding::InteractionActionName;
    use crate::action_dispatch_record::{
        InteractionSemanticActionDispatchBlockReason, InteractionSemanticActionDispatchRecordId,
        InteractionSemanticActionDispatchRecordStatus,
    };
    use crate::action_dispatch_route::{
        InteractionSemanticActionDispatchEffectEligibility,
        InteractionSemanticActionDispatchRouteId, InteractionSemanticActionDispatchRouteKind,
    };
    use crate::action_dispatch_trace::{
        InteractionSemanticActionDispatchTraceReason, InteractionSemanticActionDispatchTraceReport,
        InteractionSemanticActionDispatchTraceStatus,
    };
    use crate::admitted_action::InteractionAdmittedSemanticActionId;
    use crate::interaction::InteractionIntentKind;

    fn prepare_effect_trace() -> InteractionSemanticActionDispatchTraceReport {
        InteractionSemanticActionDispatchTraceReport {
            record_id: InteractionSemanticActionDispatchRecordId(11),
            route_id: InteractionSemanticActionDispatchRouteId(11),
            admitted_action_id: InteractionAdmittedSemanticActionId(11),
            action: InteractionActionName::PrepareEffect,
            source_intent: InteractionIntentKind::Submit,
            binding_id: crate::action_binding::InteractionActionBindingId(11),
            route: InteractionSemanticActionDispatchRouteKind::EffectRequestCandidate,
            record_status: InteractionSemanticActionDispatchRecordStatus::Recorded,
            trace_status: InteractionSemanticActionDispatchTraceStatus::Recorded,
            block_reason: InteractionSemanticActionDispatchBlockReason::None,
            trace_reason: InteractionSemanticActionDispatchTraceReason::RouteRecorded,
            trace_requirement: InteractionActionAdmissionTraceRequirement::Required,
            effect_relationship:
                crate::action_admission::InteractionActionAdmissionEffectRelationship::MayRequestEffectAfterAdmission,
            policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace::CoreUi,
            effect_eligibility:
                InteractionSemanticActionDispatchEffectEligibility::RequiresFutureEffectBoundary,
        }
    }

    fn close_window_no_effect_trace() -> InteractionSemanticActionDispatchTraceReport {
        InteractionSemanticActionDispatchTraceReport {
            record_id: InteractionSemanticActionDispatchRecordId(12),
            route_id: InteractionSemanticActionDispatchRouteId(12),
            admitted_action_id: InteractionAdmittedSemanticActionId(12),
            action: InteractionActionName::CloseWindow,
            source_intent: InteractionIntentKind::Close,
            binding_id: crate::action_binding::InteractionActionBindingId(12),
            route: InteractionSemanticActionDispatchRouteKind::LocalUiStateCandidate,
            record_status: InteractionSemanticActionDispatchRecordStatus::Recorded,
            trace_status: InteractionSemanticActionDispatchTraceStatus::Recorded,
            block_reason: InteractionSemanticActionDispatchBlockReason::None,
            trace_reason: InteractionSemanticActionDispatchTraceReason::RouteRecorded,
            trace_requirement: InteractionActionAdmissionTraceRequirement::Required,
            effect_relationship: crate::action_admission::InteractionActionAdmissionEffectRelationship::NoEffect,
            policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace::WorkbenchLocal,
            effect_eligibility: InteractionSemanticActionDispatchEffectEligibility::NoEffect,
        }
    }

    #[test]
    fn effect_trace_produces_descriptor() {
        let trace = prepare_effect_trace();

        let descriptor = describe_interaction_effect_request(&trace)
            .expect("effect candidate trace should produce descriptor");

        assert_eq!(descriptor.id, InteractionEffectRequestDescriptorId(11));
        assert_eq!(descriptor.source_admitted_action_id, trace.admitted_action_id);
        assert_eq!(descriptor.dispatch_record_id, trace.record_id);
        assert_eq!(descriptor.dispatch_route_id, trace.route_id);
        assert_eq!(descriptor.action, trace.action);
        assert_eq!(descriptor.source_intent, trace.source_intent);
        assert_eq!(
            descriptor.requested_effect,
            InteractionEffectRequestKind::PrepareEffect
        );
    }

    #[test]
    fn close_window_trace_does_not_produce_descriptor() {
        let trace = close_window_no_effect_trace();

        let descriptor = describe_interaction_effect_request(&trace);

        assert!(descriptor.is_none());
    }

    #[test]
    fn descriptor_preserves_capability_and_policy_metadata() {
        let trace = prepare_effect_trace();

        let descriptor = describe_interaction_effect_request(&trace)
            .expect("effect candidate trace should produce descriptor");

        assert_eq!(
            descriptor.target_policy,
            InteractionEffectRequestTargetPolicy::Required
        );
        assert_eq!(
            descriptor.required_ui_capability,
            InteractionEffectRequestUiCapability::EffectControl
        );
        assert_eq!(
            descriptor.required_runtime_capability,
            InteractionEffectRequestRuntimeCapability::EffectControl
        );
        assert_eq!(
            descriptor.lifecycle_precondition,
            InteractionEffectRequestLifecyclePrecondition::SessionActive
        );
        assert_eq!(descriptor.trace_requirement, trace.trace_requirement);
        assert_eq!(
            descriptor.denial_behavior,
            InteractionEffectRequestDenialBehavior::VisibleRequired
        );
        assert_eq!(
            descriptor.prepare_commit_relationship,
            InteractionEffectRequestPrepareCommitRelationship::PrepareThenCommit
        );
        assert_eq!(descriptor.policy_gate_namespace, trace.policy_gate_namespace);
        assert_eq!(descriptor.scope, InteractionEffectRequestScope::CoreUi);
    }

    #[test]
    fn close_window_maps_to_window_lifecycle_metadata() {
        let mut trace = close_window_no_effect_trace();
        trace.route = InteractionSemanticActionDispatchRouteKind::EffectRequestCandidate;
        trace.effect_eligibility =
            InteractionSemanticActionDispatchEffectEligibility::RequiresFutureEffectBoundary;
        trace.policy_gate_namespace = InteractionActionAdmissionPolicyGateNamespace::WorkbenchLocal;

        let descriptor = describe_interaction_effect_request(&trace)
            .expect("effect candidate trace should produce descriptor");

        assert_eq!(
            descriptor.requested_effect,
            InteractionEffectRequestKind::CloseWindow
        );
        assert_eq!(
            descriptor.target_policy,
            InteractionEffectRequestTargetPolicy::Optional
        );
        assert_eq!(
            descriptor.required_ui_capability,
            InteractionEffectRequestUiCapability::WindowLifecycle
        );
        assert_eq!(
            descriptor.required_runtime_capability,
            InteractionEffectRequestRuntimeCapability::WindowLifecycle
        );
        assert_eq!(
            descriptor.lifecycle_precondition,
            InteractionEffectRequestLifecyclePrecondition::WindowAlive
        );
        assert_eq!(descriptor.scope, InteractionEffectRequestScope::WorkbenchLocal);
    }

    #[test]
    fn descriptor_is_not_authority() {
        let trace = prepare_effect_trace();

        let descriptor = describe_interaction_effect_request(&trace)
            .expect("effect candidate trace should produce descriptor");

        assert!(!descriptor.is_capability_admission());
        assert!(!descriptor.is_prepared_effect());
        assert!(!descriptor.is_committed_effect());
        assert!(!descriptor.is_execution_authority());
        assert!(!descriptor.is_runtime_mutation());
        assert!(!descriptor.is_audit_authority());
        assert!(!descriptor.calls_vm_or_host_abi());
    }

    #[test]
    fn descriptor_generation_is_deterministic() {
        let trace = prepare_effect_trace();

        let first = describe_interaction_effect_request(&trace);
        let second = describe_interaction_effect_request(&trace);

        assert_eq!(first, second);
    }
}
