//! Semantic UI action dispatch record scaffold.
//!
//! This module records inert dispatch metadata for a future dispatcher. It
//! does not implement dispatch execution, effect requests, VM/Host ABI calls,
//! or runtime mutation.

use crate::action_admission::{
    InteractionActionAdmissionEffectRelationship, InteractionActionAdmissionPolicyGateNamespace,
    InteractionActionAdmissionTraceRequirement,
};
use crate::action_binding::{InteractionActionBindingId, InteractionActionName};
use crate::action_dispatch_route::{
    InteractionSemanticActionDispatchEffectEligibility,
    InteractionSemanticActionDispatchRouteDescriptor, InteractionSemanticActionDispatchRouteId,
    InteractionSemanticActionDispatchRouteKind,
};
use crate::admitted_action::InteractionAdmittedSemanticActionId;
use crate::interaction::InteractionIntentKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InteractionSemanticActionDispatchRecordId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionSemanticActionDispatchRecordStatus {
    Recorded,
    BlockedMissingRoute,
    BlockedPolicy,
    BlockedEffectBoundary,
    BlockedUnknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionSemanticActionDispatchBlockReason {
    None,
    MissingRoute,
    Policy,
    EffectBoundary,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionSemanticActionDispatchRecord {
    pub id: InteractionSemanticActionDispatchRecordId,
    pub route_id: InteractionSemanticActionDispatchRouteId,
    pub admitted_action_id: InteractionAdmittedSemanticActionId,
    pub action: InteractionActionName,
    pub source_intent: InteractionIntentKind,
    pub binding_id: InteractionActionBindingId,
    pub route: InteractionSemanticActionDispatchRouteKind,
    pub status: InteractionSemanticActionDispatchRecordStatus,
    pub block_reason: InteractionSemanticActionDispatchBlockReason,
    pub trace_requirement: InteractionActionAdmissionTraceRequirement,
    pub effect_relationship: InteractionActionAdmissionEffectRelationship,
    pub policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace,
    pub effect_eligibility: InteractionSemanticActionDispatchEffectEligibility,
}

pub fn record_interaction_semantic_action_dispatch(
    route: &InteractionSemanticActionDispatchRouteDescriptor,
) -> InteractionSemanticActionDispatchRecord {
    InteractionSemanticActionDispatchRecord {
        id: InteractionSemanticActionDispatchRecordId(route.id.0),
        route_id: route.id,
        admitted_action_id: route.admitted_action_id,
        action: route.action,
        source_intent: route.source_intent,
        binding_id: route.binding_id,
        route: route.route,
        status: record_status_for_route(route.route),
        block_reason: block_reason_for_route(route.route),
        trace_requirement: route.trace_requirement,
        effect_relationship: route.effect_relationship,
        policy_gate_namespace: route.policy_gate_namespace,
        effect_eligibility: route.effect_eligibility,
    }
}

const fn record_status_for_route(
    route: InteractionSemanticActionDispatchRouteKind,
) -> InteractionSemanticActionDispatchRecordStatus {
    match route {
        InteractionSemanticActionDispatchRouteKind::Unknown => {
            InteractionSemanticActionDispatchRecordStatus::BlockedMissingRoute
        }
        _ => InteractionSemanticActionDispatchRecordStatus::Recorded,
    }
}

const fn block_reason_for_route(
    route: InteractionSemanticActionDispatchRouteKind,
) -> InteractionSemanticActionDispatchBlockReason {
    match route {
        InteractionSemanticActionDispatchRouteKind::Unknown => {
            InteractionSemanticActionDispatchBlockReason::MissingRoute
        }
        _ => InteractionSemanticActionDispatchBlockReason::None,
    }
}

impl InteractionSemanticActionDispatchRecord {
    pub const fn is_recorded(&self) -> bool {
        matches!(
            self.status,
            InteractionSemanticActionDispatchRecordStatus::Recorded
        )
    }

    pub const fn is_blocked(&self) -> bool {
        !self.is_recorded()
    }

    pub const fn is_execution_authority(&self) -> bool {
        false
    }

    pub const fn is_effect_request(&self) -> bool {
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
    use crate::action_admission::{
        InteractionActionAdmissionEffectRelationship,
        InteractionActionAdmissionPolicyGateNamespace, InteractionActionAdmissionTraceRequirement,
    };
    use crate::action_binding::{InteractionActionBindingId, InteractionActionName};
    use crate::action_dispatch_route::{
        InteractionSemanticActionDispatchEffectEligibility,
        InteractionSemanticActionDispatchRouteDescriptor, InteractionSemanticActionDispatchRouteId,
        InteractionSemanticActionDispatchRouteKind,
    };
    use crate::admitted_action::InteractionAdmittedSemanticActionId;
    use crate::interaction::InteractionIntentKind;

    fn local_route() -> InteractionSemanticActionDispatchRouteDescriptor {
        InteractionSemanticActionDispatchRouteDescriptor {
            id: InteractionSemanticActionDispatchRouteId(1),
            admitted_action_id: InteractionAdmittedSemanticActionId(1),
            action: InteractionActionName::CloseWindow,
            source_intent: InteractionIntentKind::Close,
            binding_id: InteractionActionBindingId(1),
            route: InteractionSemanticActionDispatchRouteKind::LocalUiStateCandidate,
            trace_requirement: InteractionActionAdmissionTraceRequirement::Required,
            effect_relationship: InteractionActionAdmissionEffectRelationship::NoEffect,
            policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace::CoreUi,
            effect_eligibility: InteractionSemanticActionDispatchEffectEligibility::NoEffect,
        }
    }

    #[test]
    fn local_route_records_dispatch_metadata() {
        let route = local_route();

        let record = record_interaction_semantic_action_dispatch(&route);

        assert_eq!(record.route_id, route.id);
        assert_eq!(record.admitted_action_id, route.admitted_action_id);
        assert_eq!(record.action, route.action);
        assert_eq!(record.source_intent, route.source_intent);
        assert_eq!(record.binding_id, route.binding_id);
        assert_eq!(
            record.status,
            InteractionSemanticActionDispatchRecordStatus::Recorded
        );
        assert_eq!(
            record.block_reason,
            InteractionSemanticActionDispatchBlockReason::None
        );
    }

    #[test]
    fn unknown_route_records_blocked_missing_route() {
        let mut route = local_route();
        route.route = InteractionSemanticActionDispatchRouteKind::Unknown;

        let record = record_interaction_semantic_action_dispatch(&route);

        assert_eq!(
            record.status,
            InteractionSemanticActionDispatchRecordStatus::BlockedMissingRoute
        );
        assert_eq!(
            record.block_reason,
            InteractionSemanticActionDispatchBlockReason::MissingRoute
        );
        assert!(record.is_blocked());
    }

    #[test]
    fn effect_candidate_remains_recorded_not_effect_request() {
        let mut route = local_route();
        route.route = InteractionSemanticActionDispatchRouteKind::EffectRequestCandidate;
        route.effect_eligibility =
            InteractionSemanticActionDispatchEffectEligibility::RequiresFutureEffectBoundary;

        let record = record_interaction_semantic_action_dispatch(&route);

        assert_eq!(
            record.status,
            InteractionSemanticActionDispatchRecordStatus::Recorded
        );
        assert_eq!(
            record.block_reason,
            InteractionSemanticActionDispatchBlockReason::None
        );
        assert_eq!(
            record.effect_eligibility,
            InteractionSemanticActionDispatchEffectEligibility::RequiresFutureEffectBoundary
        );
        assert!(!record.is_effect_request());
    }

    #[test]
    fn record_preserves_trace_effect_policy_metadata() {
        let route = local_route();

        let record = record_interaction_semantic_action_dispatch(&route);

        assert_eq!(record.trace_requirement, route.trace_requirement);
        assert_eq!(record.effect_relationship, route.effect_relationship);
        assert_eq!(record.policy_gate_namespace, route.policy_gate_namespace);
        assert_eq!(record.effect_eligibility, route.effect_eligibility);
    }

    #[test]
    fn dispatch_record_is_not_authority() {
        let route = local_route();

        let record = record_interaction_semantic_action_dispatch(&route);

        assert!(!record.is_execution_authority());
        assert!(!record.is_effect_request());
        assert!(!record.is_runtime_mutation());
        assert!(!record.calls_vm_or_host_abi());
    }

    #[test]
    fn dispatch_record_generation_is_deterministic() {
        let route = local_route();

        let first = record_interaction_semantic_action_dispatch(&route);
        let second = record_interaction_semantic_action_dispatch(&route);

        assert_eq!(first, second);
    }
}
