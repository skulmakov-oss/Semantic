//! Semantic UI committed effect descriptor scaffold.
//!
//! This module records an inert committed-effect descriptor derived from a
//! committed commit-boundary result. It does not implement Host ABI calls, VM
//! calls, effect execution, runtime mutation, audit backend behavior, or any
//! host runtime path.

use crate::action_admission::{
    InteractionActionAdmissionPolicyGateNamespace, InteractionActionAdmissionTraceRequirement,
};
use crate::action_dispatch_record::InteractionSemanticActionDispatchRecordId;
use crate::action_dispatch_route::InteractionSemanticActionDispatchRouteId;
use crate::admitted_action::InteractionAdmittedSemanticActionId;
use crate::commit_boundary::{
    InteractionCommitAuditRequirement, InteractionCommitBoundaryDescriptorId,
};
use crate::commit_boundary_result::{
    InteractionCommitBoundaryDecisionStatus, InteractionCommitBoundaryResult,
    InteractionCommitBoundaryResultId,
};
use crate::effect_request::{
    InteractionEffectRequestDescriptorId, InteractionEffectRequestKind,
    InteractionEffectRequestLifecyclePrecondition, InteractionEffectRequestRuntimeCapability,
    InteractionEffectRequestScope, InteractionEffectRequestTargetPolicy,
    InteractionEffectRequestUiCapability,
};
use crate::prepared_effect::InteractionPreparedEffectDescriptorId;
use crate::prepared_effect_result::InteractionPreparedEffectResultId;
use crate::runtime_capability_mapping::{
    InteractionRuntimeCapabilityMappingDescriptorId, InteractionRuntimeCapabilityNamespace,
};
use crate::runtime_capability_mapping_result::InteractionRuntimeCapabilityMappingResultId;
use crate::ui_capability_admission::InteractionUiCapabilityAdmissionDescriptorId;
use crate::ui_capability_admission_result::InteractionUiCapabilityAdmissionResultId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InteractionCommittedEffectDescriptorId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionCommittedEffectAuditVisibility {
    Required,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionCommittedEffectRuntimeMutationRequirement {
    SeparateBoundaryRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionCommittedEffectHostPathRequirement {
    SeparateBoundaryRequired,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionCommittedEffectDescriptor {
    pub id: InteractionCommittedEffectDescriptorId,
    pub commit_boundary_result_id: InteractionCommitBoundaryResultId,
    pub commit_boundary_descriptor_id: InteractionCommitBoundaryDescriptorId,
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
    pub audit_visibility: InteractionCommittedEffectAuditVisibility,
    pub runtime_mutation_requirement: InteractionCommittedEffectRuntimeMutationRequirement,
    pub host_path_requirement: InteractionCommittedEffectHostPathRequirement,
}

pub fn describe_interaction_committed_effect(
    commit_result: &InteractionCommitBoundaryResult,
) -> Option<InteractionCommittedEffectDescriptor> {
    if !matches!(
        commit_result.status,
        InteractionCommitBoundaryDecisionStatus::Committed
    ) {
        return None;
    }

    Some(InteractionCommittedEffectDescriptor {
        id: InteractionCommittedEffectDescriptorId(commit_result.id.0),
        commit_boundary_result_id: commit_result.id,
        commit_boundary_descriptor_id: commit_result.descriptor_id,
        prepared_effect_result_id: commit_result.prepared_effect_result_id,
        prepared_effect_descriptor_id: commit_result.prepared_effect_descriptor_id,
        runtime_capability_mapping_result_id: commit_result.runtime_capability_mapping_result_id,
        runtime_capability_mapping_descriptor_id: commit_result
            .runtime_capability_mapping_descriptor_id,
        ui_capability_admission_result_id: commit_result.ui_capability_admission_result_id,
        ui_capability_admission_descriptor_id: commit_result.ui_capability_admission_descriptor_id,
        effect_request_descriptor_id: commit_result.effect_request_descriptor_id,
        source_admitted_action_id: commit_result.source_admitted_action_id,
        dispatch_record_id: commit_result.dispatch_record_id,
        dispatch_route_id: commit_result.dispatch_route_id,
        requested_effect: commit_result.requested_effect,
        declared_ui_capability: commit_result.declared_ui_capability,
        declared_runtime_capability_requirement: commit_result
            .declared_runtime_capability_requirement,
        runtime_capability_namespace: commit_result.runtime_capability_namespace,
        lifecycle_precondition: commit_result.lifecycle_precondition,
        target_policy: commit_result.target_policy,
        trace_requirement: commit_result.trace_requirement,
        policy_gate_namespace: commit_result.policy_gate_namespace,
        scope: commit_result.scope,
        audit_requirement: commit_result.audit_requirement,
        audit_visibility: InteractionCommittedEffectAuditVisibility::Required,
        runtime_mutation_requirement:
            InteractionCommittedEffectRuntimeMutationRequirement::SeparateBoundaryRequired,
        host_path_requirement: InteractionCommittedEffectHostPathRequirement::SeparateBoundaryRequired,
    })
}

impl InteractionCommittedEffectDescriptor {
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

    pub const fn is_host_runtime_path(&self) -> bool {
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
    use crate::commit_boundary::describe_interaction_commit_boundary;
    use crate::commit_boundary_result::{
        record_interaction_commit_boundary_denied_result,
        record_interaction_commit_boundary_committed_result,
        InteractionCommitBoundaryDenialReason, InteractionCommitBoundaryMissingRequirement,
    };
    use crate::prepared_effect::describe_interaction_prepared_effect;
    use crate::prepared_effect_result::record_interaction_prepared_effect_result;
    use crate::runtime_capability_mapping::describe_interaction_runtime_capability_mapping;
    use crate::runtime_capability_mapping_result::record_interaction_runtime_capability_mapped_result;
    use crate::ui_capability_admission::describe_interaction_ui_capability_admission;
    use crate::ui_capability_admission_result::record_interaction_ui_capability_admitted_result;

    fn effect_request_descriptor() -> crate::effect_request::InteractionEffectRequestDescriptor {
        let dispatch_trace = InteractionSemanticActionDispatchTraceReport {
            record_id: InteractionSemanticActionDispatchRecordId(141),
            route_id: InteractionSemanticActionDispatchRouteId(141),
            admitted_action_id: InteractionAdmittedSemanticActionId(141),
            action: InteractionActionName::PrepareEffect,
            source_intent: InteractionIntentKind::Submit,
            binding_id: InteractionActionBindingId(141),
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

    fn commit_boundary_descriptor() -> crate::commit_boundary::InteractionCommitBoundaryDescriptor {
        let effect_request = effect_request_descriptor();
        let admission = describe_interaction_ui_capability_admission(&effect_request);
        let ui_result = record_interaction_ui_capability_admitted_result(&admission);
        let mapping_descriptor = describe_interaction_runtime_capability_mapping(&ui_result)
            .expect("admitted result should produce mapping descriptor");
        let mapping_result = record_interaction_runtime_capability_mapped_result(&mapping_descriptor);
        let prepared_descriptor = describe_interaction_prepared_effect(&mapping_result)
            .expect("mapped result should produce prepared effect descriptor");
        let prepared_result = record_interaction_prepared_effect_result(&prepared_descriptor);
        describe_interaction_commit_boundary(&prepared_result)
            .expect("prepared result should produce commit boundary descriptor")
    }

    fn committed_effect_result() -> crate::commit_boundary_result::InteractionCommitBoundaryResult {
        record_interaction_commit_boundary_committed_result(&commit_boundary_descriptor())
    }

    fn denied_commit_boundary_result() -> crate::commit_boundary_result::InteractionCommitBoundaryResult {
        record_interaction_commit_boundary_denied_result(
            &commit_boundary_descriptor(),
            InteractionCommitBoundaryDenialReason::PolicyDenied,
            InteractionCommitBoundaryMissingRequirement::Policy,
        )
    }

    fn committed_effect_descriptor() -> InteractionCommittedEffectDescriptor {
        describe_interaction_committed_effect(&committed_effect_result())
            .expect("committed result should produce committed effect descriptor")
    }

    #[test]
    fn committed_commit_boundary_result_creates_committed_effect_descriptor() {
        let descriptor = committed_effect_descriptor();

        assert_eq!(
            descriptor.id,
            InteractionCommittedEffectDescriptorId(descriptor.commit_boundary_result_id.0)
        );
    }

    #[test]
    fn denied_commit_boundary_result_returns_none() {
        assert!(describe_interaction_committed_effect(&denied_commit_boundary_result()).is_none());
    }

    #[test]
    fn descriptor_preserves_source_capability_audit_metadata() {
        let descriptor = committed_effect_descriptor();

        assert_eq!(descriptor.audit_requirement, InteractionCommitAuditRequirement::Required);
        assert_eq!(
            descriptor.runtime_mutation_requirement,
            InteractionCommittedEffectRuntimeMutationRequirement::SeparateBoundaryRequired
        );
        assert_eq!(
            descriptor.host_path_requirement,
            InteractionCommittedEffectHostPathRequirement::SeparateBoundaryRequired
        );
        assert_eq!(descriptor.audit_visibility, InteractionCommittedEffectAuditVisibility::Required);
    }

    #[test]
    fn descriptor_records_separate_runtime_mutation_and_host_path_requirements() {
        let descriptor = committed_effect_descriptor();

        assert_eq!(
            descriptor.runtime_mutation_requirement,
            InteractionCommittedEffectRuntimeMutationRequirement::SeparateBoundaryRequired
        );
        assert_eq!(
            descriptor.host_path_requirement,
            InteractionCommittedEffectHostPathRequirement::SeparateBoundaryRequired
        );
    }

    #[test]
    fn descriptor_is_not_authority() {
        let descriptor = committed_effect_descriptor();

        assert!(!descriptor.is_host_abi_authority());
        assert!(!descriptor.is_vm_authority());
        assert!(!descriptor.is_execution_authority());
        assert!(!descriptor.is_runtime_mutation());
        assert!(!descriptor.is_audit_backend());
        assert!(!descriptor.is_host_runtime_path());
    }

    #[test]
    fn generation_is_deterministic() {
        let left = committed_effect_descriptor();
        let right = committed_effect_descriptor();

        assert_eq!(left, right);
    }
}
