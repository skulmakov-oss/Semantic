//! Semantic UI host runtime effect denial trace scaffold.
//!
//! This module records an inert trace for a denied host runtime effect
//! boundary result. It does not implement Host ABI calls, VM calls, effect
//! execution, runtime mutation, audit backend writes, or a host runtime effect
//! path.

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
    InteractionHostRuntimeAuditWriteRequirement, InteractionHostRuntimeEffectBoundaryDescriptorId,
    InteractionHostRuntimeEffectPathRequirement, InteractionHostRuntimeMutationRequirement,
};
use crate::host_runtime_effect_result::{
    InteractionHostRuntimeEffectBoundaryDenialReason,
    InteractionHostRuntimeEffectBoundaryMissingRequirement,
    InteractionHostRuntimeEffectBoundaryResult,
    InteractionHostRuntimeEffectBoundaryResultId,
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
pub struct InteractionHostRuntimeEffectDenialTraceId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionHostRuntimeEffectDenialTraceStatus {
    Traced,
    NotDenied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionHostRuntimeEffectDenialRetryHint {
    None,
    ProvideCommittedRecord,
    ProvideRuntimeCapability,
    SatisfyLifecycle,
    ResolveTarget,
    AdjustPolicy,
    SatisfyAuditRequirement,
    InspectHostBoundary,
    InspectUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionHostRuntimeEffectDenialTrace {
    pub id: InteractionHostRuntimeEffectDenialTraceId,
    pub result_id: InteractionHostRuntimeEffectBoundaryResultId,
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
    pub status: InteractionHostRuntimeEffectDenialTraceStatus,
    pub denial_reason: InteractionHostRuntimeEffectBoundaryDenialReason,
    pub missing_requirement: InteractionHostRuntimeEffectBoundaryMissingRequirement,
    pub retry_hint: InteractionHostRuntimeEffectDenialRetryHint,
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

pub fn trace_interaction_host_runtime_effect_denial(
    result: &InteractionHostRuntimeEffectBoundaryResult,
) -> Option<InteractionHostRuntimeEffectDenialTrace> {
    if !matches!(
        result.status,
        crate::host_runtime_effect_result::InteractionHostRuntimeEffectBoundaryDecisionStatus::Denied
    ) {
        return None;
    }

    Some(InteractionHostRuntimeEffectDenialTrace {
        id: InteractionHostRuntimeEffectDenialTraceId(result.id.0),
        result_id: result.id,
        descriptor_id: result.descriptor_id,
        committed_effect_record_id: result.committed_effect_record_id,
        committed_effect_descriptor_id: result.committed_effect_descriptor_id,
        commit_boundary_result_id: result.commit_boundary_result_id,
        commit_boundary_descriptor_id: result.commit_boundary_descriptor_id,
        prepared_effect_result_id: result.prepared_effect_result_id,
        prepared_effect_descriptor_id: result.prepared_effect_descriptor_id,
        runtime_capability_mapping_result_id: result.runtime_capability_mapping_result_id,
        runtime_capability_mapping_descriptor_id: result.runtime_capability_mapping_descriptor_id,
        ui_capability_admission_result_id: result.ui_capability_admission_result_id,
        ui_capability_admission_descriptor_id: result.ui_capability_admission_descriptor_id,
        effect_request_descriptor_id: result.effect_request_descriptor_id,
        source_admitted_action_id: result.source_admitted_action_id,
        dispatch_record_id: result.dispatch_record_id,
        dispatch_route_id: result.dispatch_route_id,
        status: InteractionHostRuntimeEffectDenialTraceStatus::Traced,
        denial_reason: result.denial_reason,
        missing_requirement: result.missing_requirement,
        retry_hint: map_retry_hint(result.denial_reason),
        requested_effect: result.requested_effect,
        declared_ui_capability: result.declared_ui_capability,
        declared_runtime_capability_requirement: result.declared_runtime_capability_requirement,
        runtime_capability_namespace: result.runtime_capability_namespace,
        lifecycle_precondition: result.lifecycle_precondition,
        target_policy: result.target_policy,
        trace_requirement: result.trace_requirement,
        policy_gate_namespace: result.policy_gate_namespace,
        scope: result.scope,
        audit_requirement: result.audit_requirement,
        audit_visibility: result.audit_visibility,
        runtime_mutation_requirement: result.runtime_mutation_requirement,
        host_path_requirement: result.host_path_requirement,
        record_status: result.record_status,
        runtime_mutation_status: result.runtime_mutation_status,
        host_path_status: result.host_path_status,
        future_host_path_requirement: result.future_host_path_requirement,
        future_runtime_mutation_requirement: result.future_runtime_mutation_requirement,
        future_audit_write_requirement: result.future_audit_write_requirement,
    })
}

const fn map_retry_hint(
    denial_reason: InteractionHostRuntimeEffectBoundaryDenialReason,
) -> InteractionHostRuntimeEffectDenialRetryHint {
    match denial_reason {
        InteractionHostRuntimeEffectBoundaryDenialReason::None => {
            InteractionHostRuntimeEffectDenialRetryHint::None
        }
        InteractionHostRuntimeEffectBoundaryDenialReason::MissingCommittedRecord => {
            InteractionHostRuntimeEffectDenialRetryHint::ProvideCommittedRecord
        }
        InteractionHostRuntimeEffectBoundaryDenialReason::MissingRuntimeCapability => {
            InteractionHostRuntimeEffectDenialRetryHint::ProvideRuntimeCapability
        }
        InteractionHostRuntimeEffectBoundaryDenialReason::LifecycleBlocked => {
            InteractionHostRuntimeEffectDenialRetryHint::SatisfyLifecycle
        }
        InteractionHostRuntimeEffectBoundaryDenialReason::TargetUnavailable
        | InteractionHostRuntimeEffectBoundaryDenialReason::TargetInvalid => {
            InteractionHostRuntimeEffectDenialRetryHint::ResolveTarget
        }
        InteractionHostRuntimeEffectBoundaryDenialReason::PolicyDenied => {
            InteractionHostRuntimeEffectDenialRetryHint::AdjustPolicy
        }
        InteractionHostRuntimeEffectBoundaryDenialReason::AuditRequired => {
            InteractionHostRuntimeEffectDenialRetryHint::SatisfyAuditRequirement
        }
        InteractionHostRuntimeEffectBoundaryDenialReason::HostBoundaryDenied => {
            InteractionHostRuntimeEffectDenialRetryHint::InspectHostBoundary
        }
        InteractionHostRuntimeEffectBoundaryDenialReason::Unknown => {
            InteractionHostRuntimeEffectDenialRetryHint::InspectUnknown
        }
    }
}

impl InteractionHostRuntimeEffectDenialTrace {
    pub const fn is_retry_execution(&self) -> bool {
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

    pub const fn is_host_runtime_path(&self) -> bool {
        false
    }

    pub const fn grants_capability(&self) -> bool {
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
    use crate::host_runtime_effect_result::{
        record_interaction_host_runtime_effect_boundary_admitted_result,
        record_interaction_host_runtime_effect_boundary_denied_result,
        InteractionHostRuntimeEffectBoundaryDecisionStatus,
    };
    use crate::interaction::InteractionIntentKind;
    use crate::prepared_effect::describe_interaction_prepared_effect;
    use crate::prepared_effect_result::record_interaction_prepared_effect_result;
    use crate::runtime_capability_mapping::describe_interaction_runtime_capability_mapping;
    use crate::runtime_capability_mapping_result::record_interaction_runtime_capability_mapped_result;
    use crate::ui_capability_admission::describe_interaction_ui_capability_admission;
    use crate::ui_capability_admission_result::record_interaction_ui_capability_admitted_result;

    fn effect_request_descriptor() -> crate::effect_request::InteractionEffectRequestDescriptor {
        let dispatch_trace = InteractionSemanticActionDispatchTraceReport {
            record_id: InteractionSemanticActionDispatchRecordId(181),
            route_id: InteractionSemanticActionDispatchRouteId(181),
            admitted_action_id: InteractionAdmittedSemanticActionId(181),
            action: InteractionActionName::PrepareEffect,
            source_intent: InteractionIntentKind::Submit,
            binding_id: InteractionActionBindingId(181),
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

    fn host_runtime_result(
        status: InteractionHostRuntimeEffectBoundaryDecisionStatus,
        denial_reason: InteractionHostRuntimeEffectBoundaryDenialReason,
        missing_requirement: InteractionHostRuntimeEffectBoundaryMissingRequirement,
    ) -> crate::host_runtime_effect_result::InteractionHostRuntimeEffectBoundaryResult {
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
        let descriptor = describe_interaction_host_runtime_effect_boundary(&committed_record)
            .expect("committed record should produce host runtime effect boundary descriptor");

        match status {
            InteractionHostRuntimeEffectBoundaryDecisionStatus::Denied => {
                record_interaction_host_runtime_effect_boundary_denied_result(
                    &descriptor,
                    denial_reason,
                    missing_requirement,
                )
            }
            InteractionHostRuntimeEffectBoundaryDecisionStatus::AdmittedToHostPath => {
                record_interaction_host_runtime_effect_boundary_admitted_result(&descriptor)
            }
        }
    }

    fn denied_result() -> crate::host_runtime_effect_result::InteractionHostRuntimeEffectBoundaryResult
    {
        host_runtime_result(
            InteractionHostRuntimeEffectBoundaryDecisionStatus::Denied,
            InteractionHostRuntimeEffectBoundaryDenialReason::PolicyDenied,
            InteractionHostRuntimeEffectBoundaryMissingRequirement::Policy,
        )
    }

    fn admitted_result() -> crate::host_runtime_effect_result::InteractionHostRuntimeEffectBoundaryResult
    {
        host_runtime_result(
            InteractionHostRuntimeEffectBoundaryDecisionStatus::AdmittedToHostPath,
            InteractionHostRuntimeEffectBoundaryDenialReason::None,
            InteractionHostRuntimeEffectBoundaryMissingRequirement::None,
        )
    }

    #[test]
    fn denied_result_produces_denial_trace() {
        let result = denied_result();

        let trace = trace_interaction_host_runtime_effect_denial(&result)
            .expect("denied result should produce trace");

        assert_eq!(trace.status, InteractionHostRuntimeEffectDenialTraceStatus::Traced);
        assert_eq!(trace.result_id, result.id);
        assert_eq!(trace.id, InteractionHostRuntimeEffectDenialTraceId(result.id.0));
    }

    #[test]
    fn admitted_result_produces_no_denial_trace() {
        let result = admitted_result();

        assert!(trace_interaction_host_runtime_effect_denial(&result).is_none());
    }

    #[test]
    fn trace_preserves_source_capability_audit_status_metadata() {
        let result = denied_result();
        let trace = trace_interaction_host_runtime_effect_denial(&result)
            .expect("denied result should produce trace");

        assert_eq!(trace.descriptor_id, result.descriptor_id);
        assert_eq!(trace.committed_effect_record_id, result.committed_effect_record_id);
        assert_eq!(trace.committed_effect_descriptor_id, result.committed_effect_descriptor_id);
        assert_eq!(trace.commit_boundary_result_id, result.commit_boundary_result_id);
        assert_eq!(trace.commit_boundary_descriptor_id, result.commit_boundary_descriptor_id);
        assert_eq!(trace.prepared_effect_result_id, result.prepared_effect_result_id);
        assert_eq!(trace.prepared_effect_descriptor_id, result.prepared_effect_descriptor_id);
        assert_eq!(
            trace.runtime_capability_mapping_result_id,
            result.runtime_capability_mapping_result_id
        );
        assert_eq!(
            trace.runtime_capability_mapping_descriptor_id,
            result.runtime_capability_mapping_descriptor_id
        );
        assert_eq!(trace.ui_capability_admission_result_id, result.ui_capability_admission_result_id);
        assert_eq!(
            trace.ui_capability_admission_descriptor_id,
            result.ui_capability_admission_descriptor_id
        );
        assert_eq!(trace.effect_request_descriptor_id, result.effect_request_descriptor_id);
        assert_eq!(trace.source_admitted_action_id, result.source_admitted_action_id);
        assert_eq!(trace.dispatch_record_id, result.dispatch_record_id);
        assert_eq!(trace.dispatch_route_id, result.dispatch_route_id);
        assert_eq!(trace.denial_reason, result.denial_reason);
        assert_eq!(trace.missing_requirement, result.missing_requirement);
        assert_eq!(trace.requested_effect, result.requested_effect);
        assert_eq!(trace.declared_ui_capability, result.declared_ui_capability);
        assert_eq!(
            trace.declared_runtime_capability_requirement,
            result.declared_runtime_capability_requirement
        );
        assert_eq!(trace.runtime_capability_namespace, result.runtime_capability_namespace);
        assert_eq!(trace.lifecycle_precondition, result.lifecycle_precondition);
        assert_eq!(trace.target_policy, result.target_policy);
        assert_eq!(trace.trace_requirement, result.trace_requirement);
        assert_eq!(trace.policy_gate_namespace, result.policy_gate_namespace);
        assert_eq!(trace.scope, result.scope);
        assert_eq!(trace.audit_requirement, result.audit_requirement);
        assert_eq!(trace.audit_visibility, result.audit_visibility);
        assert_eq!(trace.runtime_mutation_requirement, result.runtime_mutation_requirement);
        assert_eq!(trace.host_path_requirement, result.host_path_requirement);
        assert_eq!(trace.record_status, result.record_status);
        assert_eq!(trace.runtime_mutation_status, result.runtime_mutation_status);
        assert_eq!(trace.host_path_status, result.host_path_status);
        assert_eq!(
            trace.future_host_path_requirement,
            result.future_host_path_requirement
        );
        assert_eq!(
            trace.future_runtime_mutation_requirement,
            result.future_runtime_mutation_requirement
        );
        assert_eq!(
            trace.future_audit_write_requirement,
            result.future_audit_write_requirement
        );
    }

    #[test]
    fn retry_hints_map_from_denial_reason() {
        let result = denied_result();
        let trace = trace_interaction_host_runtime_effect_denial(&result)
            .expect("denied result should produce trace");

        assert_eq!(
            trace.retry_hint,
            InteractionHostRuntimeEffectDenialRetryHint::AdjustPolicy
        );
    }

    #[test]
    fn trace_is_not_authority() {
        let result = denied_result();
        let trace = trace_interaction_host_runtime_effect_denial(&result)
            .expect("denied result should produce trace");

        assert!(!trace.is_retry_execution());
        assert!(!trace.is_host_abi_authority());
        assert!(!trace.is_vm_authority());
        assert!(!trace.is_execution_authority());
        assert!(!trace.is_runtime_mutation());
        assert!(!trace.is_audit_backend());
        assert!(!trace.is_host_runtime_path());
        assert!(!trace.grants_capability());
    }

    #[test]
    fn generation_is_deterministic() {
        let left = trace_interaction_host_runtime_effect_denial(&denied_result())
            .expect("denied result should produce trace");
        let right = trace_interaction_host_runtime_effect_denial(&denied_result())
            .expect("denied result should produce trace");

        assert_eq!(left, right);
    }
}
