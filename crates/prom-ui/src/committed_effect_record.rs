//! Semantic UI committed effect record scaffold.
//!
//! This module records an inert UI-side committed-effect record derived from a
//! committed effect descriptor. It does not implement Host ABI calls, VM
//! calls, effect execution, runtime mutation, an audit backend, or any host
//! runtime path.

use crate::action_admission::{
    InteractionActionAdmissionPolicyGateNamespace, InteractionActionAdmissionTraceRequirement,
};
use crate::action_dispatch_record::InteractionSemanticActionDispatchRecordId;
use crate::action_dispatch_route::InteractionSemanticActionDispatchRouteId;
use crate::admitted_action::InteractionAdmittedSemanticActionId;
use crate::commit_boundary::InteractionCommitAuditRequirement;
use crate::commit_boundary_result::InteractionCommitBoundaryResultId;
use crate::committed_effect::{
    InteractionCommittedEffectAuditVisibility, InteractionCommittedEffectDescriptor,
    InteractionCommittedEffectDescriptorId,
    InteractionCommittedEffectHostPathRequirement,
    InteractionCommittedEffectRuntimeMutationRequirement,
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
pub struct InteractionCommittedEffectRecordId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionCommittedEffectRecordStatus {
    Recorded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionCommittedEffectRuntimeMutationStatus {
    NotPerformed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionCommittedEffectHostPathStatus {
    NotEntered,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionCommittedEffectRecord {
    pub id: InteractionCommittedEffectRecordId,
    pub descriptor_id: InteractionCommittedEffectDescriptorId,
    pub commit_boundary_result_id: InteractionCommitBoundaryResultId,
    pub commit_boundary_descriptor_id: crate::commit_boundary::InteractionCommitBoundaryDescriptorId,
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
    pub record_status: InteractionCommittedEffectRecordStatus,
    pub runtime_mutation_status: InteractionCommittedEffectRuntimeMutationStatus,
    pub host_path_status: InteractionCommittedEffectHostPathStatus,
}

pub fn record_interaction_committed_effect(
    descriptor: &InteractionCommittedEffectDescriptor,
) -> InteractionCommittedEffectRecord {
    InteractionCommittedEffectRecord {
        id: InteractionCommittedEffectRecordId(descriptor.id.0),
        descriptor_id: descriptor.id,
        commit_boundary_result_id: descriptor.commit_boundary_result_id,
        commit_boundary_descriptor_id: descriptor.commit_boundary_descriptor_id,
        prepared_effect_result_id: descriptor.prepared_effect_result_id,
        prepared_effect_descriptor_id: descriptor.prepared_effect_descriptor_id,
        runtime_capability_mapping_result_id: descriptor.runtime_capability_mapping_result_id,
        runtime_capability_mapping_descriptor_id: descriptor
            .runtime_capability_mapping_descriptor_id,
        ui_capability_admission_result_id: descriptor.ui_capability_admission_result_id,
        ui_capability_admission_descriptor_id: descriptor.ui_capability_admission_descriptor_id,
        effect_request_descriptor_id: descriptor.effect_request_descriptor_id,
        source_admitted_action_id: descriptor.source_admitted_action_id,
        dispatch_record_id: descriptor.dispatch_record_id,
        dispatch_route_id: descriptor.dispatch_route_id,
        requested_effect: descriptor.requested_effect,
        declared_ui_capability: descriptor.declared_ui_capability,
        declared_runtime_capability_requirement: descriptor
            .declared_runtime_capability_requirement,
        runtime_capability_namespace: descriptor.runtime_capability_namespace,
        lifecycle_precondition: descriptor.lifecycle_precondition,
        target_policy: descriptor.target_policy,
        trace_requirement: descriptor.trace_requirement,
        policy_gate_namespace: descriptor.policy_gate_namespace,
        scope: descriptor.scope,
        audit_requirement: descriptor.audit_requirement,
        audit_visibility: descriptor.audit_visibility,
        runtime_mutation_requirement: descriptor.runtime_mutation_requirement,
        host_path_requirement: descriptor.host_path_requirement,
        record_status: InteractionCommittedEffectRecordStatus::Recorded,
        runtime_mutation_status: InteractionCommittedEffectRuntimeMutationStatus::NotPerformed,
        host_path_status: InteractionCommittedEffectHostPathStatus::NotEntered,
    }
}

impl InteractionCommittedEffectRecord {
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
    use crate::commit_boundary::describe_interaction_commit_boundary;
    use crate::commit_boundary_result::record_interaction_commit_boundary_committed_result;
    use crate::committed_effect::describe_interaction_committed_effect;
    use crate::effect_request::describe_interaction_effect_request;
    use crate::interaction::InteractionIntentKind;
    use crate::prepared_effect::describe_interaction_prepared_effect;
    use crate::prepared_effect_result::record_interaction_prepared_effect_result;
    use crate::runtime_capability_mapping::describe_interaction_runtime_capability_mapping;
    use crate::runtime_capability_mapping_result::record_interaction_runtime_capability_mapped_result;
    use crate::ui_capability_admission::describe_interaction_ui_capability_admission;
    use crate::ui_capability_admission_result::record_interaction_ui_capability_admitted_result;

    fn effect_request_descriptor() -> crate::effect_request::InteractionEffectRequestDescriptor {
        let dispatch_trace = InteractionSemanticActionDispatchTraceReport {
            record_id: InteractionSemanticActionDispatchRecordId(151),
            route_id: InteractionSemanticActionDispatchRouteId(151),
            admitted_action_id: InteractionAdmittedSemanticActionId(151),
            action: InteractionActionName::PrepareEffect,
            source_intent: InteractionIntentKind::Submit,
            binding_id: InteractionActionBindingId(151),
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

    fn committed_effect_descriptor() -> InteractionCommittedEffectDescriptor {
        let effect_request = effect_request_descriptor();
        let admission = describe_interaction_ui_capability_admission(&effect_request);
        let ui_result = record_interaction_ui_capability_admitted_result(&admission);
        let mapping_descriptor = describe_interaction_runtime_capability_mapping(&ui_result)
            .expect("admitted result should produce mapping descriptor");
        let mapping_result = record_interaction_runtime_capability_mapped_result(&mapping_descriptor);
        let prepared_descriptor = describe_interaction_prepared_effect(&mapping_result)
            .expect("mapped result should produce prepared effect descriptor");
        let prepared_result = record_interaction_prepared_effect_result(&prepared_descriptor);
        let commit_descriptor = describe_interaction_commit_boundary(&prepared_result)
            .expect("prepared result should produce commit boundary descriptor");
        let commit_result = record_interaction_commit_boundary_committed_result(&commit_descriptor);

        describe_interaction_committed_effect(&commit_result)
            .expect("committed result should produce committed effect descriptor")
    }

    #[test]
    fn committed_effect_descriptor_creates_committed_effect_record() {
        let descriptor = committed_effect_descriptor();
        let record = record_interaction_committed_effect(&descriptor);

        assert_eq!(record.id, InteractionCommittedEffectRecordId(descriptor.id.0));
        assert_eq!(record.descriptor_id, descriptor.id);
    }

    #[test]
    fn record_preserves_source_capability_audit_metadata() {
        let descriptor = committed_effect_descriptor();
        let record = record_interaction_committed_effect(&descriptor);

        assert_eq!(record.commit_boundary_result_id, descriptor.commit_boundary_result_id);
        assert_eq!(record.commit_boundary_descriptor_id, descriptor.commit_boundary_descriptor_id);
        assert_eq!(record.prepared_effect_result_id, descriptor.prepared_effect_result_id);
        assert_eq!(record.prepared_effect_descriptor_id, descriptor.prepared_effect_descriptor_id);
        assert_eq!(
            record.runtime_capability_mapping_result_id,
            descriptor.runtime_capability_mapping_result_id
        );
        assert_eq!(
            record.runtime_capability_mapping_descriptor_id,
            descriptor.runtime_capability_mapping_descriptor_id
        );
        assert_eq!(record.ui_capability_admission_result_id, descriptor.ui_capability_admission_result_id);
        assert_eq!(
            record.ui_capability_admission_descriptor_id,
            descriptor.ui_capability_admission_descriptor_id
        );
        assert_eq!(record.effect_request_descriptor_id, descriptor.effect_request_descriptor_id);
        assert_eq!(record.source_admitted_action_id, descriptor.source_admitted_action_id);
        assert_eq!(record.dispatch_record_id, descriptor.dispatch_record_id);
        assert_eq!(record.dispatch_route_id, descriptor.dispatch_route_id);
        assert_eq!(record.requested_effect, descriptor.requested_effect);
        assert_eq!(record.declared_ui_capability, descriptor.declared_ui_capability);
        assert_eq!(
            record.declared_runtime_capability_requirement,
            descriptor.declared_runtime_capability_requirement
        );
        assert_eq!(record.runtime_capability_namespace, descriptor.runtime_capability_namespace);
        assert_eq!(record.lifecycle_precondition, descriptor.lifecycle_precondition);
        assert_eq!(record.target_policy, descriptor.target_policy);
        assert_eq!(record.trace_requirement, descriptor.trace_requirement);
        assert_eq!(record.policy_gate_namespace, descriptor.policy_gate_namespace);
        assert_eq!(record.scope, descriptor.scope);
        assert_eq!(record.audit_requirement, descriptor.audit_requirement);
        assert_eq!(record.audit_visibility, descriptor.audit_visibility);
    }

    #[test]
    fn record_preserves_separate_runtime_mutation_and_host_path_requirements() {
        let descriptor = committed_effect_descriptor();
        let record = record_interaction_committed_effect(&descriptor);

        assert_eq!(record.runtime_mutation_requirement, descriptor.runtime_mutation_requirement);
        assert_eq!(record.host_path_requirement, descriptor.host_path_requirement);
    }

    #[test]
    fn record_status_fields_are_inert() {
        let descriptor = committed_effect_descriptor();
        let record = record_interaction_committed_effect(&descriptor);

        assert_eq!(record.record_status, InteractionCommittedEffectRecordStatus::Recorded);
        assert_eq!(
            record.runtime_mutation_status,
            InteractionCommittedEffectRuntimeMutationStatus::NotPerformed
        );
        assert_eq!(
            record.host_path_status,
            InteractionCommittedEffectHostPathStatus::NotEntered
        );
    }

    #[test]
    fn record_is_not_authority() {
        let descriptor = committed_effect_descriptor();
        let record = record_interaction_committed_effect(&descriptor);

        assert!(!record.is_host_abi_authority());
        assert!(!record.is_vm_authority());
        assert!(!record.is_execution_authority());
        assert!(!record.is_runtime_mutation());
        assert!(!record.is_audit_backend());
        assert!(!record.is_host_runtime_path());
    }

    #[test]
    fn generation_is_deterministic() {
        let left = record_interaction_committed_effect(&committed_effect_descriptor());
        let right = record_interaction_committed_effect(&committed_effect_descriptor());

        assert_eq!(left, right);
    }
}
