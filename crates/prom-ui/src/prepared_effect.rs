//! Semantic UI prepared effect descriptor scaffold.
//!
//! This module records an inert descriptor for the future prepared effect
//! boundary. It does not implement prepared effect execution, committed
//! effects, Host ABI calls, VM calls, or runtime mutation.

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
use crate::runtime_capability_mapping::{
    InteractionRuntimeCapabilityMappingDescriptorId,
    InteractionRuntimeCapabilityNamespace,
};
use crate::runtime_capability_mapping_result::{
    InteractionRuntimeCapabilityMappingDecisionStatus,
    InteractionRuntimeCapabilityMappingResult,
    InteractionRuntimeCapabilityMappingResultId,
};
use crate::ui_capability_admission::InteractionUiCapabilityAdmissionDescriptorId;
use crate::ui_capability_admission_result::InteractionUiCapabilityAdmissionResultId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InteractionPreparedEffectDescriptorId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionPreparedEffectStatusShape {
    PreparedOrDeniedWithReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionPreparedEffectCommitRequirement {
    Required,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionPreparedEffectDescriptor {
    pub id: InteractionPreparedEffectDescriptorId,
    pub runtime_capability_mapping_result_id: InteractionRuntimeCapabilityMappingResultId,
    pub runtime_capability_mapping_descriptor_id: InteractionRuntimeCapabilityMappingDescriptorId,
    pub ui_capability_admission_result_id: InteractionUiCapabilityAdmissionResultId,
    pub ui_capability_admission_descriptor_id: InteractionUiCapabilityAdmissionDescriptorId,
    pub effect_request_descriptor_id: InteractionEffectRequestDescriptorId,
    pub source_admitted_action_id: InteractionAdmittedSemanticActionId,
    pub dispatch_record_id: InteractionSemanticActionDispatchRecordId,
    pub dispatch_route_id: InteractionSemanticActionDispatchRouteId,
    pub requested_effect: InteractionEffectRequestKind,
    pub declared_ui_capability: InteractionEffectRequestUiCapability,
    pub declared_runtime_capability_requirement: InteractionEffectRequestRuntimeCapability,
    pub runtime_capability_namespace: InteractionRuntimeCapabilityNamespace,
    pub lifecycle_precondition: InteractionEffectRequestLifecyclePrecondition,
    pub target_policy: InteractionEffectRequestTargetPolicy,
    pub trace_requirement: InteractionActionAdmissionTraceRequirement,
    pub policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace,
    pub scope: InteractionEffectRequestScope,
    pub prepare_status_shape: InteractionPreparedEffectStatusShape,
    pub future_commit_requirement: InteractionPreparedEffectCommitRequirement,
}

pub fn describe_interaction_prepared_effect(
    mapping_result: &InteractionRuntimeCapabilityMappingResult,
) -> Option<InteractionPreparedEffectDescriptor> {
    if !matches!(
        mapping_result.status,
        InteractionRuntimeCapabilityMappingDecisionStatus::Mapped
    ) {
        return None;
    }

    Some(InteractionPreparedEffectDescriptor {
        id: InteractionPreparedEffectDescriptorId(mapping_result.id.0),
        runtime_capability_mapping_result_id: mapping_result.id,
        runtime_capability_mapping_descriptor_id: mapping_result.descriptor_id,
        ui_capability_admission_result_id: mapping_result.ui_capability_admission_result_id,
        ui_capability_admission_descriptor_id: mapping_result.ui_capability_admission_descriptor_id,
        effect_request_descriptor_id: mapping_result.effect_request_descriptor_id,
        source_admitted_action_id: mapping_result.source_admitted_action_id,
        dispatch_record_id: mapping_result.dispatch_record_id,
        dispatch_route_id: mapping_result.dispatch_route_id,
        requested_effect: mapping_result.requested_effect,
        declared_ui_capability: mapping_result.declared_ui_capability,
        declared_runtime_capability_requirement: mapping_result
            .declared_runtime_capability_requirement,
        runtime_capability_namespace: mapping_result.runtime_capability_namespace,
        lifecycle_precondition: mapping_result.lifecycle_precondition,
        target_policy: mapping_result.target_policy,
        trace_requirement: mapping_result.trace_requirement,
        policy_gate_namespace: mapping_result.policy_gate_namespace,
        scope: mapping_result.scope,
        prepare_status_shape: InteractionPreparedEffectStatusShape::PreparedOrDeniedWithReason,
        future_commit_requirement: InteractionPreparedEffectCommitRequirement::Required,
    })
}

impl InteractionPreparedEffectDescriptor {
    pub const fn is_prepared_effect_execution(&self) -> bool {
        false
    }

    pub const fn is_committed_effect(&self) -> bool {
        false
    }

    pub const fn is_commit_boundary(&self) -> bool {
        false
    }

    pub const fn is_host_abi_authority(&self) -> bool {
        false
    }

    pub const fn is_vm_authority(&self) -> bool {
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
    use crate::action_admission::InteractionActionAdmissionEffectRelationship;
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
    use crate::runtime_capability_mapping::describe_interaction_runtime_capability_mapping;
    use crate::runtime_capability_mapping_result::{
        record_interaction_runtime_capability_denied_result,
        record_interaction_runtime_capability_mapped_result,
        InteractionRuntimeCapabilityMappingDenialReason,
        InteractionRuntimeCapabilityMappingMissingRequirement,
    };
    use crate::ui_capability_admission::describe_interaction_ui_capability_admission;

    fn effect_request_descriptor() -> crate::effect_request::InteractionEffectRequestDescriptor {
        let dispatch_trace = InteractionSemanticActionDispatchTraceReport {
            record_id: InteractionSemanticActionDispatchRecordId(101),
            route_id: InteractionSemanticActionDispatchRouteId(101),
            admitted_action_id: InteractionAdmittedSemanticActionId(101),
            action: InteractionActionName::PrepareEffect,
            source_intent: InteractionIntentKind::Submit,
            binding_id: InteractionActionBindingId(101),
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

    fn mapping_result_mapped() -> InteractionRuntimeCapabilityMappingResult {
        let effect_request = effect_request_descriptor();
        let admission = describe_interaction_ui_capability_admission(&effect_request);
        let mapping_descriptor = describe_interaction_runtime_capability_mapping(
            &crate::ui_capability_admission_result::record_interaction_ui_capability_admitted_result(&admission),
        )
        .expect("admitted result should produce mapping descriptor");

        record_interaction_runtime_capability_mapped_result(&mapping_descriptor)
    }

    fn mapping_result_denied() -> InteractionRuntimeCapabilityMappingResult {
        let effect_request = effect_request_descriptor();
        let admission = describe_interaction_ui_capability_admission(&effect_request);
        let mapping_descriptor = describe_interaction_runtime_capability_mapping(
            &crate::ui_capability_admission_result::record_interaction_ui_capability_admitted_result(&admission),
        )
        .expect("admitted result should produce mapping descriptor");

        record_interaction_runtime_capability_denied_result(
            &mapping_descriptor,
            InteractionRuntimeCapabilityMappingDenialReason::PolicyDenied,
            InteractionRuntimeCapabilityMappingMissingRequirement::Policy,
        )
    }

    #[test]
    fn mapped_runtime_capability_result_creates_prepared_effect_descriptor() {
        let mapping_result = mapping_result_mapped();

        let descriptor = describe_interaction_prepared_effect(&mapping_result)
            .expect("mapped runtime mapping result should produce prepared effect descriptor");

        assert_eq!(descriptor.id, InteractionPreparedEffectDescriptorId(101));
        assert_eq!(
            descriptor.runtime_capability_mapping_result_id,
            mapping_result.id
        );
        assert_eq!(
            descriptor.future_commit_requirement,
            InteractionPreparedEffectCommitRequirement::Required
        );
    }

    #[test]
    fn denied_runtime_capability_result_returns_none() {
        let mapping_result = mapping_result_denied();

        let descriptor = describe_interaction_prepared_effect(&mapping_result);

        assert!(descriptor.is_none());
    }

    #[test]
    fn descriptor_preserves_source_and_capability_metadata() {
        let mapping_result = mapping_result_mapped();

        let descriptor = describe_interaction_prepared_effect(&mapping_result)
            .expect("mapped runtime mapping result should produce prepared effect descriptor");

        assert_eq!(
            descriptor.runtime_capability_mapping_result_id,
            mapping_result.id
        );
        assert_eq!(
            descriptor.runtime_capability_mapping_descriptor_id,
            mapping_result.descriptor_id
        );
        assert_eq!(
            descriptor.ui_capability_admission_result_id,
            mapping_result.ui_capability_admission_result_id
        );
        assert_eq!(
            descriptor.ui_capability_admission_descriptor_id,
            mapping_result.ui_capability_admission_descriptor_id
        );
        assert_eq!(
            descriptor.effect_request_descriptor_id,
            mapping_result.effect_request_descriptor_id
        );
        assert_eq!(
            descriptor.source_admitted_action_id,
            mapping_result.source_admitted_action_id
        );
        assert_eq!(descriptor.dispatch_record_id, mapping_result.dispatch_record_id);
        assert_eq!(descriptor.dispatch_route_id, mapping_result.dispatch_route_id);
        assert_eq!(descriptor.requested_effect, mapping_result.requested_effect);
        assert_eq!(
            descriptor.declared_ui_capability,
            mapping_result.declared_ui_capability
        );
        assert_eq!(
            descriptor.declared_runtime_capability_requirement,
            mapping_result.declared_runtime_capability_requirement
        );
        assert_eq!(
            descriptor.runtime_capability_namespace,
            mapping_result.runtime_capability_namespace
        );
        assert_eq!(
            descriptor.lifecycle_precondition,
            mapping_result.lifecycle_precondition
        );
        assert_eq!(descriptor.target_policy, mapping_result.target_policy);
        assert_eq!(descriptor.trace_requirement, mapping_result.trace_requirement);
        assert_eq!(descriptor.policy_gate_namespace, mapping_result.policy_gate_namespace);
        assert_eq!(descriptor.scope, mapping_result.scope);
    }

    #[test]
    fn descriptor_records_prepare_status_shape_and_commit_requirement() {
        let mapping_result = mapping_result_mapped();

        let descriptor = describe_interaction_prepared_effect(&mapping_result)
            .expect("mapped runtime mapping result should produce prepared effect descriptor");

        assert_eq!(
            descriptor.prepare_status_shape,
            InteractionPreparedEffectStatusShape::PreparedOrDeniedWithReason
        );
        assert_eq!(
            descriptor.future_commit_requirement,
            InteractionPreparedEffectCommitRequirement::Required
        );
    }

    #[test]
    fn descriptor_is_not_authority() {
        let mapping_result = mapping_result_mapped();

        let descriptor = describe_interaction_prepared_effect(&mapping_result)
            .expect("mapped runtime mapping result should produce prepared effect descriptor");

        assert!(!descriptor.is_prepared_effect_execution());
        assert!(!descriptor.is_committed_effect());
        assert!(!descriptor.is_commit_boundary());
        assert!(!descriptor.is_host_abi_authority());
        assert!(!descriptor.is_vm_authority());
        assert!(!descriptor.is_execution_authority());
        assert!(!descriptor.is_runtime_mutation());
    }

    #[test]
    fn deterministic_descriptor_generation() {
        let mapping_result = mapping_result_mapped();

        let first = describe_interaction_prepared_effect(&mapping_result);
        let second = describe_interaction_prepared_effect(&mapping_result);

        assert_eq!(first, second);
    }
}
