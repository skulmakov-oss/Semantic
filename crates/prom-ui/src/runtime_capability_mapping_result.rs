//! Semantic UI runtime capability mapping result scaffold.
//!
//! This module records an inert decision outcome for a runtime capability
//! mapping descriptor. It does not implement runtime capability grant, Host
//! ABI calls, VM calls, prepared effects, committed effects, effect execution,
//! or runtime mutation.

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
    InteractionRuntimeCapabilityMappingDescriptor,
    InteractionRuntimeCapabilityMappingDescriptorId,
    InteractionRuntimeCapabilityNamespace,
};
use crate::ui_capability_admission::{
    InteractionUiCapabilityAdmissionDescriptorId,
    InteractionUiCapabilityAdmissionRuntimeMappingRequirement,
};
use crate::ui_capability_admission_result::InteractionUiCapabilityAdmissionResultId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InteractionRuntimeCapabilityMappingResultId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionRuntimeCapabilityMappingDecisionStatus {
    Mapped,
    Denied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionRuntimeCapabilityMappingDenialReason {
    None,
    MissingRuntimeCapability,
    LifecycleBlocked,
    TargetUnavailable,
    TargetInvalid,
    PolicyDenied,
    HostBoundaryDenied,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionRuntimeCapabilityMappingMissingRequirement {
    None,
    RuntimeCapability,
    Lifecycle,
    Target,
    Policy,
    HostBoundary,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionRuntimeCapabilityMappingResult {
    pub id: InteractionRuntimeCapabilityMappingResultId,
    pub descriptor_id: InteractionRuntimeCapabilityMappingDescriptorId,
    pub ui_capability_admission_result_id: InteractionUiCapabilityAdmissionResultId,
    pub ui_capability_admission_descriptor_id: InteractionUiCapabilityAdmissionDescriptorId,
    pub effect_request_descriptor_id: InteractionEffectRequestDescriptorId,
    pub source_admitted_action_id: InteractionAdmittedSemanticActionId,
    pub dispatch_record_id: InteractionSemanticActionDispatchRecordId,
    pub dispatch_route_id: InteractionSemanticActionDispatchRouteId,
    pub status: InteractionRuntimeCapabilityMappingDecisionStatus,
    pub denial_reason: InteractionRuntimeCapabilityMappingDenialReason,
    pub missing_requirement: InteractionRuntimeCapabilityMappingMissingRequirement,
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
}

pub fn record_interaction_runtime_capability_mapped_result(
    descriptor: &InteractionRuntimeCapabilityMappingDescriptor,
) -> InteractionRuntimeCapabilityMappingResult {
    build_result(
        descriptor,
        InteractionRuntimeCapabilityMappingDecisionStatus::Mapped,
        InteractionRuntimeCapabilityMappingDenialReason::None,
        InteractionRuntimeCapabilityMappingMissingRequirement::None,
    )
}

pub fn record_interaction_runtime_capability_denied_result(
    descriptor: &InteractionRuntimeCapabilityMappingDescriptor,
    denial_reason: InteractionRuntimeCapabilityMappingDenialReason,
    missing_requirement: InteractionRuntimeCapabilityMappingMissingRequirement,
) -> InteractionRuntimeCapabilityMappingResult {
    build_result(
        descriptor,
        InteractionRuntimeCapabilityMappingDecisionStatus::Denied,
        denial_reason,
        missing_requirement,
    )
}

fn build_result(
    descriptor: &InteractionRuntimeCapabilityMappingDescriptor,
    status: InteractionRuntimeCapabilityMappingDecisionStatus,
    denial_reason: InteractionRuntimeCapabilityMappingDenialReason,
    missing_requirement: InteractionRuntimeCapabilityMappingMissingRequirement,
) -> InteractionRuntimeCapabilityMappingResult {
    InteractionRuntimeCapabilityMappingResult {
        id: InteractionRuntimeCapabilityMappingResultId(descriptor.id.0),
        descriptor_id: descriptor.id,
        ui_capability_admission_result_id: descriptor.ui_capability_admission_result_id,
        ui_capability_admission_descriptor_id: descriptor.ui_capability_admission_descriptor_id,
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
        runtime_mapping_requirement: descriptor.runtime_mapping_requirement,
        runtime_capability_namespace: descriptor.runtime_capability_namespace,
        lifecycle_precondition: descriptor.lifecycle_precondition,
        target_policy: descriptor.target_policy,
        trace_requirement: descriptor.trace_requirement,
        policy_gate_namespace: descriptor.policy_gate_namespace,
        scope: descriptor.scope,
    }
}

impl InteractionRuntimeCapabilityMappingResult {
    pub const fn is_mapped(&self) -> bool {
        matches!(self.status, InteractionRuntimeCapabilityMappingDecisionStatus::Mapped)
    }

    pub const fn is_denied(&self) -> bool {
        matches!(self.status, InteractionRuntimeCapabilityMappingDecisionStatus::Denied)
    }

    pub const fn grants_runtime_capability(&self) -> bool {
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
        InteractionActionAdmissionEffectRelationship, InteractionActionAdmissionPolicyGateNamespace,
        InteractionActionAdmissionTraceRequirement,
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
    use crate::runtime_capability_mapping::describe_interaction_runtime_capability_mapping;
    use crate::ui_capability_admission::describe_interaction_ui_capability_admission;
    use crate::ui_capability_admission_result::record_interaction_ui_capability_admitted_result;

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
            effect_relationship: InteractionActionAdmissionEffectRelationship::MayRequestEffectAfterAdmission,
            policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace::CoreUi,
            effect_eligibility:
                InteractionSemanticActionDispatchEffectEligibility::RequiresFutureEffectBoundary,
        };

        describe_interaction_effect_request(&dispatch_trace)
            .expect("effect candidate trace should produce descriptor")
    }

    fn mapping_descriptor(
        action: InteractionActionName,
        record_id: u64,
    ) -> InteractionRuntimeCapabilityMappingDescriptor {
        let effect_request = effect_request_descriptor(action, record_id);
        let admission = describe_interaction_ui_capability_admission(&effect_request);
        let result = record_interaction_ui_capability_admitted_result(&admission);

        describe_interaction_runtime_capability_mapping(&result)
            .expect("admitted result should produce mapping descriptor")
    }

    #[test]
    fn mapped_result_is_built_from_descriptor() {
        let descriptor = mapping_descriptor(InteractionActionName::PrepareEffect, 91);

        let result = record_interaction_runtime_capability_mapped_result(&descriptor);

        assert_eq!(result.status, InteractionRuntimeCapabilityMappingDecisionStatus::Mapped);
        assert_eq!(result.descriptor_id, descriptor.id);
        assert_eq!(result.id, InteractionRuntimeCapabilityMappingResultId(descriptor.id.0));
    }

    #[test]
    fn denied_result_is_built_from_descriptor() {
        let descriptor = mapping_descriptor(InteractionActionName::PrepareEffect, 92);

        let result = record_interaction_runtime_capability_denied_result(
            &descriptor,
            InteractionRuntimeCapabilityMappingDenialReason::PolicyDenied,
            InteractionRuntimeCapabilityMappingMissingRequirement::Policy,
        );

        assert_eq!(result.status, InteractionRuntimeCapabilityMappingDecisionStatus::Denied);
        assert_eq!(result.descriptor_id, descriptor.id);
        assert_eq!(result.id, InteractionRuntimeCapabilityMappingResultId(descriptor.id.0));
    }

    #[test]
    fn result_preserves_source_and_capability_metadata() {
        let descriptor = mapping_descriptor(InteractionActionName::PrepareEffect, 93);

        let result = record_interaction_runtime_capability_mapped_result(&descriptor);

        assert_eq!(
            result.ui_capability_admission_result_id,
            descriptor.ui_capability_admission_result_id
        );
        assert_eq!(
            result.ui_capability_admission_descriptor_id,
            descriptor.ui_capability_admission_descriptor_id
        );
        assert_eq!(result.effect_request_descriptor_id, descriptor.effect_request_descriptor_id);
        assert_eq!(result.source_admitted_action_id, descriptor.source_admitted_action_id);
        assert_eq!(result.dispatch_record_id, descriptor.dispatch_record_id);
        assert_eq!(result.dispatch_route_id, descriptor.dispatch_route_id);
        assert_eq!(result.requested_effect, descriptor.requested_effect);
        assert_eq!(result.declared_ui_capability, descriptor.declared_ui_capability);
        assert_eq!(
            result.declared_runtime_capability_requirement,
            descriptor.declared_runtime_capability_requirement
        );
        assert_eq!(
            result.runtime_mapping_requirement,
            descriptor.runtime_mapping_requirement
        );
        assert_eq!(
            result.runtime_capability_namespace,
            descriptor.runtime_capability_namespace
        );
        assert_eq!(result.lifecycle_precondition, descriptor.lifecycle_precondition);
        assert_eq!(result.target_policy, descriptor.target_policy);
        assert_eq!(result.trace_requirement, descriptor.trace_requirement);
        assert_eq!(result.policy_gate_namespace, descriptor.policy_gate_namespace);
        assert_eq!(result.scope, descriptor.scope);
    }

    #[test]
    fn mapped_result_has_none_denial_metadata() {
        let descriptor = mapping_descriptor(InteractionActionName::PrepareEffect, 94);

        let result = record_interaction_runtime_capability_mapped_result(&descriptor);

        assert_eq!(result.denial_reason, InteractionRuntimeCapabilityMappingDenialReason::None);
        assert_eq!(
            result.missing_requirement,
            InteractionRuntimeCapabilityMappingMissingRequirement::None
        );
        assert!(result.is_mapped());
        assert!(!result.is_denied());
    }

    #[test]
    fn denied_result_preserves_denial_metadata() {
        let descriptor = mapping_descriptor(InteractionActionName::PrepareEffect, 95);

        let result = record_interaction_runtime_capability_denied_result(
            &descriptor,
            InteractionRuntimeCapabilityMappingDenialReason::HostBoundaryDenied,
            InteractionRuntimeCapabilityMappingMissingRequirement::HostBoundary,
        );

        assert_eq!(
            result.denial_reason,
            InteractionRuntimeCapabilityMappingDenialReason::HostBoundaryDenied
        );
        assert_eq!(
            result.missing_requirement,
            InteractionRuntimeCapabilityMappingMissingRequirement::HostBoundary
        );
        assert!(result.is_denied());
        assert!(!result.is_mapped());
    }

    #[test]
    fn result_is_not_authority() {
        let descriptor = mapping_descriptor(InteractionActionName::PrepareEffect, 96);

        let result = record_interaction_runtime_capability_mapped_result(&descriptor);

        assert!(!result.grants_runtime_capability());
        assert!(!result.is_host_abi_authority());
        assert!(!result.is_vm_authority());
        assert!(!result.is_prepared_effect());
        assert!(!result.is_committed_effect());
        assert!(!result.is_execution_authority());
        assert!(!result.is_runtime_mutation());
    }

    #[test]
    fn deterministic_result_generation() {
        let descriptor = mapping_descriptor(InteractionActionName::PrepareEffect, 97);

        let first = record_interaction_runtime_capability_mapped_result(&descriptor);
        let second = record_interaction_runtime_capability_mapped_result(&descriptor);

        assert_eq!(first, second);
    }
}
