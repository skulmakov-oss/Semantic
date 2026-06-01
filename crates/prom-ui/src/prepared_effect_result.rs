//! Semantic UI prepared effect result scaffold.
//!
//! This module records an inert decision outcome for a prepared effect
//! descriptor. It does not implement commit, committed effects, Host ABI
//! calls, VM calls, effect execution, or runtime mutation.

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
use crate::prepared_effect::{
    InteractionPreparedEffectCommitRequirement, InteractionPreparedEffectDescriptor,
    InteractionPreparedEffectDescriptorId,
};
use crate::runtime_capability_mapping::{
    InteractionRuntimeCapabilityMappingDescriptorId, InteractionRuntimeCapabilityNamespace,
};
use crate::runtime_capability_mapping_result::InteractionRuntimeCapabilityMappingResultId;
use crate::ui_capability_admission::InteractionUiCapabilityAdmissionDescriptorId;
use crate::ui_capability_admission_result::InteractionUiCapabilityAdmissionResultId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InteractionPreparedEffectResultId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionPreparedEffectDecisionStatus {
    Prepared,
    Denied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionPreparedEffectDenialReason {
    None,
    MissingRuntimeMapping,
    LifecycleBlocked,
    TargetUnavailable,
    TargetInvalid,
    PolicyDenied,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionPreparedEffectMissingRequirement {
    None,
    RuntimeMapping,
    Lifecycle,
    Target,
    Policy,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionPreparedEffectResult {
    pub id: InteractionPreparedEffectResultId,
    pub descriptor_id: InteractionPreparedEffectDescriptorId,
    pub runtime_capability_mapping_result_id: InteractionRuntimeCapabilityMappingResultId,
    pub runtime_capability_mapping_descriptor_id: InteractionRuntimeCapabilityMappingDescriptorId,
    pub ui_capability_admission_result_id: InteractionUiCapabilityAdmissionResultId,
    pub ui_capability_admission_descriptor_id: InteractionUiCapabilityAdmissionDescriptorId,
    pub effect_request_descriptor_id: InteractionEffectRequestDescriptorId,
    pub source_admitted_action_id: InteractionAdmittedSemanticActionId,
    pub dispatch_record_id: InteractionSemanticActionDispatchRecordId,
    pub dispatch_route_id: InteractionSemanticActionDispatchRouteId,
    pub status: InteractionPreparedEffectDecisionStatus,
    pub denial_reason: InteractionPreparedEffectDenialReason,
    pub missing_requirement: InteractionPreparedEffectMissingRequirement,
    pub requested_effect: InteractionEffectRequestKind,
    pub declared_ui_capability: InteractionEffectRequestUiCapability,
    pub declared_runtime_capability_requirement: InteractionEffectRequestRuntimeCapability,
    pub runtime_capability_namespace: InteractionRuntimeCapabilityNamespace,
    pub lifecycle_precondition: InteractionEffectRequestLifecyclePrecondition,
    pub target_policy: InteractionEffectRequestTargetPolicy,
    pub trace_requirement: InteractionActionAdmissionTraceRequirement,
    pub policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace,
    pub scope: InteractionEffectRequestScope,
    pub future_commit_requirement: InteractionPreparedEffectCommitRequirement,
}

pub fn record_interaction_prepared_effect_result(
    descriptor: &InteractionPreparedEffectDescriptor,
) -> InteractionPreparedEffectResult {
    build_result(
        descriptor,
        InteractionPreparedEffectDecisionStatus::Prepared,
        InteractionPreparedEffectDenialReason::None,
        InteractionPreparedEffectMissingRequirement::None,
    )
}

pub fn record_interaction_prepared_effect_denied_result(
    descriptor: &InteractionPreparedEffectDescriptor,
    denial_reason: InteractionPreparedEffectDenialReason,
    missing_requirement: InteractionPreparedEffectMissingRequirement,
) -> InteractionPreparedEffectResult {
    build_result(
        descriptor,
        InteractionPreparedEffectDecisionStatus::Denied,
        denial_reason,
        missing_requirement,
    )
}

fn build_result(
    descriptor: &InteractionPreparedEffectDescriptor,
    status: InteractionPreparedEffectDecisionStatus,
    denial_reason: InteractionPreparedEffectDenialReason,
    missing_requirement: InteractionPreparedEffectMissingRequirement,
) -> InteractionPreparedEffectResult {
    InteractionPreparedEffectResult {
        id: InteractionPreparedEffectResultId(descriptor.id.0),
        descriptor_id: descriptor.id,
        runtime_capability_mapping_result_id: descriptor.runtime_capability_mapping_result_id,
        runtime_capability_mapping_descriptor_id: descriptor
            .runtime_capability_mapping_descriptor_id,
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
        declared_runtime_capability_requirement: descriptor.declared_runtime_capability_requirement,
        runtime_capability_namespace: descriptor.runtime_capability_namespace,
        lifecycle_precondition: descriptor.lifecycle_precondition,
        target_policy: descriptor.target_policy,
        trace_requirement: descriptor.trace_requirement,
        policy_gate_namespace: descriptor.policy_gate_namespace,
        scope: descriptor.scope,
        future_commit_requirement: descriptor.future_commit_requirement,
    }
}

impl InteractionPreparedEffectResult {
    pub const fn is_prepared(&self) -> bool {
        matches!(
            self.status,
            InteractionPreparedEffectDecisionStatus::Prepared
        )
    }

    pub const fn is_denied(&self) -> bool {
        matches!(self.status, InteractionPreparedEffectDecisionStatus::Denied)
    }

    pub const fn is_commit_boundary(&self) -> bool {
        false
    }

    pub const fn is_committed_effect(&self) -> bool {
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
    use crate::prepared_effect::describe_interaction_prepared_effect;
    use crate::runtime_capability_mapping::describe_interaction_runtime_capability_mapping;
    use crate::runtime_capability_mapping_result::record_interaction_runtime_capability_mapped_result;
    use crate::ui_capability_admission::describe_interaction_ui_capability_admission;

    fn effect_request_descriptor() -> crate::effect_request::InteractionEffectRequestDescriptor {
        let dispatch_trace = InteractionSemanticActionDispatchTraceReport {
            record_id: InteractionSemanticActionDispatchRecordId(111),
            route_id: InteractionSemanticActionDispatchRouteId(111),
            admitted_action_id: InteractionAdmittedSemanticActionId(111),
            action: InteractionActionName::PrepareEffect,
            source_intent: InteractionIntentKind::Submit,
            binding_id: InteractionActionBindingId(111),
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

    fn prepared_effect_descriptor() -> InteractionPreparedEffectDescriptor {
        let effect_request = effect_request_descriptor();
        let admission = describe_interaction_ui_capability_admission(&effect_request);
        let mapping_descriptor = describe_interaction_runtime_capability_mapping(
            &crate::ui_capability_admission_result::record_interaction_ui_capability_admitted_result(&admission),
        )
        .expect("admitted result should produce mapping descriptor");
        let mapping_result =
            record_interaction_runtime_capability_mapped_result(&mapping_descriptor);

        describe_interaction_prepared_effect(&mapping_result)
            .expect("mapped result should produce prepared effect descriptor")
    }

    #[test]
    fn prepared_result_is_built_from_descriptor() {
        let descriptor = prepared_effect_descriptor();

        let result = record_interaction_prepared_effect_result(&descriptor);

        assert_eq!(
            result.status,
            InteractionPreparedEffectDecisionStatus::Prepared
        );
        assert_eq!(result.descriptor_id, descriptor.id);
        assert_eq!(
            result.id,
            InteractionPreparedEffectResultId(descriptor.id.0)
        );
    }

    #[test]
    fn denied_result_is_built_from_descriptor() {
        let descriptor = prepared_effect_descriptor();

        let result = record_interaction_prepared_effect_denied_result(
            &descriptor,
            InteractionPreparedEffectDenialReason::PolicyDenied,
            InteractionPreparedEffectMissingRequirement::Policy,
        );

        assert_eq!(
            result.status,
            InteractionPreparedEffectDecisionStatus::Denied
        );
        assert_eq!(result.descriptor_id, descriptor.id);
        assert_eq!(
            result.id,
            InteractionPreparedEffectResultId(descriptor.id.0)
        );
    }

    #[test]
    fn result_preserves_source_and_capability_metadata() {
        let descriptor = prepared_effect_descriptor();

        let result = record_interaction_prepared_effect_result(&descriptor);

        assert_eq!(
            result.runtime_capability_mapping_result_id,
            descriptor.runtime_capability_mapping_result_id
        );
        assert_eq!(
            result.runtime_capability_mapping_descriptor_id,
            descriptor.runtime_capability_mapping_descriptor_id
        );
        assert_eq!(
            result.ui_capability_admission_result_id,
            descriptor.ui_capability_admission_result_id
        );
        assert_eq!(
            result.ui_capability_admission_descriptor_id,
            descriptor.ui_capability_admission_descriptor_id
        );
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
            result.runtime_capability_namespace,
            descriptor.runtime_capability_namespace
        );
        assert_eq!(
            result.lifecycle_precondition,
            descriptor.lifecycle_precondition
        );
        assert_eq!(result.target_policy, descriptor.target_policy);
        assert_eq!(result.trace_requirement, descriptor.trace_requirement);
        assert_eq!(
            result.policy_gate_namespace,
            descriptor.policy_gate_namespace
        );
        assert_eq!(result.scope, descriptor.scope);
    }

    #[test]
    fn prepared_result_has_none_denial_metadata() {
        let descriptor = prepared_effect_descriptor();

        let result = record_interaction_prepared_effect_result(&descriptor);

        assert_eq!(
            result.denial_reason,
            InteractionPreparedEffectDenialReason::None
        );
        assert_eq!(
            result.missing_requirement,
            InteractionPreparedEffectMissingRequirement::None
        );
        assert!(result.is_prepared());
        assert!(!result.is_denied());
    }

    #[test]
    fn denied_result_preserves_denial_metadata() {
        let descriptor = prepared_effect_descriptor();

        let result = record_interaction_prepared_effect_denied_result(
            &descriptor,
            InteractionPreparedEffectDenialReason::MissingRuntimeMapping,
            InteractionPreparedEffectMissingRequirement::RuntimeMapping,
        );

        assert_eq!(
            result.denial_reason,
            InteractionPreparedEffectDenialReason::MissingRuntimeMapping
        );
        assert_eq!(
            result.missing_requirement,
            InteractionPreparedEffectMissingRequirement::RuntimeMapping
        );
        assert!(result.is_denied());
        assert!(!result.is_prepared());
    }

    #[test]
    fn result_is_not_authority() {
        let descriptor = prepared_effect_descriptor();

        let result = record_interaction_prepared_effect_result(&descriptor);

        assert!(!result.is_commit_boundary());
        assert!(!result.is_committed_effect());
        assert!(!result.is_host_abi_authority());
        assert!(!result.is_vm_authority());
        assert!(!result.is_execution_authority());
        assert!(!result.is_runtime_mutation());
    }

    #[test]
    fn generation_is_deterministic() {
        let descriptor = prepared_effect_descriptor();

        let first = record_interaction_prepared_effect_result(&descriptor);
        let second = record_interaction_prepared_effect_result(&descriptor);

        assert_eq!(first, second);
    }
}
