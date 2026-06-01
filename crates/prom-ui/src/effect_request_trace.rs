//! Semantic UI effect request trace scaffold.
//!
//! This module records a visible trace report for effect request descriptor
//! metadata. It does not implement capability admission, prepared effects,
//! committed effects, effect execution, VM/Host ABI calls, audit authority, or
//! runtime mutation.

use crate::action_admission::{
    InteractionActionAdmissionPolicyGateNamespace, InteractionActionAdmissionTraceRequirement,
};
use crate::action_binding::InteractionActionName;
use crate::action_dispatch_record::InteractionSemanticActionDispatchRecordId;
use crate::action_dispatch_route::InteractionSemanticActionDispatchRouteId;
use crate::admitted_action::InteractionAdmittedSemanticActionId;
use crate::effect_request::{
    InteractionEffectRequestDenialBehavior, InteractionEffectRequestDescriptor,
    InteractionEffectRequestDescriptorId, InteractionEffectRequestKind,
    InteractionEffectRequestLifecyclePrecondition,
    InteractionEffectRequestPrepareCommitRelationship, InteractionEffectRequestRuntimeCapability,
    InteractionEffectRequestScope, InteractionEffectRequestTargetPolicy,
    InteractionEffectRequestUiCapability,
};
use crate::interaction::InteractionIntentKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionEffectRequestTraceStatus {
    Described,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionEffectRequestTraceReason {
    DescriptorRecorded,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionEffectRequestTraceReport {
    pub descriptor_id: InteractionEffectRequestDescriptorId,
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
    pub trace_status: InteractionEffectRequestTraceStatus,
    pub trace_reason: InteractionEffectRequestTraceReason,
}

pub fn trace_interaction_effect_request(
    descriptor: &InteractionEffectRequestDescriptor,
) -> InteractionEffectRequestTraceReport {
    InteractionEffectRequestTraceReport {
        descriptor_id: descriptor.id,
        source_admitted_action_id: descriptor.source_admitted_action_id,
        dispatch_record_id: descriptor.dispatch_record_id,
        dispatch_route_id: descriptor.dispatch_route_id,
        action: descriptor.action,
        source_intent: descriptor.source_intent,
        requested_effect: descriptor.requested_effect,
        target_policy: descriptor.target_policy,
        required_ui_capability: descriptor.required_ui_capability,
        required_runtime_capability: descriptor.required_runtime_capability,
        lifecycle_precondition: descriptor.lifecycle_precondition,
        trace_requirement: descriptor.trace_requirement,
        denial_behavior: descriptor.denial_behavior,
        prepare_commit_relationship: descriptor.prepare_commit_relationship,
        policy_gate_namespace: descriptor.policy_gate_namespace,
        scope: descriptor.scope,
        trace_status: InteractionEffectRequestTraceStatus::Described,
        trace_reason: InteractionEffectRequestTraceReason::DescriptorRecorded,
    }
}

impl InteractionEffectRequestTraceReport {
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
        InteractionActionAdmissionEffectRelationship,
        InteractionActionAdmissionPolicyGateNamespace, InteractionActionAdmissionTraceRequirement,
    };
    use crate::action_binding::{InteractionActionBindingId, InteractionActionName};
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
    use crate::effect_request::{
        describe_interaction_effect_request, InteractionEffectRequestDescriptor,
    };
    use crate::interaction::InteractionIntentKind;

    fn prepare_effect_trace() -> InteractionSemanticActionDispatchTraceReport {
        InteractionSemanticActionDispatchTraceReport {
            record_id: InteractionSemanticActionDispatchRecordId(11),
            route_id: InteractionSemanticActionDispatchRouteId(11),
            admitted_action_id: InteractionAdmittedSemanticActionId(11),
            action: InteractionActionName::PrepareEffect,
            source_intent: InteractionIntentKind::Submit,
            binding_id: InteractionActionBindingId(11),
            route: InteractionSemanticActionDispatchRouteKind::EffectRequestCandidate,
            record_status: InteractionSemanticActionDispatchRecordStatus::Recorded,
            trace_status: InteractionSemanticActionDispatchTraceStatus::Recorded,
            block_reason: InteractionSemanticActionDispatchBlockReason::None,
            trace_reason: InteractionSemanticActionDispatchTraceReason::RouteRecorded,
            trace_requirement: InteractionActionAdmissionTraceRequirement::Required,
            effect_relationship:
                InteractionActionAdmissionEffectRelationship::MayRequestEffectAfterAdmission,
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
            binding_id: InteractionActionBindingId(12),
            route: InteractionSemanticActionDispatchRouteKind::LocalUiStateCandidate,
            record_status: InteractionSemanticActionDispatchRecordStatus::Recorded,
            trace_status: InteractionSemanticActionDispatchTraceStatus::Recorded,
            block_reason: InteractionSemanticActionDispatchBlockReason::None,
            trace_reason: InteractionSemanticActionDispatchTraceReason::RouteRecorded,
            trace_requirement: InteractionActionAdmissionTraceRequirement::Required,
            effect_relationship: InteractionActionAdmissionEffectRelationship::NoEffect,
            policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace::WorkbenchLocal,
            effect_eligibility: InteractionSemanticActionDispatchEffectEligibility::NoEffect,
        }
    }

    fn effect_request_descriptor() -> InteractionEffectRequestDescriptor {
        let trace = prepare_effect_trace();
        describe_interaction_effect_request(&trace)
            .expect("effect candidate trace should produce descriptor")
    }

    #[test]
    fn descriptor_produces_trace_report() {
        let descriptor = effect_request_descriptor();

        let trace = trace_interaction_effect_request(&descriptor);

        assert_eq!(trace.descriptor_id, descriptor.id);
        assert_eq!(
            trace.trace_status,
            InteractionEffectRequestTraceStatus::Described
        );
        assert_eq!(
            trace.trace_reason,
            InteractionEffectRequestTraceReason::DescriptorRecorded
        );
    }

    #[test]
    fn trace_preserves_descriptor_identity() {
        let descriptor = effect_request_descriptor();

        let trace = trace_interaction_effect_request(&descriptor);

        assert_eq!(trace.descriptor_id, descriptor.id);
        assert_eq!(
            trace.source_admitted_action_id,
            descriptor.source_admitted_action_id
        );
        assert_eq!(trace.dispatch_record_id, descriptor.dispatch_record_id);
        assert_eq!(trace.dispatch_route_id, descriptor.dispatch_route_id);
    }

    #[test]
    fn trace_preserves_source_action_effect_metadata() {
        let descriptor = effect_request_descriptor();

        let trace = trace_interaction_effect_request(&descriptor);

        assert_eq!(trace.action, descriptor.action);
        assert_eq!(trace.source_intent, descriptor.source_intent);
        assert_eq!(trace.requested_effect, descriptor.requested_effect);
    }

    #[test]
    fn trace_preserves_capability_lifecycle_policy_metadata() {
        let descriptor = effect_request_descriptor();

        let trace = trace_interaction_effect_request(&descriptor);

        assert_eq!(trace.target_policy, descriptor.target_policy);
        assert_eq!(
            trace.required_ui_capability,
            descriptor.required_ui_capability
        );
        assert_eq!(
            trace.required_runtime_capability,
            descriptor.required_runtime_capability
        );
        assert_eq!(
            trace.lifecycle_precondition,
            descriptor.lifecycle_precondition
        );
        assert_eq!(trace.trace_requirement, descriptor.trace_requirement);
        assert_eq!(trace.denial_behavior, descriptor.denial_behavior);
        assert_eq!(
            trace.prepare_commit_relationship,
            descriptor.prepare_commit_relationship
        );
        assert_eq!(
            trace.policy_gate_namespace,
            descriptor.policy_gate_namespace
        );
        assert_eq!(trace.scope, descriptor.scope);
    }

    #[test]
    fn trace_is_not_authority() {
        let descriptor = effect_request_descriptor();

        let trace = trace_interaction_effect_request(&descriptor);

        assert!(!trace.is_capability_admission());
        assert!(!trace.is_prepared_effect());
        assert!(!trace.is_committed_effect());
        assert!(!trace.is_execution_authority());
        assert!(!trace.is_runtime_mutation());
        assert!(!trace.is_audit_authority());
        assert!(!trace.calls_vm_or_host_abi());
    }

    #[test]
    fn close_window_trace_does_not_produce_descriptor() {
        let trace = close_window_no_effect_trace();

        let descriptor = describe_interaction_effect_request(&trace);

        assert!(descriptor.is_none());
    }

    #[test]
    fn descriptor_generation_is_deterministic() {
        let descriptor = effect_request_descriptor();

        let first = trace_interaction_effect_request(&descriptor);
        let second = trace_interaction_effect_request(&descriptor);

        assert_eq!(first, second);
    }
}
