//! Semantic UI commit boundary result scaffold.
//!
//! This module records an inert decision outcome for a commit boundary
//! descriptor. It does not implement committed effects, Host ABI calls, VM
//! calls, effect execution, runtime mutation, or an audit backend.

use crate::action_admission::{
    InteractionActionAdmissionPolicyGateNamespace, InteractionActionAdmissionTraceRequirement,
};
use crate::action_dispatch_record::InteractionSemanticActionDispatchRecordId;
use crate::action_dispatch_route::InteractionSemanticActionDispatchRouteId;
use crate::admitted_action::InteractionAdmittedSemanticActionId;
use crate::commit_boundary::{
    InteractionCommitAuditRequirement, InteractionCommitBoundaryDescriptor,
    InteractionCommitBoundaryDescriptorId, InteractionCommitBoundaryFutureCommittedEffectShape,
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
pub struct InteractionCommitBoundaryResultId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionCommitBoundaryDecisionStatus {
    Committed,
    Denied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionCommitBoundaryDenialReason {
    None,
    MissingPreparedEffect,
    LifecycleBlocked,
    TargetUnavailable,
    TargetInvalid,
    PolicyDenied,
    AuditRequired,
    HostBoundaryDenied,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionCommitBoundaryMissingRequirement {
    None,
    PreparedEffect,
    Lifecycle,
    Target,
    Policy,
    Audit,
    HostBoundary,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionCommitBoundaryResult {
    pub id: InteractionCommitBoundaryResultId,
    pub descriptor_id: InteractionCommitBoundaryDescriptorId,
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
    pub status: InteractionCommitBoundaryDecisionStatus,
    pub denial_reason: InteractionCommitBoundaryDenialReason,
    pub missing_requirement: InteractionCommitBoundaryMissingRequirement,
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

pub fn record_interaction_commit_boundary_committed_result(
    descriptor: &InteractionCommitBoundaryDescriptor,
) -> InteractionCommitBoundaryResult {
    build_result(
        descriptor,
        InteractionCommitBoundaryDecisionStatus::Committed,
        InteractionCommitBoundaryDenialReason::None,
        InteractionCommitBoundaryMissingRequirement::None,
    )
}

pub fn record_interaction_commit_boundary_denied_result(
    descriptor: &InteractionCommitBoundaryDescriptor,
    denial_reason: InteractionCommitBoundaryDenialReason,
    missing_requirement: InteractionCommitBoundaryMissingRequirement,
) -> InteractionCommitBoundaryResult {
    build_result(
        descriptor,
        InteractionCommitBoundaryDecisionStatus::Denied,
        denial_reason,
        missing_requirement,
    )
}

fn build_result(
    descriptor: &InteractionCommitBoundaryDescriptor,
    status: InteractionCommitBoundaryDecisionStatus,
    denial_reason: InteractionCommitBoundaryDenialReason,
    missing_requirement: InteractionCommitBoundaryMissingRequirement,
) -> InteractionCommitBoundaryResult {
    InteractionCommitBoundaryResult {
        id: InteractionCommitBoundaryResultId(descriptor.id.0),
        descriptor_id: descriptor.id,
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
        declared_runtime_capability_requirement: descriptor.declared_runtime_capability_requirement,
        runtime_capability_namespace: descriptor.runtime_capability_namespace,
        lifecycle_precondition: descriptor.lifecycle_precondition,
        target_policy: descriptor.target_policy,
        trace_requirement: descriptor.trace_requirement,
        policy_gate_namespace: descriptor.policy_gate_namespace,
        scope: descriptor.scope,
        audit_requirement: descriptor.audit_requirement,
        future_committed_effect_shape: descriptor.future_committed_effect_shape,
    }
}

impl InteractionCommitBoundaryResult {
    pub const fn is_committed(&self) -> bool {
        matches!(
            self.status,
            InteractionCommitBoundaryDecisionStatus::Committed
        )
    }

    pub const fn is_denied(&self) -> bool {
        matches!(self.status, InteractionCommitBoundaryDecisionStatus::Denied)
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
    use crate::commit_boundary::describe_interaction_commit_boundary;
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
            record_id: InteractionSemanticActionDispatchRecordId(131),
            route_id: InteractionSemanticActionDispatchRouteId(131),
            admitted_action_id: InteractionAdmittedSemanticActionId(131),
            action: InteractionActionName::PrepareEffect,
            source_intent: InteractionIntentKind::Submit,
            binding_id: InteractionActionBindingId(131),
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

    fn commit_boundary_descriptor() -> InteractionCommitBoundaryDescriptor {
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
        let prepared_result = record_interaction_prepared_effect_result(&prepared_descriptor);

        describe_interaction_commit_boundary(&prepared_result)
            .expect("prepared result should produce commit boundary descriptor")
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
    fn committed_result_is_built_from_descriptor() {
        let descriptor = commit_boundary_descriptor();

        let result = record_interaction_commit_boundary_committed_result(&descriptor);

        assert_eq!(
            result.status,
            InteractionCommitBoundaryDecisionStatus::Committed
        );
        assert_eq!(result.descriptor_id, descriptor.id);
        assert_eq!(
            result.id,
            InteractionCommitBoundaryResultId(descriptor.id.0)
        );
    }

    #[test]
    fn denied_result_is_built_from_descriptor() {
        let descriptor = commit_boundary_descriptor();

        let result = record_interaction_commit_boundary_denied_result(
            &descriptor,
            InteractionCommitBoundaryDenialReason::PolicyDenied,
            InteractionCommitBoundaryMissingRequirement::Policy,
        );

        assert_eq!(
            result.status,
            InteractionCommitBoundaryDecisionStatus::Denied
        );
        assert_eq!(result.descriptor_id, descriptor.id);
        assert_eq!(
            result.id,
            InteractionCommitBoundaryResultId(descriptor.id.0)
        );
    }

    #[test]
    fn result_preserves_source_and_capability_metadata() {
        let descriptor = commit_boundary_descriptor();

        let result = record_interaction_commit_boundary_committed_result(&descriptor);

        assert_eq!(
            result.prepared_effect_result_id,
            descriptor.prepared_effect_result_id
        );
        assert_eq!(
            result.prepared_effect_descriptor_id,
            descriptor.prepared_effect_descriptor_id
        );
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
    fn committed_result_has_none_denial_metadata() {
        let descriptor = commit_boundary_descriptor();

        let result = record_interaction_commit_boundary_committed_result(&descriptor);

        assert_eq!(
            result.denial_reason,
            InteractionCommitBoundaryDenialReason::None
        );
        assert_eq!(
            result.missing_requirement,
            InteractionCommitBoundaryMissingRequirement::None
        );
        assert!(result.is_committed());
        assert!(!result.is_denied());
    }

    #[test]
    fn denied_result_preserves_denial_metadata() {
        let descriptor = commit_boundary_descriptor();

        let result = record_interaction_commit_boundary_denied_result(
            &descriptor,
            InteractionCommitBoundaryDenialReason::AuditRequired,
            InteractionCommitBoundaryMissingRequirement::Audit,
        );

        assert_eq!(
            result.denial_reason,
            InteractionCommitBoundaryDenialReason::AuditRequired
        );
        assert_eq!(
            result.missing_requirement,
            InteractionCommitBoundaryMissingRequirement::Audit
        );
        assert!(result.is_denied());
        assert!(!result.is_committed());
    }

    #[test]
    fn result_is_not_authority() {
        let descriptor = commit_boundary_descriptor();

        let result = record_interaction_commit_boundary_committed_result(&descriptor);

        assert!(!result.is_committed_effect());
        assert!(!result.is_host_abi_authority());
        assert!(!result.is_vm_authority());
        assert!(!result.is_execution_authority());
        assert!(!result.is_runtime_mutation());
        assert!(!result.is_audit_backend());
    }

    #[test]
    fn deterministic_result_generation() {
        let descriptor = commit_boundary_descriptor();

        let first = record_interaction_commit_boundary_committed_result(&descriptor);
        let second = record_interaction_commit_boundary_committed_result(&descriptor);

        assert_eq!(first, second);
    }

    #[test]
    fn denied_prepared_result_returns_none_for_descriptor_helper() {
        let result = denied_prepared_result();

        let descriptor = describe_interaction_commit_boundary(&result);

        assert!(descriptor.is_none());
    }
}
