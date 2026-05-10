//! Semantic UI capability denial trace scaffold.
//!
//! This module records an inert trace for a denied UI capability admission
//! result. It does not implement a capability checker, capability grant,
//! runtime capability mapping, prepared effects, committed effects, VM/Host
//! ABI calls, or runtime mutation.

use crate::action_admission::{
    InteractionActionAdmissionPolicyGateNamespace, InteractionActionAdmissionTraceRequirement,
};
use crate::action_dispatch_record::InteractionSemanticActionDispatchRecordId;
use crate::action_dispatch_route::InteractionSemanticActionDispatchRouteId;
use crate::admitted_action::InteractionAdmittedSemanticActionId;
use crate::effect_request::{
    InteractionEffectRequestDenialBehavior, InteractionEffectRequestDescriptorId,
    InteractionEffectRequestKind, InteractionEffectRequestLifecyclePrecondition,
    InteractionEffectRequestRuntimeCapability, InteractionEffectRequestScope,
    InteractionEffectRequestTargetPolicy, InteractionEffectRequestUiCapability,
};
use crate::ui_capability_admission::{
    InteractionUiCapabilityAdmissionDescriptorId,
    InteractionUiCapabilityAdmissionRuntimeMappingRequirement,
};
use crate::ui_capability_admission_result::{
    InteractionUiCapabilityAdmissionDecisionStatus, InteractionUiCapabilityAdmissionDenialReason,
    InteractionUiCapabilityAdmissionMissingRequirement, InteractionUiCapabilityAdmissionResult,
    InteractionUiCapabilityAdmissionResultId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InteractionUiCapabilityDenialTraceId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionUiCapabilityDenialTraceStatus {
    Traced,
    NotDenied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionUiCapabilityDenialRetryHint {
    None,
    ProvideUiCapability,
    SatisfyLifecycle,
    ResolveTarget,
    AdjustPolicy,
    CompleteRuntimeMapping,
    InspectUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionUiCapabilityDenialTrace {
    pub id: InteractionUiCapabilityDenialTraceId,
    pub result_id: InteractionUiCapabilityAdmissionResultId,
    pub descriptor_id: InteractionUiCapabilityAdmissionDescriptorId,
    pub effect_request_descriptor_id: InteractionEffectRequestDescriptorId,
    pub source_admitted_action_id: InteractionAdmittedSemanticActionId,
    pub dispatch_record_id: InteractionSemanticActionDispatchRecordId,
    pub dispatch_route_id: InteractionSemanticActionDispatchRouteId,
    pub status: InteractionUiCapabilityDenialTraceStatus,
    pub denial_reason: InteractionUiCapabilityAdmissionDenialReason,
    pub missing_requirement: InteractionUiCapabilityAdmissionMissingRequirement,
    pub retry_hint: InteractionUiCapabilityDenialRetryHint,
    pub requested_effect: InteractionEffectRequestKind,
    pub declared_ui_capability: InteractionEffectRequestUiCapability,
    pub declared_runtime_capability_requirement: InteractionEffectRequestRuntimeCapability,
    pub lifecycle_precondition: InteractionEffectRequestLifecyclePrecondition,
    pub target_policy: InteractionEffectRequestTargetPolicy,
    pub denial_behavior: InteractionEffectRequestDenialBehavior,
    pub trace_requirement: InteractionActionAdmissionTraceRequirement,
    pub policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace,
    pub scope: InteractionEffectRequestScope,
    pub runtime_mapping_requirement: InteractionUiCapabilityAdmissionRuntimeMappingRequirement,
}

pub fn trace_interaction_ui_capability_denial(
    result: &InteractionUiCapabilityAdmissionResult,
) -> Option<InteractionUiCapabilityDenialTrace> {
    if !matches!(
        result.status,
        InteractionUiCapabilityAdmissionDecisionStatus::Denied
    ) {
        return None;
    }

    Some(InteractionUiCapabilityDenialTrace {
        id: InteractionUiCapabilityDenialTraceId(result.id.0),
        result_id: result.id,
        descriptor_id: result.descriptor_id,
        effect_request_descriptor_id: result.effect_request_descriptor_id,
        source_admitted_action_id: result.source_admitted_action_id,
        dispatch_record_id: result.dispatch_record_id,
        dispatch_route_id: result.dispatch_route_id,
        status: InteractionUiCapabilityDenialTraceStatus::Traced,
        denial_reason: result.denial_reason,
        missing_requirement: result.missing_requirement,
        retry_hint: map_retry_hint(result.denial_reason),
        requested_effect: result.requested_effect,
        declared_ui_capability: result.declared_ui_capability,
        declared_runtime_capability_requirement: result
            .declared_runtime_capability_requirement,
        lifecycle_precondition: result.lifecycle_precondition,
        target_policy: result.target_policy,
        denial_behavior: result.denial_behavior,
        trace_requirement: result.trace_requirement,
        policy_gate_namespace: result.policy_gate_namespace,
        scope: result.scope,
        runtime_mapping_requirement: result.runtime_mapping_requirement,
    })
}

const fn map_retry_hint(
    denial_reason: InteractionUiCapabilityAdmissionDenialReason,
) -> InteractionUiCapabilityDenialRetryHint {
    match denial_reason {
        InteractionUiCapabilityAdmissionDenialReason::None => {
            InteractionUiCapabilityDenialRetryHint::None
        }
        InteractionUiCapabilityAdmissionDenialReason::MissingUiCapability => {
            InteractionUiCapabilityDenialRetryHint::ProvideUiCapability
        }
        InteractionUiCapabilityAdmissionDenialReason::LifecycleBlocked => {
            InteractionUiCapabilityDenialRetryHint::SatisfyLifecycle
        }
        InteractionUiCapabilityAdmissionDenialReason::TargetUnavailable
        | InteractionUiCapabilityAdmissionDenialReason::TargetInvalid => {
            InteractionUiCapabilityDenialRetryHint::ResolveTarget
        }
        InteractionUiCapabilityAdmissionDenialReason::PolicyDenied => {
            InteractionUiCapabilityDenialRetryHint::AdjustPolicy
        }
        InteractionUiCapabilityAdmissionDenialReason::RuntimeMappingRequired => {
            InteractionUiCapabilityDenialRetryHint::CompleteRuntimeMapping
        }
        InteractionUiCapabilityAdmissionDenialReason::Unknown => {
            InteractionUiCapabilityDenialRetryHint::InspectUnknown
        }
    }
}

impl InteractionUiCapabilityDenialTrace {
    pub const fn is_retry_execution(&self) -> bool {
        false
    }

    pub const fn grants_ui_capability(&self) -> bool {
        false
    }

    pub const fn grants_runtime_capability(&self) -> bool {
        false
    }

    pub const fn is_runtime_mapping(&self) -> bool {
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

    pub const fn calls_vm_or_host_abi(&self) -> bool {
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
    use crate::ui_capability_admission::describe_interaction_ui_capability_admission;
    use crate::ui_capability_admission_result::{
        record_interaction_ui_capability_admitted_result,
        record_interaction_ui_capability_denied_result,
    };

    fn effect_request_descriptor() -> crate::effect_request::InteractionEffectRequestDescriptor {
        let dispatch_trace = InteractionSemanticActionDispatchTraceReport {
            record_id: InteractionSemanticActionDispatchRecordId(61),
            route_id: InteractionSemanticActionDispatchRouteId(61),
            admitted_action_id: InteractionAdmittedSemanticActionId(61),
            action: InteractionActionName::PrepareEffect,
            source_intent: InteractionIntentKind::Submit,
            binding_id: InteractionActionBindingId(61),
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

    fn denied_result(
        reason: InteractionUiCapabilityAdmissionDenialReason,
        missing: InteractionUiCapabilityAdmissionMissingRequirement,
    ) -> InteractionUiCapabilityAdmissionResult {
        let effect_request = effect_request_descriptor();
        let descriptor = describe_interaction_ui_capability_admission(&effect_request);

        record_interaction_ui_capability_denied_result(&descriptor, reason, missing)
    }

    fn admitted_result() -> InteractionUiCapabilityAdmissionResult {
        let effect_request = effect_request_descriptor();
        let descriptor = describe_interaction_ui_capability_admission(&effect_request);

        record_interaction_ui_capability_admitted_result(&descriptor)
    }

    #[test]
    fn denied_result_produces_denial_trace() {
        let result = denied_result(
            InteractionUiCapabilityAdmissionDenialReason::PolicyDenied,
            InteractionUiCapabilityAdmissionMissingRequirement::Policy,
        );

        let trace = trace_interaction_ui_capability_denial(&result)
            .expect("denied result should produce trace");

        assert_eq!(trace.status, InteractionUiCapabilityDenialTraceStatus::Traced);
        assert_eq!(trace.result_id, result.id);
        assert_eq!(trace.id, InteractionUiCapabilityDenialTraceId(result.id.0));
    }

    #[test]
    fn admitted_result_produces_no_denial_trace() {
        let result = admitted_result();

        let trace = trace_interaction_ui_capability_denial(&result);

        assert!(trace.is_none());
    }

    #[test]
    fn trace_preserves_identity_and_metadata() {
        let result = denied_result(
            InteractionUiCapabilityAdmissionDenialReason::MissingUiCapability,
            InteractionUiCapabilityAdmissionMissingRequirement::UiCapability,
        );

        let trace = trace_interaction_ui_capability_denial(&result)
            .expect("denied result should produce trace");

        assert_eq!(trace.result_id, result.id);
        assert_eq!(trace.descriptor_id, result.descriptor_id);
        assert_eq!(
            trace.effect_request_descriptor_id,
            result.effect_request_descriptor_id
        );
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
        assert_eq!(trace.lifecycle_precondition, result.lifecycle_precondition);
        assert_eq!(trace.target_policy, result.target_policy);
        assert_eq!(trace.denial_behavior, result.denial_behavior);
        assert_eq!(trace.trace_requirement, result.trace_requirement);
        assert_eq!(trace.policy_gate_namespace, result.policy_gate_namespace);
        assert_eq!(trace.scope, result.scope);
        assert_eq!(
            trace.runtime_mapping_requirement,
            result.runtime_mapping_requirement
        );
    }

    #[test]
    fn retry_hints_map_from_denial_reason() {
        let cases = [
            (
                InteractionUiCapabilityAdmissionDenialReason::None,
                InteractionUiCapabilityDenialRetryHint::None,
            ),
            (
                InteractionUiCapabilityAdmissionDenialReason::MissingUiCapability,
                InteractionUiCapabilityDenialRetryHint::ProvideUiCapability,
            ),
            (
                InteractionUiCapabilityAdmissionDenialReason::LifecycleBlocked,
                InteractionUiCapabilityDenialRetryHint::SatisfyLifecycle,
            ),
            (
                InteractionUiCapabilityAdmissionDenialReason::TargetUnavailable,
                InteractionUiCapabilityDenialRetryHint::ResolveTarget,
            ),
            (
                InteractionUiCapabilityAdmissionDenialReason::TargetInvalid,
                InteractionUiCapabilityDenialRetryHint::ResolveTarget,
            ),
            (
                InteractionUiCapabilityAdmissionDenialReason::PolicyDenied,
                InteractionUiCapabilityDenialRetryHint::AdjustPolicy,
            ),
            (
                InteractionUiCapabilityAdmissionDenialReason::RuntimeMappingRequired,
                InteractionUiCapabilityDenialRetryHint::CompleteRuntimeMapping,
            ),
            (
                InteractionUiCapabilityAdmissionDenialReason::Unknown,
                InteractionUiCapabilityDenialRetryHint::InspectUnknown,
            ),
        ];

        for (reason, expected) in cases {
            let result = denied_result(
                reason,
                InteractionUiCapabilityAdmissionMissingRequirement::Unknown,
            );

            let trace = trace_interaction_ui_capability_denial(&result)
                .expect("denied result should produce trace");

            assert_eq!(trace.retry_hint, expected);
        }
    }

    #[test]
    fn trace_is_not_authority() {
        let result = denied_result(
            InteractionUiCapabilityAdmissionDenialReason::PolicyDenied,
            InteractionUiCapabilityAdmissionMissingRequirement::Policy,
        );

        let trace = trace_interaction_ui_capability_denial(&result)
            .expect("denied result should produce trace");

        assert!(!trace.is_retry_execution());
        assert!(!trace.grants_ui_capability());
        assert!(!trace.grants_runtime_capability());
        assert!(!trace.is_runtime_mapping());
        assert!(!trace.is_prepared_effect());
        assert!(!trace.is_committed_effect());
        assert!(!trace.is_execution_authority());
        assert!(!trace.is_runtime_mutation());
        assert!(!trace.calls_vm_or_host_abi());
    }

    #[test]
    fn generation_is_deterministic() {
        let result = denied_result(
            InteractionUiCapabilityAdmissionDenialReason::RuntimeMappingRequired,
            InteractionUiCapabilityAdmissionMissingRequirement::RuntimeMapping,
        );

        let first = trace_interaction_ui_capability_denial(&result);
        let second = trace_interaction_ui_capability_denial(&result);

        assert_eq!(first, second);
    }
}
