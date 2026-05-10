//! Semantic UI host runtime effect boundary result scaffold.
//!
//! This module records an inert result for the future host runtime effect
//! boundary. It does not implement Host ABI calls, VM calls, effect execution,
//! runtime mutation, audit backend writes, or a host runtime effect path.

use crate::action_admission::{
    InteractionActionAdmissionPolicyGateNamespace, InteractionActionAdmissionTraceRequirement,
};
use crate::action_dispatch_record::InteractionSemanticActionDispatchRecordId;
use crate::action_dispatch_route::InteractionSemanticActionDispatchRouteId;
use crate::admitted_action::InteractionAdmittedSemanticActionId;
use crate::commit_boundary::{
    InteractionCommitAuditRequirement, InteractionCommitBoundaryDescriptorId,
};
use crate::commit_boundary_result::InteractionCommitBoundaryResultId;
use crate::committed_effect::{
    InteractionCommittedEffectAuditVisibility, InteractionCommittedEffectDescriptorId,
    InteractionCommittedEffectHostPathRequirement,
    InteractionCommittedEffectRuntimeMutationRequirement,
};
use crate::committed_effect_record::{
    InteractionCommittedEffectHostPathStatus, InteractionCommittedEffectRecordId,
    InteractionCommittedEffectRecordStatus, InteractionCommittedEffectRuntimeMutationStatus,
};
use crate::effect_request::{
    InteractionEffectRequestDescriptorId, InteractionEffectRequestKind,
    InteractionEffectRequestLifecyclePrecondition, InteractionEffectRequestRuntimeCapability,
    InteractionEffectRequestScope, InteractionEffectRequestTargetPolicy,
    InteractionEffectRequestUiCapability,
};
use crate::host_runtime_effect::{
    InteractionHostRuntimeAuditWriteRequirement,
    InteractionHostRuntimeEffectBoundaryDescriptor,
    InteractionHostRuntimeEffectBoundaryDescriptorId,
    InteractionHostRuntimeEffectPathRequirement,
    InteractionHostRuntimeMutationRequirement,
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
pub struct InteractionHostRuntimeEffectBoundaryResultId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionHostRuntimeEffectBoundaryDecisionStatus {
    AdmittedToHostPath,
    Denied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionHostRuntimeEffectBoundaryDenialReason {
    None,
    MissingCommittedRecord,
    MissingRuntimeCapability,
    LifecycleBlocked,
    TargetUnavailable,
    TargetInvalid,
    PolicyDenied,
    AuditRequired,
    HostBoundaryDenied,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionHostRuntimeEffectBoundaryMissingRequirement {
    None,
    CommittedRecord,
    RuntimeCapability,
    Lifecycle,
    Target,
    Policy,
    Audit,
    HostBoundary,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionHostRuntimeEffectBoundaryResult {
    pub id: InteractionHostRuntimeEffectBoundaryResultId,
    pub descriptor_id: InteractionHostRuntimeEffectBoundaryDescriptorId,
    pub committed_effect_record_id: InteractionCommittedEffectRecordId,
    pub committed_effect_descriptor_id: InteractionCommittedEffectDescriptorId,
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
    pub status: InteractionHostRuntimeEffectBoundaryDecisionStatus,
    pub denial_reason: InteractionHostRuntimeEffectBoundaryDenialReason,
    pub missing_requirement: InteractionHostRuntimeEffectBoundaryMissingRequirement,
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
    pub future_host_path_requirement: InteractionHostRuntimeEffectPathRequirement,
    pub future_runtime_mutation_requirement: InteractionHostRuntimeMutationRequirement,
    pub future_audit_write_requirement: InteractionHostRuntimeAuditWriteRequirement,
}

pub fn record_interaction_host_runtime_effect_boundary_admitted_result(
    descriptor: &InteractionHostRuntimeEffectBoundaryDescriptor,
) -> InteractionHostRuntimeEffectBoundaryResult {
    build_result(
        descriptor,
        InteractionHostRuntimeEffectBoundaryDecisionStatus::AdmittedToHostPath,
        InteractionHostRuntimeEffectBoundaryDenialReason::None,
        InteractionHostRuntimeEffectBoundaryMissingRequirement::None,
    )
}

pub fn record_interaction_host_runtime_effect_boundary_denied_result(
    descriptor: &InteractionHostRuntimeEffectBoundaryDescriptor,
    denial_reason: InteractionHostRuntimeEffectBoundaryDenialReason,
    missing_requirement: InteractionHostRuntimeEffectBoundaryMissingRequirement,
) -> InteractionHostRuntimeEffectBoundaryResult {
    build_result(
        descriptor,
        InteractionHostRuntimeEffectBoundaryDecisionStatus::Denied,
        denial_reason,
        missing_requirement,
    )
}

fn build_result(
    descriptor: &InteractionHostRuntimeEffectBoundaryDescriptor,
    status: InteractionHostRuntimeEffectBoundaryDecisionStatus,
    denial_reason: InteractionHostRuntimeEffectBoundaryDenialReason,
    missing_requirement: InteractionHostRuntimeEffectBoundaryMissingRequirement,
) -> InteractionHostRuntimeEffectBoundaryResult {
    InteractionHostRuntimeEffectBoundaryResult {
        id: InteractionHostRuntimeEffectBoundaryResultId(descriptor.id.0),
        descriptor_id: descriptor.id,
        committed_effect_record_id: descriptor.committed_effect_record_id,
        committed_effect_descriptor_id: descriptor.committed_effect_descriptor_id,
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
        status,
        denial_reason,
        missing_requirement,
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
        record_status: descriptor.record_status,
        runtime_mutation_status: descriptor.runtime_mutation_status,
        host_path_status: descriptor.host_path_status,
        future_host_path_requirement: descriptor.future_host_path_requirement,
        future_runtime_mutation_requirement: descriptor.future_runtime_mutation_requirement,
        future_audit_write_requirement: descriptor.future_audit_write_requirement,
    }
}

impl InteractionHostRuntimeEffectBoundaryResult {
    pub const fn is_admitted_to_host_path(&self) -> bool {
        matches!(
            self.status,
            InteractionHostRuntimeEffectBoundaryDecisionStatus::AdmittedToHostPath
        )
    }

    pub const fn is_denied(&self) -> bool {
        matches!(
            self.status,
            InteractionHostRuntimeEffectBoundaryDecisionStatus::Denied
        )
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
    use crate::committed_effect_record::record_interaction_committed_effect;
    use crate::effect_request::describe_interaction_effect_request;
    use crate::host_runtime_effect::describe_interaction_host_runtime_effect_boundary;
    use crate::interaction::InteractionIntentKind;
    use crate::prepared_effect::describe_interaction_prepared_effect;
    use crate::prepared_effect_result::record_interaction_prepared_effect_result;
    use crate::runtime_capability_mapping::describe_interaction_runtime_capability_mapping;
    use crate::runtime_capability_mapping_result::record_interaction_runtime_capability_mapped_result;
    use crate::ui_capability_admission::describe_interaction_ui_capability_admission;
    use crate::ui_capability_admission_result::record_interaction_ui_capability_admitted_result;

    fn effect_request_descriptor() -> crate::effect_request::InteractionEffectRequestDescriptor {
        let dispatch_trace = InteractionSemanticActionDispatchTraceReport {
            record_id: InteractionSemanticActionDispatchRecordId(171),
            route_id: InteractionSemanticActionDispatchRouteId(171),
            admitted_action_id: InteractionAdmittedSemanticActionId(171),
            action: InteractionActionName::PrepareEffect,
            source_intent: InteractionIntentKind::Submit,
            binding_id: InteractionActionBindingId(171),
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

    fn host_runtime_descriptor() -> InteractionHostRuntimeEffectBoundaryDescriptor {
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
        let committed_descriptor = describe_interaction_committed_effect(&commit_result)
            .expect("committed result should produce committed effect descriptor");
        let committed_record = record_interaction_committed_effect(&committed_descriptor);

        describe_interaction_host_runtime_effect_boundary(&committed_record)
            .expect("committed record should produce host runtime effect boundary descriptor")
    }

    #[test]
    fn admitted_result_is_built_from_descriptor() {
        let descriptor = host_runtime_descriptor();
        let result = record_interaction_host_runtime_effect_boundary_admitted_result(&descriptor);

        assert_eq!(
            result.status,
            InteractionHostRuntimeEffectBoundaryDecisionStatus::AdmittedToHostPath
        );
        assert_eq!(result.id, InteractionHostRuntimeEffectBoundaryResultId(descriptor.id.0));
    }

    #[test]
    fn denied_result_is_built_from_descriptor() {
        let descriptor = host_runtime_descriptor();
        let result = record_interaction_host_runtime_effect_boundary_denied_result(
            &descriptor,
            InteractionHostRuntimeEffectBoundaryDenialReason::PolicyDenied,
            InteractionHostRuntimeEffectBoundaryMissingRequirement::Policy,
        );

        assert_eq!(
            result.status,
            InteractionHostRuntimeEffectBoundaryDecisionStatus::Denied
        );
        assert_eq!(result.id, InteractionHostRuntimeEffectBoundaryResultId(descriptor.id.0));
    }

    #[test]
    fn result_preserves_source_capability_audit_status_metadata() {
        let descriptor = host_runtime_descriptor();
        let result = record_interaction_host_runtime_effect_boundary_admitted_result(&descriptor);

        assert_eq!(result.descriptor_id, descriptor.id);
        assert_eq!(result.committed_effect_record_id, descriptor.committed_effect_record_id);
        assert_eq!(result.committed_effect_descriptor_id, descriptor.committed_effect_descriptor_id);
        assert_eq!(result.commit_boundary_result_id, descriptor.commit_boundary_result_id);
        assert_eq!(result.commit_boundary_descriptor_id, descriptor.commit_boundary_descriptor_id);
        assert_eq!(result.prepared_effect_result_id, descriptor.prepared_effect_result_id);
        assert_eq!(result.prepared_effect_descriptor_id, descriptor.prepared_effect_descriptor_id);
        assert_eq!(
            result.runtime_capability_mapping_result_id,
            descriptor.runtime_capability_mapping_result_id
        );
        assert_eq!(
            result.runtime_capability_mapping_descriptor_id,
            descriptor.runtime_capability_mapping_descriptor_id
        );
        assert_eq!(result.ui_capability_admission_result_id, descriptor.ui_capability_admission_result_id);
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
        assert_eq!(result.runtime_capability_namespace, descriptor.runtime_capability_namespace);
        assert_eq!(result.lifecycle_precondition, descriptor.lifecycle_precondition);
        assert_eq!(result.target_policy, descriptor.target_policy);
        assert_eq!(result.trace_requirement, descriptor.trace_requirement);
        assert_eq!(result.policy_gate_namespace, descriptor.policy_gate_namespace);
        assert_eq!(result.scope, descriptor.scope);
        assert_eq!(result.audit_requirement, descriptor.audit_requirement);
        assert_eq!(result.audit_visibility, descriptor.audit_visibility);
        assert_eq!(result.runtime_mutation_requirement, descriptor.runtime_mutation_requirement);
        assert_eq!(result.host_path_requirement, descriptor.host_path_requirement);
        assert_eq!(result.record_status, descriptor.record_status);
        assert_eq!(result.runtime_mutation_status, descriptor.runtime_mutation_status);
        assert_eq!(result.host_path_status, descriptor.host_path_status);
        assert_eq!(
            result.future_host_path_requirement,
            descriptor.future_host_path_requirement
        );
        assert_eq!(
            result.future_runtime_mutation_requirement,
            descriptor.future_runtime_mutation_requirement
        );
        assert_eq!(
            result.future_audit_write_requirement,
            descriptor.future_audit_write_requirement
        );
    }

    #[test]
    fn admitted_result_has_none_denial_metadata() {
        let descriptor = host_runtime_descriptor();
        let result = record_interaction_host_runtime_effect_boundary_admitted_result(&descriptor);

        assert_eq!(
            result.denial_reason,
            InteractionHostRuntimeEffectBoundaryDenialReason::None
        );
        assert_eq!(
            result.missing_requirement,
            InteractionHostRuntimeEffectBoundaryMissingRequirement::None
        );
    }

    #[test]
    fn denied_result_preserves_denial_metadata() {
        let descriptor = host_runtime_descriptor();
        let result = record_interaction_host_runtime_effect_boundary_denied_result(
            &descriptor,
            InteractionHostRuntimeEffectBoundaryDenialReason::AuditRequired,
            InteractionHostRuntimeEffectBoundaryMissingRequirement::Audit,
        );

        assert_eq!(
            result.denial_reason,
            InteractionHostRuntimeEffectBoundaryDenialReason::AuditRequired
        );
        assert_eq!(
            result.missing_requirement,
            InteractionHostRuntimeEffectBoundaryMissingRequirement::Audit
        );
    }

    #[test]
    fn result_is_not_authority() {
        let descriptor = host_runtime_descriptor();
        let result = record_interaction_host_runtime_effect_boundary_admitted_result(&descriptor);

        assert!(!result.is_host_abi_authority());
        assert!(!result.is_vm_authority());
        assert!(!result.is_execution_authority());
        assert!(!result.is_runtime_mutation());
        assert!(!result.is_audit_backend());
        assert!(!result.is_host_runtime_path());
    }

    #[test]
    fn generation_is_deterministic() {
        let left =
            record_interaction_host_runtime_effect_boundary_admitted_result(&host_runtime_descriptor());
        let right =
            record_interaction_host_runtime_effect_boundary_admitted_result(&host_runtime_descriptor());

        assert_eq!(left, right);
    }
}
