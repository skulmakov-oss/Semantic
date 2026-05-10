//! Semantic UI capability admission result scaffold.
//!
//! This module records an inert decision outcome for a UI capability
//! admission descriptor. It does not implement capability checking, UI
//! capability grants, runtime capability grants, runtime mapping, prepared
//! effects, committed effects, VM/Host ABI calls, or runtime mutation.

use crate::action_admission::{
    InteractionActionAdmissionPolicyGateNamespace, InteractionActionAdmissionTraceRequirement,
};
use crate::action_dispatch_record::InteractionSemanticActionDispatchRecordId;
use crate::action_dispatch_route::InteractionSemanticActionDispatchRouteId;
use crate::admitted_action::InteractionAdmittedSemanticActionId;
use crate::effect_request::{
    InteractionEffectRequestDenialBehavior, InteractionEffectRequestDescriptorId,
    InteractionEffectRequestKind, InteractionEffectRequestLifecyclePrecondition,
    InteractionEffectRequestRuntimeCapability, InteractionEffectRequestScope,
    InteractionEffectRequestTargetPolicy, InteractionEffectRequestUiCapability,
};
use crate::ui_capability_admission::{
    InteractionUiCapabilityAdmissionDescriptor,
    InteractionUiCapabilityAdmissionDescriptorId,
    InteractionUiCapabilityAdmissionRuntimeMappingRequirement,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InteractionUiCapabilityAdmissionResultId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionUiCapabilityAdmissionDecisionStatus {
    Admitted,
    Denied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionUiCapabilityAdmissionDenialReason {
    None,
    MissingUiCapability,
    LifecycleBlocked,
    TargetUnavailable,
    TargetInvalid,
    PolicyDenied,
    RuntimeMappingRequired,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionUiCapabilityAdmissionMissingRequirement {
    None,
    UiCapability,
    Lifecycle,
    Target,
    Policy,
    RuntimeMapping,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionUiCapabilityAdmissionResult {
    pub id: InteractionUiCapabilityAdmissionResultId,
    pub descriptor_id: InteractionUiCapabilityAdmissionDescriptorId,
    pub effect_request_descriptor_id: InteractionEffectRequestDescriptorId,
    pub source_admitted_action_id: InteractionAdmittedSemanticActionId,
    pub dispatch_record_id: InteractionSemanticActionDispatchRecordId,
    pub dispatch_route_id: InteractionSemanticActionDispatchRouteId,
    pub status: InteractionUiCapabilityAdmissionDecisionStatus,
    pub denial_reason: InteractionUiCapabilityAdmissionDenialReason,
    pub missing_requirement: InteractionUiCapabilityAdmissionMissingRequirement,
    pub requested_effect: InteractionEffectRequestKind,
    pub declared_ui_capability: InteractionEffectRequestUiCapability,
    pub declared_runtime_capability_requirement: InteractionEffectRequestRuntimeCapability,
    pub lifecycle_precondition: InteractionEffectRequestLifecyclePrecondition,
    pub target_policy: InteractionEffectRequestTargetPolicy,
    pub denial_behavior: InteractionEffectRequestDenialBehavior,
    pub trace_requirement: InteractionActionAdmissionTraceRequirement,
    pub policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace,
    pub scope: InteractionEffectRequestScope,
    pub runtime_mapping_requirement: InteractionUiCapabilityAdmissionRuntimeMappingRequirement,
}

pub fn record_interaction_ui_capability_admitted_result(
    descriptor: &InteractionUiCapabilityAdmissionDescriptor,
) -> InteractionUiCapabilityAdmissionResult {
    build_result(
        descriptor,
        InteractionUiCapabilityAdmissionDecisionStatus::Admitted,
        InteractionUiCapabilityAdmissionDenialReason::None,
        InteractionUiCapabilityAdmissionMissingRequirement::None,
    )
}

pub fn record_interaction_ui_capability_denied_result(
    descriptor: &InteractionUiCapabilityAdmissionDescriptor,
    denial_reason: InteractionUiCapabilityAdmissionDenialReason,
    missing_requirement: InteractionUiCapabilityAdmissionMissingRequirement,
) -> InteractionUiCapabilityAdmissionResult {
    build_result(
        descriptor,
        InteractionUiCapabilityAdmissionDecisionStatus::Denied,
        denial_reason,
        missing_requirement,
    )
}

fn build_result(
    descriptor: &InteractionUiCapabilityAdmissionDescriptor,
    status: InteractionUiCapabilityAdmissionDecisionStatus,
    denial_reason: InteractionUiCapabilityAdmissionDenialReason,
    missing_requirement: InteractionUiCapabilityAdmissionMissingRequirement,
) -> InteractionUiCapabilityAdmissionResult {
    InteractionUiCapabilityAdmissionResult {
        id: InteractionUiCapabilityAdmissionResultId(descriptor.id.0),
        descriptor_id: descriptor.id,
        effect_request_descriptor_id: descriptor.effect_request_descriptor_id,
        source_admitted_action_id: descriptor.source_admitted_action_id,
        dispatch_record_id: descriptor.dispatch_record_id,
        dispatch_route_id: descriptor.dispatch_route_id,
        status,
        denial_reason,
        missing_requirement,
        requested_effect: descriptor.requested_effect,
        declared_ui_capability: descriptor.declared_ui_capability,
        declared_runtime_capability_requirement: descriptor
            .declared_runtime_capability_requirement,
        lifecycle_precondition: descriptor.lifecycle_precondition,
        target_policy: descriptor.target_policy,
        denial_behavior: descriptor.denial_behavior,
        trace_requirement: descriptor.trace_requirement,
        policy_gate_namespace: descriptor.policy_gate_namespace,
        scope: descriptor.scope,
        runtime_mapping_requirement: descriptor.runtime_mapping_requirement,
    }
}

impl InteractionUiCapabilityAdmissionResult {
    pub const fn is_admitted(&self) -> bool {
        matches!(self.status, InteractionUiCapabilityAdmissionDecisionStatus::Admitted)
    }

    pub const fn is_denied(&self) -> bool {
        matches!(self.status, InteractionUiCapabilityAdmissionDecisionStatus::Denied)
    }

    pub const fn grants_ui_capability(&self) -> bool {
        false
    }

    pub const fn grants_runtime_capability(&self) -> bool {
        false
    }

    pub const fn is_runtime_mapping(&self) -> bool {
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
    use crate::effect_request::describe_interaction_effect_request;
    use crate::interaction::InteractionIntentKind;
    use crate::ui_capability_admission::describe_interaction_ui_capability_admission;

    fn descriptor() -> InteractionUiCapabilityAdmissionDescriptor {
        let dispatch_trace = InteractionSemanticActionDispatchTraceReport {
            record_id: InteractionSemanticActionDispatchRecordId(41),
            route_id: InteractionSemanticActionDispatchRouteId(41),
            admitted_action_id: InteractionAdmittedSemanticActionId(41),
            action: InteractionActionName::PrepareEffect,
            source_intent: InteractionIntentKind::Submit,
            binding_id: InteractionActionBindingId(41),
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
        };

        let effect_request = describe_interaction_effect_request(&dispatch_trace)
            .expect("effect candidate trace should produce descriptor");

        describe_interaction_ui_capability_admission(&effect_request)
    }

    fn unknown_runtime_mapping_descriptor() -> InteractionUiCapabilityAdmissionDescriptor {
        let mut trace = InteractionSemanticActionDispatchTraceReport {
            record_id: InteractionSemanticActionDispatchRecordId(42),
            route_id: InteractionSemanticActionDispatchRouteId(42),
            admitted_action_id: InteractionAdmittedSemanticActionId(42),
            action: InteractionActionName::OpenInspector,
            source_intent: InteractionIntentKind::Submit,
            binding_id: InteractionActionBindingId(42),
            route: InteractionSemanticActionDispatchRouteKind::EffectRequestCandidate,
            record_status: InteractionSemanticActionDispatchRecordStatus::Recorded,
            trace_status: InteractionSemanticActionDispatchTraceStatus::Recorded,
            block_reason: InteractionSemanticActionDispatchBlockReason::None,
            trace_reason: InteractionSemanticActionDispatchTraceReason::RouteRecorded,
            trace_requirement: InteractionActionAdmissionTraceRequirement::Required,
            effect_relationship:
                crate::action_admission::InteractionActionAdmissionEffectRelationship::NoEffect,
            policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace::WorkbenchLocal,
            effect_eligibility:
                InteractionSemanticActionDispatchEffectEligibility::RequiresFutureEffectBoundary,
        };

        trace.action = InteractionActionName::OpenInspector;

        let effect_request = describe_interaction_effect_request(&trace)
            .expect("effect candidate trace should produce descriptor");

        describe_interaction_ui_capability_admission(&effect_request)
    }

    #[test]
    fn admitted_result_is_built_from_descriptor() {
        let descriptor = descriptor();

        let result = record_interaction_ui_capability_admitted_result(&descriptor);

        assert_eq!(result.status, InteractionUiCapabilityAdmissionDecisionStatus::Admitted);
        assert_eq!(result.descriptor_id, descriptor.id);
        assert_eq!(result.id, InteractionUiCapabilityAdmissionResultId(descriptor.id.0));
    }

    #[test]
    fn denied_result_is_built_from_descriptor() {
        let descriptor = descriptor();

        let result = record_interaction_ui_capability_denied_result(
            &descriptor,
            InteractionUiCapabilityAdmissionDenialReason::PolicyDenied,
            InteractionUiCapabilityAdmissionMissingRequirement::Policy,
        );

        assert_eq!(result.status, InteractionUiCapabilityAdmissionDecisionStatus::Denied);
        assert_eq!(result.descriptor_id, descriptor.id);
        assert_eq!(result.id, InteractionUiCapabilityAdmissionResultId(descriptor.id.0));
    }

    #[test]
    fn result_preserves_descriptor_and_capability_metadata() {
        let descriptor = descriptor();

        let result = record_interaction_ui_capability_admitted_result(&descriptor);

        assert_eq!(
            result.effect_request_descriptor_id,
            descriptor.effect_request_descriptor_id
        );
        assert_eq!(
            result.source_admitted_action_id,
            descriptor.source_admitted_action_id
        );
        assert_eq!(result.dispatch_record_id, descriptor.dispatch_record_id);
        assert_eq!(result.dispatch_route_id, descriptor.dispatch_route_id);
        assert_eq!(result.requested_effect, descriptor.requested_effect);
        assert_eq!(
            result.declared_ui_capability,
            descriptor.declared_ui_capability
        );
        assert_eq!(
            result.declared_runtime_capability_requirement,
            descriptor.declared_runtime_capability_requirement
        );
        assert_eq!(
            result.lifecycle_precondition,
            descriptor.lifecycle_precondition
        );
        assert_eq!(result.target_policy, descriptor.target_policy);
        assert_eq!(result.denial_behavior, descriptor.denial_behavior);
        assert_eq!(result.trace_requirement, descriptor.trace_requirement);
        assert_eq!(result.policy_gate_namespace, descriptor.policy_gate_namespace);
        assert_eq!(result.scope, descriptor.scope);
        assert_eq!(
            result.runtime_mapping_requirement,
            descriptor.runtime_mapping_requirement
        );
    }

    #[test]
    fn admitted_result_has_none_denial_metadata() {
        let descriptor = descriptor();

        let result = record_interaction_ui_capability_admitted_result(&descriptor);

        assert_eq!(result.denial_reason, InteractionUiCapabilityAdmissionDenialReason::None);
        assert_eq!(
            result.missing_requirement,
            InteractionUiCapabilityAdmissionMissingRequirement::None
        );
        assert!(result.is_admitted());
        assert!(!result.is_denied());
    }

    #[test]
    fn denied_result_preserves_denial_metadata() {
        let descriptor = descriptor();

        let result = record_interaction_ui_capability_denied_result(
            &descriptor,
            InteractionUiCapabilityAdmissionDenialReason::RuntimeMappingRequired,
            InteractionUiCapabilityAdmissionMissingRequirement::RuntimeMapping,
        );

        assert_eq!(
            result.denial_reason,
            InteractionUiCapabilityAdmissionDenialReason::RuntimeMappingRequired
        );
        assert_eq!(
            result.missing_requirement,
            InteractionUiCapabilityAdmissionMissingRequirement::RuntimeMapping
        );
        assert!(result.is_denied());
        assert!(!result.is_admitted());
    }

    #[test]
    fn result_is_not_authority() {
        let descriptor = descriptor();

        let result = record_interaction_ui_capability_admitted_result(&descriptor);

        assert!(!result.grants_ui_capability());
        assert!(!result.grants_runtime_capability());
        assert!(!result.is_runtime_mapping());
        assert!(!result.is_prepared_effect());
        assert!(!result.is_committed_effect());
        assert!(!result.is_execution_authority());
        assert!(!result.is_runtime_mutation());
        assert!(!result.calls_vm_or_host_abi());
    }

    #[test]
    fn runtime_mapping_requirement_is_preserved_as_metadata_only() {
        let descriptor = unknown_runtime_mapping_descriptor();

        let result = record_interaction_ui_capability_denied_result(
            &descriptor,
            InteractionUiCapabilityAdmissionDenialReason::MissingUiCapability,
            InteractionUiCapabilityAdmissionMissingRequirement::UiCapability,
        );

        assert_eq!(
            result.runtime_mapping_requirement,
            descriptor.runtime_mapping_requirement
        );
        assert!(!result.is_runtime_mapping());
    }

    #[test]
    fn generation_is_deterministic() {
        let descriptor = descriptor();

        let first = record_interaction_ui_capability_admitted_result(&descriptor);
        let second = record_interaction_ui_capability_admitted_result(&descriptor);

        assert_eq!(first, second);
    }
}
