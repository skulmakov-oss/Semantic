//! Semantic UI commit boundary descriptor scaffold.
//!
//! This module records an inert descriptor for the future commit boundary. It
//! does not implement a commit decision, committed effect, Host ABI calls, VM
//! calls, effect execution, audit backend integration, or runtime mutation.

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
use crate::prepared_effect::InteractionPreparedEffectDescriptorId;
use crate::prepared_effect_result::{
    InteractionPreparedEffectDecisionStatus, InteractionPreparedEffectResult,
    InteractionPreparedEffectResultId,
};
use crate::runtime_capability_mapping::{
    InteractionRuntimeCapabilityMappingDescriptorId, InteractionRuntimeCapabilityNamespace,
};
use crate::runtime_capability_mapping_result::InteractionRuntimeCapabilityMappingResultId;
use crate::ui_capability_admission::InteractionUiCapabilityAdmissionDescriptorId;
use crate::ui_capability_admission_result::InteractionUiCapabilityAdmissionResultId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InteractionCommitBoundaryDescriptorId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionCommitAuditRequirement {
    Required,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionCommitBoundaryFutureCommittedEffectShape {
    CommitOrDenyWithReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionCommitBoundaryDescriptor {
    pub id: InteractionCommitBoundaryDescriptorId,
    pub prepared_effect_result_id: InteractionPreparedEffectResultId,
    pub prepared_effect_descriptor_id: InteractionPreparedEffectDescriptorId,
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
    pub audit_requirement: InteractionCommitAuditRequirement,
    pub future_committed_effect_shape: InteractionCommitBoundaryFutureCommittedEffectShape,
}

pub fn describe_interaction_commit_boundary(
    prepared_result: &InteractionPreparedEffectResult,
) -> Option<InteractionCommitBoundaryDescriptor> {
    if !matches!(
        prepared_result.status,
        InteractionPreparedEffectDecisionStatus::Prepared
    ) {
        return None;
    }

    Some(InteractionCommitBoundaryDescriptor {
        id: InteractionCommitBoundaryDescriptorId(prepared_result.id.0),
        prepared_effect_result_id: prepared_result.id,
        prepared_effect_descriptor_id: prepared_result.descriptor_id,
        runtime_capability_mapping_result_id: prepared_result.runtime_capability_mapping_result_id,
        runtime_capability_mapping_descriptor_id: prepared_result
            .runtime_capability_mapping_descriptor_id,
        ui_capability_admission_result_id: prepared_result.ui_capability_admission_result_id,
        ui_capability_admission_descriptor_id: prepared_result
            .ui_capability_admission_descriptor_id,
        effect_request_descriptor_id: prepared_result.effect_request_descriptor_id,
        source_admitted_action_id: prepared_result.source_admitted_action_id,
        dispatch_record_id: prepared_result.dispatch_record_id,
        dispatch_route_id: prepared_result.dispatch_route_id,
        requested_effect: prepared_result.requested_effect,
        declared_ui_capability: prepared_result.declared_ui_capability,
        declared_runtime_capability_requirement: prepared_result
            .declared_runtime_capability_requirement,
        runtime_capability_namespace: prepared_result.runtime_capability_namespace,
        lifecycle_precondition: prepared_result.lifecycle_precondition,
        target_policy: prepared_result.target_policy,
        trace_requirement: prepared_result.trace_requirement,
        policy_gate_namespace: prepared_result.policy_gate_namespace,
        scope: prepared_result.scope,
        audit_requirement: InteractionCommitAuditRequirement::Required,
        future_committed_effect_shape:
            InteractionCommitBoundaryFutureCommittedEffectShape::CommitOrDenyWithReason,
    })
}

impl InteractionCommitBoundaryDescriptor {
    pub const fn is_commit_decision(&self) -> bool {
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

    pub const fn is_audit_backend(&self) -> bool {
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
    use crate::prepared_effect_result::{
        record_interaction_prepared_effect_denied_result,
        record_interaction_prepared_effect_result, InteractionPreparedEffectDenialReason,
        InteractionPreparedEffectMissingRequirement,
    };
    use crate::runtime_capability_mapping::describe_interaction_runtime_capability_mapping;
    use crate::runtime_capability_mapping_result::record_interaction_runtime_capability_mapped_result;
    use crate::ui_capability_admission::describe_interaction_ui_capability_admission;
    use crate::ui_capability_admission_result::record_interaction_ui_capability_admitted_result;

    fn effect_request_descriptor() -> crate::effect_request::InteractionEffectRequestDescriptor {
        let dispatch_trace = InteractionSemanticActionDispatchTraceReport {
            record_id: InteractionSemanticActionDispatchRecordId(121),
            route_id: InteractionSemanticActionDispatchRouteId(121),
            admitted_action_id: InteractionAdmittedSemanticActionId(121),
            action: InteractionActionName::PrepareEffect,
            source_intent: InteractionIntentKind::Submit,
            binding_id: InteractionActionBindingId(121),
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

    fn prepared_result() -> crate::prepared_effect_result::InteractionPreparedEffectResult {
        let effect_request = effect_request_descriptor();
        let admission = describe_interaction_ui_capability_admission(&effect_request);
        let mapping_descriptor = describe_interaction_runtime_capability_mapping(
            &record_interaction_ui_capability_admitted_result(&admission),
        )
        .expect("admitted result should produce mapping descriptor");
        let mapping_result =
            record_interaction_runtime_capability_mapped_result(&mapping_descriptor);
        let prepared_descriptor = describe_interaction_prepared_effect(&mapping_result)
            .expect("mapped result should produce prepared effect descriptor");

        record_interaction_prepared_effect_result(&prepared_descriptor)
    }

    fn denied_prepared_result() -> crate::prepared_effect_result::InteractionPreparedEffectResult {
        let effect_request = effect_request_descriptor();
        let admission = describe_interaction_ui_capability_admission(&effect_request);
        let mapping_descriptor = describe_interaction_runtime_capability_mapping(
            &record_interaction_ui_capability_admitted_result(&admission),
        )
        .expect("admitted result should produce mapping descriptor");
        let mapping_result =
            record_interaction_runtime_capability_mapped_result(&mapping_descriptor);
        let prepared_descriptor = describe_interaction_prepared_effect(&mapping_result)
            .expect("mapped result should produce prepared effect descriptor");

        record_interaction_prepared_effect_denied_result(
            &prepared_descriptor,
            InteractionPreparedEffectDenialReason::PolicyDenied,
            InteractionPreparedEffectMissingRequirement::Policy,
        )
    }

    #[test]
    fn prepared_effect_result_creates_commit_boundary_descriptor() {
        let result = prepared_result();

        let descriptor = describe_interaction_commit_boundary(&result)
            .expect("prepared result should produce commit boundary descriptor");

        assert_eq!(descriptor.id, InteractionCommitBoundaryDescriptorId(121));
        assert_eq!(descriptor.prepared_effect_result_id, result.id);
    }

    #[test]
    fn denied_prepared_effect_result_returns_none() {
        let result = denied_prepared_result();

        let descriptor = describe_interaction_commit_boundary(&result);

        assert!(descriptor.is_none());
    }

    #[test]
    fn descriptor_preserves_source_and_capability_metadata() {
        let result = prepared_result();

        let descriptor = describe_interaction_commit_boundary(&result)
            .expect("prepared result should produce commit boundary descriptor");

        assert_eq!(descriptor.prepared_effect_result_id, result.id);
        assert_eq!(
            descriptor.prepared_effect_descriptor_id,
            result.descriptor_id
        );
        assert_eq!(
            descriptor.runtime_capability_mapping_result_id,
            result.runtime_capability_mapping_result_id
        );
        assert_eq!(
            descriptor.runtime_capability_mapping_descriptor_id,
            result.runtime_capability_mapping_descriptor_id
        );
        assert_eq!(
            descriptor.ui_capability_admission_result_id,
            result.ui_capability_admission_result_id
        );
        assert_eq!(
            descriptor.ui_capability_admission_descriptor_id,
            result.ui_capability_admission_descriptor_id
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
            descriptor.runtime_capability_namespace,
            result.runtime_capability_namespace
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
    }

    #[test]
    fn descriptor_records_future_committed_effect_shape() {
        let result = prepared_result();

        let descriptor = describe_interaction_commit_boundary(&result)
            .expect("prepared result should produce commit boundary descriptor");

        assert_eq!(
            descriptor.audit_requirement,
            InteractionCommitAuditRequirement::Required
        );
        assert_eq!(
            descriptor.future_committed_effect_shape,
            InteractionCommitBoundaryFutureCommittedEffectShape::CommitOrDenyWithReason
        );
    }

    #[test]
    fn descriptor_is_not_authority() {
        let result = prepared_result();

        let descriptor = describe_interaction_commit_boundary(&result)
            .expect("prepared result should produce commit boundary descriptor");

        assert!(!descriptor.is_commit_decision());
        assert!(!descriptor.is_committed_effect());
        assert!(!descriptor.is_host_abi_authority());
        assert!(!descriptor.is_vm_authority());
        assert!(!descriptor.is_execution_authority());
        assert!(!descriptor.is_runtime_mutation());
        assert!(!descriptor.is_audit_backend());
    }

    #[test]
    fn deterministic_descriptor_generation() {
        let result = prepared_result();

        let first = describe_interaction_commit_boundary(&result);
        let second = describe_interaction_commit_boundary(&result);

        assert_eq!(first, second);
    }
}
