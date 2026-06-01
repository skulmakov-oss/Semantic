//! Semantic UI runtime capability mapping descriptor scaffold.
//!
//! This module records an inert descriptor for the future runtime capability
//! mapping boundary. It does not implement a mapping result, runtime capability
//! grant, Host ABI calls, VM calls, prepared effects, committed effects,
//! effect execution, or runtime mutation.

use crate::action_admission::{
    InteractionActionAdmissionPolicyGateNamespace, InteractionActionAdmissionTraceRequirement,
};
use crate::action_dispatch_record::InteractionSemanticActionDispatchRecordId;
use crate::action_dispatch_route::InteractionSemanticActionDispatchRouteId;
use crate::admitted_action::InteractionAdmittedSemanticActionId;
use crate::effect_request::{
    InteractionEffectRequestDescriptorId, InteractionEffectRequestKind,
    InteractionEffectRequestLifecyclePrecondition, InteractionEffectRequestRuntimeCapability,
    InteractionEffectRequestScope, InteractionEffectRequestTargetPolicy,
    InteractionEffectRequestUiCapability,
};
use crate::ui_capability_admission::{
    InteractionUiCapabilityAdmissionDescriptorId,
    InteractionUiCapabilityAdmissionRuntimeMappingRequirement,
};
use crate::ui_capability_admission_result::{
    InteractionUiCapabilityAdmissionDecisionStatus, InteractionUiCapabilityAdmissionResult,
    InteractionUiCapabilityAdmissionResultId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InteractionRuntimeCapabilityMappingDescriptorId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionRuntimeCapabilityNamespace {
    Window,
    EffectControl,
    TargetQuarantine,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionRuntimeCapabilityMappingFutureResultShape {
    MapOrDenyWithReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionRuntimeCapabilityMappingDescriptor {
    pub id: InteractionRuntimeCapabilityMappingDescriptorId,
    pub ui_capability_admission_result_id: InteractionUiCapabilityAdmissionResultId,
    pub ui_capability_admission_descriptor_id: InteractionUiCapabilityAdmissionDescriptorId,
    pub effect_request_descriptor_id: InteractionEffectRequestDescriptorId,
    pub source_admitted_action_id: InteractionAdmittedSemanticActionId,
    pub dispatch_record_id: InteractionSemanticActionDispatchRecordId,
    pub dispatch_route_id: InteractionSemanticActionDispatchRouteId,
    pub requested_effect: InteractionEffectRequestKind,
    pub declared_ui_capability: InteractionEffectRequestUiCapability,
    pub declared_runtime_capability_requirement: InteractionEffectRequestRuntimeCapability,
    pub runtime_mapping_requirement: InteractionUiCapabilityAdmissionRuntimeMappingRequirement,
    pub runtime_capability_namespace: InteractionRuntimeCapabilityNamespace,
    pub lifecycle_precondition: InteractionEffectRequestLifecyclePrecondition,
    pub target_policy: InteractionEffectRequestTargetPolicy,
    pub trace_requirement: InteractionActionAdmissionTraceRequirement,
    pub policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace,
    pub scope: InteractionEffectRequestScope,
    pub future_result_shape: InteractionRuntimeCapabilityMappingFutureResultShape,
}

pub fn describe_interaction_runtime_capability_mapping(
    result: &InteractionUiCapabilityAdmissionResult,
) -> Option<InteractionRuntimeCapabilityMappingDescriptor> {
    if !matches!(
        result.status,
        InteractionUiCapabilityAdmissionDecisionStatus::Admitted
    ) {
        return None;
    }

    if !matches!(
        result.runtime_mapping_requirement,
        InteractionUiCapabilityAdmissionRuntimeMappingRequirement::Required
    ) {
        return None;
    }

    if matches!(
        result.declared_runtime_capability_requirement,
        InteractionEffectRequestRuntimeCapability::Unknown
    ) {
        return None;
    }

    Some(InteractionRuntimeCapabilityMappingDescriptor {
        id: InteractionRuntimeCapabilityMappingDescriptorId(result.id.0),
        ui_capability_admission_result_id: result.id,
        ui_capability_admission_descriptor_id: result.descriptor_id,
        effect_request_descriptor_id: result.effect_request_descriptor_id,
        source_admitted_action_id: result.source_admitted_action_id,
        dispatch_record_id: result.dispatch_record_id,
        dispatch_route_id: result.dispatch_route_id,
        requested_effect: result.requested_effect,
        declared_ui_capability: result.declared_ui_capability,
        declared_runtime_capability_requirement: result.declared_runtime_capability_requirement,
        runtime_mapping_requirement: result.runtime_mapping_requirement,
        runtime_capability_namespace: map_runtime_capability_namespace(
            result.declared_runtime_capability_requirement,
        ),
        lifecycle_precondition: result.lifecycle_precondition,
        target_policy: result.target_policy,
        trace_requirement: result.trace_requirement,
        policy_gate_namespace: result.policy_gate_namespace,
        scope: result.scope,
        future_result_shape:
            InteractionRuntimeCapabilityMappingFutureResultShape::MapOrDenyWithReason,
    })
}

const fn map_runtime_capability_namespace(
    runtime_capability: InteractionEffectRequestRuntimeCapability,
) -> InteractionRuntimeCapabilityNamespace {
    match runtime_capability {
        InteractionEffectRequestRuntimeCapability::WindowLifecycle => {
            InteractionRuntimeCapabilityNamespace::Window
        }
        InteractionEffectRequestRuntimeCapability::EffectControl => {
            InteractionRuntimeCapabilityNamespace::EffectControl
        }
        InteractionEffectRequestRuntimeCapability::TargetQuarantine => {
            InteractionRuntimeCapabilityNamespace::TargetQuarantine
        }
        InteractionEffectRequestRuntimeCapability::Unknown => {
            InteractionRuntimeCapabilityNamespace::Unknown
        }
    }
}

impl InteractionRuntimeCapabilityMappingDescriptor {
    pub const fn grants_runtime_capability(&self) -> bool {
        false
    }

    pub const fn is_mapping_result(&self) -> bool {
        false
    }

    pub const fn is_host_abi_authority(&self) -> bool {
        false
    }

    pub const fn is_vm_authority(&self) -> bool {
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
    use crate::effect_request::describe_interaction_effect_request;
    use crate::interaction::InteractionIntentKind;
    use crate::ui_capability_admission::describe_interaction_ui_capability_admission;
    use crate::ui_capability_admission_result::{
        record_interaction_ui_capability_admitted_result,
        record_interaction_ui_capability_denied_result,
        InteractionUiCapabilityAdmissionDenialReason,
        InteractionUiCapabilityAdmissionMissingRequirement,
    };

    fn effect_request_descriptor(
        action: InteractionActionName,
        record_id: u64,
    ) -> crate::effect_request::InteractionEffectRequestDescriptor {
        let dispatch_trace = InteractionSemanticActionDispatchTraceReport {
            record_id: InteractionSemanticActionDispatchRecordId(record_id),
            route_id: InteractionSemanticActionDispatchRouteId(record_id),
            admitted_action_id: InteractionAdmittedSemanticActionId(record_id),
            action,
            source_intent: InteractionIntentKind::Submit,
            binding_id: InteractionActionBindingId(record_id),
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
        };

        describe_interaction_effect_request(&dispatch_trace)
            .expect("effect candidate trace should produce descriptor")
    }

    fn admitted_result(
        action: InteractionActionName,
        record_id: u64,
    ) -> InteractionUiCapabilityAdmissionResult {
        let effect_request = effect_request_descriptor(action, record_id);
        let descriptor = describe_interaction_ui_capability_admission(&effect_request);

        record_interaction_ui_capability_admitted_result(&descriptor)
    }

    fn denied_result(
        action: InteractionActionName,
        record_id: u64,
        reason: InteractionUiCapabilityAdmissionDenialReason,
        missing: InteractionUiCapabilityAdmissionMissingRequirement,
    ) -> InteractionUiCapabilityAdmissionResult {
        let effect_request = effect_request_descriptor(action, record_id);
        let descriptor = describe_interaction_ui_capability_admission(&effect_request);

        record_interaction_ui_capability_denied_result(&descriptor, reason, missing)
    }

    #[test]
    fn admitted_required_runtime_mapping_creates_descriptor() {
        let result = admitted_result(InteractionActionName::PrepareEffect, 71);

        let descriptor = describe_interaction_runtime_capability_mapping(&result)
            .expect("admitted required result should produce mapping descriptor");

        assert_eq!(
            descriptor.id,
            InteractionRuntimeCapabilityMappingDescriptorId(71)
        );
        assert_eq!(descriptor.ui_capability_admission_result_id, result.id);
    }

    #[test]
    fn denied_result_returns_none() {
        let result = denied_result(
            InteractionActionName::PrepareEffect,
            72,
            InteractionUiCapabilityAdmissionDenialReason::PolicyDenied,
            InteractionUiCapabilityAdmissionMissingRequirement::Policy,
        );

        let descriptor = describe_interaction_runtime_capability_mapping(&result);

        assert!(descriptor.is_none());
    }

    #[test]
    fn admitted_unknown_runtime_capability_returns_none() {
        let result = admitted_result(InteractionActionName::OpenInspector, 73);

        let descriptor = describe_interaction_runtime_capability_mapping(&result);

        assert!(descriptor.is_none());
    }

    #[test]
    fn admitted_non_required_runtime_mapping_returns_none() {
        let result = admitted_result(InteractionActionName::OpenInspector, 74);
        let mut result = result;
        result.runtime_mapping_requirement =
            InteractionUiCapabilityAdmissionRuntimeMappingRequirement::NotRequired;

        let descriptor = describe_interaction_runtime_capability_mapping(&result);

        assert!(descriptor.is_none());
    }

    #[test]
    fn descriptor_preserves_identity_and_capability_metadata() {
        let result = admitted_result(InteractionActionName::PrepareEffect, 75);

        let descriptor = describe_interaction_runtime_capability_mapping(&result)
            .expect("admitted required result should produce mapping descriptor");

        assert_eq!(descriptor.ui_capability_admission_result_id, result.id);
        assert_eq!(
            descriptor.ui_capability_admission_descriptor_id,
            result.descriptor_id
        );
        assert_eq!(
            descriptor.effect_request_descriptor_id,
            result.effect_request_descriptor_id
        );
        assert_eq!(
            descriptor.source_admitted_action_id,
            result.source_admitted_action_id
        );
        assert_eq!(descriptor.dispatch_record_id, result.dispatch_record_id);
        assert_eq!(descriptor.dispatch_route_id, result.dispatch_route_id);
        assert_eq!(descriptor.requested_effect, result.requested_effect);
        assert_eq!(
            descriptor.declared_ui_capability,
            result.declared_ui_capability
        );
        assert_eq!(
            descriptor.declared_runtime_capability_requirement,
            result.declared_runtime_capability_requirement
        );
        assert_eq!(
            descriptor.runtime_mapping_requirement,
            result.runtime_mapping_requirement
        );
        assert_eq!(
            descriptor.lifecycle_precondition,
            result.lifecycle_precondition
        );
        assert_eq!(descriptor.target_policy, result.target_policy);
        assert_eq!(descriptor.trace_requirement, result.trace_requirement);
        assert_eq!(
            descriptor.policy_gate_namespace,
            result.policy_gate_namespace
        );
        assert_eq!(descriptor.scope, result.scope);
        assert_eq!(
            descriptor.future_result_shape,
            InteractionRuntimeCapabilityMappingFutureResultShape::MapOrDenyWithReason
        );
    }

    #[test]
    fn runtime_capability_namespace_mapping_is_explicit() {
        let window = admitted_result(InteractionActionName::CloseWindow, 76);
        let effect_control = admitted_result(InteractionActionName::PrepareEffect, 77);
        let target_quarantine = admitted_result(InteractionActionName::QuarantineTarget, 78);

        let window_descriptor = describe_interaction_runtime_capability_mapping(&window)
            .expect("window lifecycle should produce mapping descriptor");
        let effect_control_descriptor =
            describe_interaction_runtime_capability_mapping(&effect_control)
                .expect("effect control should produce mapping descriptor");
        let target_quarantine_descriptor =
            describe_interaction_runtime_capability_mapping(&target_quarantine)
                .expect("target quarantine should produce mapping descriptor");

        assert_eq!(
            window_descriptor.runtime_capability_namespace,
            InteractionRuntimeCapabilityNamespace::Window
        );
        assert_eq!(
            effect_control_descriptor.runtime_capability_namespace,
            InteractionRuntimeCapabilityNamespace::EffectControl
        );
        assert_eq!(
            target_quarantine_descriptor.runtime_capability_namespace,
            InteractionRuntimeCapabilityNamespace::TargetQuarantine
        );
    }

    #[test]
    fn descriptor_is_not_authority() {
        let result = admitted_result(InteractionActionName::PrepareEffect, 79);

        let descriptor = describe_interaction_runtime_capability_mapping(&result)
            .expect("admitted required result should produce mapping descriptor");

        assert!(!descriptor.grants_runtime_capability());
        assert!(!descriptor.is_mapping_result());
        assert!(!descriptor.is_host_abi_authority());
        assert!(!descriptor.is_vm_authority());
        assert!(!descriptor.is_prepared_effect());
        assert!(!descriptor.is_committed_effect());
        assert!(!descriptor.is_execution_authority());
        assert!(!descriptor.is_runtime_mutation());
    }

    #[test]
    fn generation_is_deterministic() {
        let result = admitted_result(InteractionActionName::PrepareEffect, 80);

        let first = describe_interaction_runtime_capability_mapping(&result);
        let second = describe_interaction_runtime_capability_mapping(&result);

        assert_eq!(first, second);
    }
}
