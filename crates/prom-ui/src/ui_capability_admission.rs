//! Semantic UI capability admission descriptor scaffold.
//!
//! This module records what a future UI capability admission layer must
//! evaluate before runtime capability mapping or prepared effect construction.
//! It does not implement an admission result, capability grant, runtime
//! capability mapping, prepared effects, committed effects, VM/Host ABI calls,
//! or runtime mutation.

use crate::action_admission::{
    InteractionActionAdmissionPolicyGateNamespace, InteractionActionAdmissionTraceRequirement,
};
use crate::action_dispatch_record::InteractionSemanticActionDispatchRecordId;
use crate::action_dispatch_route::InteractionSemanticActionDispatchRouteId;
use crate::admitted_action::InteractionAdmittedSemanticActionId;
use crate::effect_request::{
    InteractionEffectRequestDenialBehavior, InteractionEffectRequestDescriptor,
    InteractionEffectRequestDescriptorId, InteractionEffectRequestKind,
    InteractionEffectRequestLifecyclePrecondition, InteractionEffectRequestRuntimeCapability,
    InteractionEffectRequestScope, InteractionEffectRequestTargetPolicy,
    InteractionEffectRequestUiCapability,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InteractionUiCapabilityAdmissionDescriptorId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionUiCapabilityAdmissionRuntimeMappingRequirement {
    Required,
    NotRequired,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionUiCapabilityAdmissionFutureResultShape {
    AdmitOrDenyWithReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionUiCapabilityAdmissionDescriptor {
    pub id: InteractionUiCapabilityAdmissionDescriptorId,
    pub effect_request_descriptor_id: InteractionEffectRequestDescriptorId,
    pub source_admitted_action_id: InteractionAdmittedSemanticActionId,
    pub dispatch_record_id: InteractionSemanticActionDispatchRecordId,
    pub dispatch_route_id: InteractionSemanticActionDispatchRouteId,
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
    pub future_result_shape: InteractionUiCapabilityAdmissionFutureResultShape,
}

pub fn describe_interaction_ui_capability_admission(
    effect_request: &InteractionEffectRequestDescriptor,
) -> InteractionUiCapabilityAdmissionDescriptor {
    InteractionUiCapabilityAdmissionDescriptor {
        id: InteractionUiCapabilityAdmissionDescriptorId(effect_request.id.0),
        effect_request_descriptor_id: effect_request.id,
        source_admitted_action_id: effect_request.source_admitted_action_id,
        dispatch_record_id: effect_request.dispatch_record_id,
        dispatch_route_id: effect_request.dispatch_route_id,
        requested_effect: effect_request.requested_effect,
        declared_ui_capability: effect_request.required_ui_capability,
        declared_runtime_capability_requirement: effect_request.required_runtime_capability,
        lifecycle_precondition: effect_request.lifecycle_precondition,
        target_policy: effect_request.target_policy,
        denial_behavior: effect_request.denial_behavior,
        trace_requirement: effect_request.trace_requirement,
        policy_gate_namespace: effect_request.policy_gate_namespace,
        scope: effect_request.scope,
        runtime_mapping_requirement: map_runtime_mapping_requirement(
            effect_request.required_runtime_capability,
        ),
        future_result_shape:
            InteractionUiCapabilityAdmissionFutureResultShape::AdmitOrDenyWithReason,
    }
}

const fn map_runtime_mapping_requirement(
    runtime_capability: InteractionEffectRequestRuntimeCapability,
) -> InteractionUiCapabilityAdmissionRuntimeMappingRequirement {
    match runtime_capability {
        InteractionEffectRequestRuntimeCapability::Unknown => {
            InteractionUiCapabilityAdmissionRuntimeMappingRequirement::Unknown
        }
        InteractionEffectRequestRuntimeCapability::WindowLifecycle
        | InteractionEffectRequestRuntimeCapability::EffectControl
        | InteractionEffectRequestRuntimeCapability::TargetQuarantine => {
            InteractionUiCapabilityAdmissionRuntimeMappingRequirement::Required
        }
    }
}

impl InteractionUiCapabilityAdmissionDescriptor {
    pub const fn is_admission_result(&self) -> bool {
        false
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
    use crate::effect_request::{
        describe_interaction_effect_request, InteractionEffectRequestDescriptor,
    };
    use crate::interaction::InteractionIntentKind;

    fn effect_request_descriptor() -> InteractionEffectRequestDescriptor {
        let dispatch_trace = InteractionSemanticActionDispatchTraceReport {
            record_id: InteractionSemanticActionDispatchRecordId(21),
            route_id: InteractionSemanticActionDispatchRouteId(21),
            admitted_action_id: InteractionAdmittedSemanticActionId(21),
            action: InteractionActionName::PrepareEffect,
            source_intent: InteractionIntentKind::Submit,
            binding_id: InteractionActionBindingId(21),
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

        describe_interaction_effect_request(&dispatch_trace)
            .expect("effect candidate trace should produce descriptor")
    }

    fn unknown_runtime_capability_descriptor() -> InteractionEffectRequestDescriptor {
        let dispatch_trace = InteractionSemanticActionDispatchTraceReport {
            record_id: InteractionSemanticActionDispatchRecordId(22),
            route_id: InteractionSemanticActionDispatchRouteId(22),
            admitted_action_id: InteractionAdmittedSemanticActionId(22),
            action: InteractionActionName::OpenInspector,
            source_intent: InteractionIntentKind::Submit,
            binding_id: InteractionActionBindingId(22),
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

        describe_interaction_effect_request(&dispatch_trace)
            .expect("effect candidate trace should produce descriptor")
    }

    #[test]
    fn descriptor_is_built_from_effect_request_descriptor() {
        let effect_request = effect_request_descriptor();

        let admission = describe_interaction_ui_capability_admission(&effect_request);

        assert_eq!(admission.effect_request_descriptor_id, effect_request.id);
        assert_eq!(
            admission.id,
            InteractionUiCapabilityAdmissionDescriptorId(21)
        );
    }

    #[test]
    fn descriptor_preserves_source_and_capability_metadata() {
        let effect_request = effect_request_descriptor();

        let admission = describe_interaction_ui_capability_admission(&effect_request);

        assert_eq!(
            admission.source_admitted_action_id,
            effect_request.source_admitted_action_id
        );
        assert_eq!(
            admission.dispatch_record_id,
            effect_request.dispatch_record_id
        );
        assert_eq!(
            admission.dispatch_route_id,
            effect_request.dispatch_route_id
        );
        assert_eq!(admission.requested_effect, effect_request.requested_effect);
        assert_eq!(
            admission.declared_ui_capability,
            effect_request.required_ui_capability
        );
        assert_eq!(
            admission.declared_runtime_capability_requirement,
            effect_request.required_runtime_capability
        );
        assert_eq!(
            admission.lifecycle_precondition,
            effect_request.lifecycle_precondition
        );
        assert_eq!(admission.target_policy, effect_request.target_policy);
        assert_eq!(admission.denial_behavior, effect_request.denial_behavior);
        assert_eq!(
            admission.trace_requirement,
            effect_request.trace_requirement
        );
        assert_eq!(
            admission.policy_gate_namespace,
            effect_request.policy_gate_namespace
        );
        assert_eq!(admission.scope, effect_request.scope);
    }

    #[test]
    fn runtime_capability_maps_to_runtime_mapping_requirement() {
        let effect_request = effect_request_descriptor();

        let admission = describe_interaction_ui_capability_admission(&effect_request);

        assert_eq!(
            admission.runtime_mapping_requirement,
            InteractionUiCapabilityAdmissionRuntimeMappingRequirement::Required
        );
        assert_eq!(
            admission.future_result_shape,
            InteractionUiCapabilityAdmissionFutureResultShape::AdmitOrDenyWithReason
        );
    }

    #[test]
    fn unknown_runtime_capability_maps_to_unknown() {
        let effect_request = unknown_runtime_capability_descriptor();

        let admission = describe_interaction_ui_capability_admission(&effect_request);

        assert_eq!(
            admission.runtime_mapping_requirement,
            InteractionUiCapabilityAdmissionRuntimeMappingRequirement::Unknown
        );
    }

    #[test]
    fn descriptor_is_not_authority() {
        let effect_request = effect_request_descriptor();

        let admission = describe_interaction_ui_capability_admission(&effect_request);

        assert!(!admission.is_admission_result());
        assert!(!admission.grants_ui_capability());
        assert!(!admission.grants_runtime_capability());
        assert!(!admission.is_runtime_mapping());
        assert!(!admission.is_prepared_effect());
        assert!(!admission.is_committed_effect());
        assert!(!admission.is_execution_authority());
        assert!(!admission.is_runtime_mutation());
        assert!(!admission.calls_vm_or_host_abi());
    }

    #[test]
    fn descriptor_generation_is_deterministic() {
        let effect_request = effect_request_descriptor();

        let first = describe_interaction_ui_capability_admission(&effect_request);
        let second = describe_interaction_ui_capability_admission(&effect_request);

        assert_eq!(first, second);
    }
}
