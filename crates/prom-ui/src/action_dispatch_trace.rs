//! Semantic UI action dispatch trace scaffold.
//!
//! This module records a visible trace report for dispatch metadata. It does
//! not implement a dispatcher, dispatch execution, effect requests, VM/Host
//! ABI calls, audit authority, or runtime mutation.

use crate::action_admission::{
    InteractionActionAdmissionEffectRelationship, InteractionActionAdmissionPolicyGateNamespace,
    InteractionActionAdmissionTraceRequirement,
};
use crate::action_binding::{InteractionActionBindingId, InteractionActionName};
use crate::action_dispatch_record::{
    InteractionSemanticActionDispatchBlockReason, InteractionSemanticActionDispatchRecord,
    InteractionSemanticActionDispatchRecordId, InteractionSemanticActionDispatchRecordStatus,
};
use crate::action_dispatch_route::{
    InteractionSemanticActionDispatchEffectEligibility, InteractionSemanticActionDispatchRouteId,
    InteractionSemanticActionDispatchRouteKind,
};
use crate::admitted_action::InteractionAdmittedSemanticActionId;
use crate::interaction::InteractionIntentKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionSemanticActionDispatchTraceStatus {
    Recorded,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionSemanticActionDispatchTraceReason {
    RouteRecorded,
    MissingRoute,
    PolicyBlocked,
    EffectBoundaryBlocked,
    UnknownBlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionSemanticActionDispatchTraceReport {
    pub record_id: InteractionSemanticActionDispatchRecordId,
    pub route_id: InteractionSemanticActionDispatchRouteId,
    pub admitted_action_id: InteractionAdmittedSemanticActionId,
    pub action: InteractionActionName,
    pub source_intent: InteractionIntentKind,
    pub binding_id: InteractionActionBindingId,
    pub route: InteractionSemanticActionDispatchRouteKind,
    pub record_status: InteractionSemanticActionDispatchRecordStatus,
    pub trace_status: InteractionSemanticActionDispatchTraceStatus,
    pub block_reason: InteractionSemanticActionDispatchBlockReason,
    pub trace_reason: InteractionSemanticActionDispatchTraceReason,
    pub trace_requirement: InteractionActionAdmissionTraceRequirement,
    pub effect_relationship: InteractionActionAdmissionEffectRelationship,
    pub policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace,
    pub effect_eligibility: InteractionSemanticActionDispatchEffectEligibility,
}

pub fn trace_interaction_semantic_action_dispatch(
    record: &InteractionSemanticActionDispatchRecord,
) -> InteractionSemanticActionDispatchTraceReport {
    InteractionSemanticActionDispatchTraceReport {
        record_id: record.id,
        route_id: record.route_id,
        admitted_action_id: record.admitted_action_id,
        action: record.action,
        source_intent: record.source_intent,
        binding_id: record.binding_id,
        route: record.route,
        record_status: record.status,
        trace_status: trace_status_for_record(record.status),
        block_reason: record.block_reason,
        trace_reason: trace_reason_for_record(record.status, record.block_reason),
        trace_requirement: record.trace_requirement,
        effect_relationship: record.effect_relationship,
        policy_gate_namespace: record.policy_gate_namespace,
        effect_eligibility: record.effect_eligibility,
    }
}

const fn trace_status_for_record(
    status: InteractionSemanticActionDispatchRecordStatus,
) -> InteractionSemanticActionDispatchTraceStatus {
    match status {
        InteractionSemanticActionDispatchRecordStatus::Recorded => {
            InteractionSemanticActionDispatchTraceStatus::Recorded
        }
        InteractionSemanticActionDispatchRecordStatus::BlockedMissingRoute
        | InteractionSemanticActionDispatchRecordStatus::BlockedPolicy
        | InteractionSemanticActionDispatchRecordStatus::BlockedEffectBoundary
        | InteractionSemanticActionDispatchRecordStatus::BlockedUnknown => {
            InteractionSemanticActionDispatchTraceStatus::Blocked
        }
    }
}

const fn trace_reason_for_record(
    status: InteractionSemanticActionDispatchRecordStatus,
    block_reason: InteractionSemanticActionDispatchBlockReason,
) -> InteractionSemanticActionDispatchTraceReason {
    match status {
        InteractionSemanticActionDispatchRecordStatus::Recorded => {
            InteractionSemanticActionDispatchTraceReason::RouteRecorded
        }
        InteractionSemanticActionDispatchRecordStatus::BlockedMissingRoute => {
            InteractionSemanticActionDispatchTraceReason::MissingRoute
        }
        InteractionSemanticActionDispatchRecordStatus::BlockedPolicy => {
            InteractionSemanticActionDispatchTraceReason::PolicyBlocked
        }
        InteractionSemanticActionDispatchRecordStatus::BlockedEffectBoundary => {
            InteractionSemanticActionDispatchTraceReason::EffectBoundaryBlocked
        }
        InteractionSemanticActionDispatchRecordStatus::BlockedUnknown => match block_reason {
            InteractionSemanticActionDispatchBlockReason::MissingRoute => {
                InteractionSemanticActionDispatchTraceReason::MissingRoute
            }
            InteractionSemanticActionDispatchBlockReason::Policy => {
                InteractionSemanticActionDispatchTraceReason::PolicyBlocked
            }
            InteractionSemanticActionDispatchBlockReason::EffectBoundary => {
                InteractionSemanticActionDispatchTraceReason::EffectBoundaryBlocked
            }
            InteractionSemanticActionDispatchBlockReason::None
            | InteractionSemanticActionDispatchBlockReason::Unknown => {
                InteractionSemanticActionDispatchTraceReason::UnknownBlocked
            }
        },
    }
}

impl InteractionSemanticActionDispatchTraceReport {
    pub const fn is_recorded(&self) -> bool {
        matches!(
            self.trace_status,
            InteractionSemanticActionDispatchTraceStatus::Recorded
        )
    }

    pub const fn is_blocked(&self) -> bool {
        matches!(
            self.trace_status,
            InteractionSemanticActionDispatchTraceStatus::Blocked
        )
    }

    pub const fn is_dispatcher(&self) -> bool {
        false
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

    pub const fn is_audit_authority(&self) -> bool {
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
    use crate::action_dispatch_record::{
        InteractionSemanticActionDispatchBlockReason, InteractionSemanticActionDispatchRecord,
        InteractionSemanticActionDispatchRecordId, InteractionSemanticActionDispatchRecordStatus,
    };
    use crate::action_dispatch_route::{
        InteractionSemanticActionDispatchEffectEligibility,
        InteractionSemanticActionDispatchRouteId, InteractionSemanticActionDispatchRouteKind,
    };
    use crate::admitted_action::InteractionAdmittedSemanticActionId;
    use crate::interaction::InteractionIntentKind;

    fn recorded_record() -> InteractionSemanticActionDispatchRecord {
        InteractionSemanticActionDispatchRecord {
            id: InteractionSemanticActionDispatchRecordId(1),
            route_id: InteractionSemanticActionDispatchRouteId(1),
            admitted_action_id: InteractionAdmittedSemanticActionId(1),
            action: InteractionActionName::CloseWindow,
            source_intent: InteractionIntentKind::Close,
            binding_id: InteractionActionBindingId(1),
            route: InteractionSemanticActionDispatchRouteKind::LocalUiStateCandidate,
            status: InteractionSemanticActionDispatchRecordStatus::Recorded,
            block_reason: InteractionSemanticActionDispatchBlockReason::None,
            trace_requirement: InteractionActionAdmissionTraceRequirement::Required,
            effect_relationship: InteractionActionAdmissionEffectRelationship::NoEffect,
            policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace::CoreUi,
            effect_eligibility: InteractionSemanticActionDispatchEffectEligibility::NoEffect,
        }
    }

    #[test]
    fn recorded_dispatch_record_traces_as_recorded() {
        let record = recorded_record();

        let trace = trace_interaction_semantic_action_dispatch(&record);

        assert_eq!(trace.record_id, record.id);
        assert_eq!(
            trace.trace_status,
            InteractionSemanticActionDispatchTraceStatus::Recorded
        );
        assert_eq!(
            trace.trace_reason,
            InteractionSemanticActionDispatchTraceReason::RouteRecorded
        );
        assert!(trace.is_recorded());
        assert!(!trace.is_blocked());
    }

    #[test]
    fn missing_route_traces_as_blocked() {
        let mut record = recorded_record();
        record.status = InteractionSemanticActionDispatchRecordStatus::BlockedMissingRoute;
        record.block_reason = InteractionSemanticActionDispatchBlockReason::MissingRoute;

        let trace = trace_interaction_semantic_action_dispatch(&record);

        assert_eq!(
            trace.trace_status,
            InteractionSemanticActionDispatchTraceStatus::Blocked
        );
        assert_eq!(
            trace.trace_reason,
            InteractionSemanticActionDispatchTraceReason::MissingRoute
        );
        assert!(trace.is_blocked());
    }

    #[test]
    fn effect_boundary_block_traces_correctly() {
        let mut record = recorded_record();
        record.status = InteractionSemanticActionDispatchRecordStatus::BlockedEffectBoundary;
        record.block_reason = InteractionSemanticActionDispatchBlockReason::EffectBoundary;

        let trace = trace_interaction_semantic_action_dispatch(&record);

        assert_eq!(
            trace.trace_reason,
            InteractionSemanticActionDispatchTraceReason::EffectBoundaryBlocked
        );
    }

    #[test]
    fn trace_preserves_dispatch_metadata() {
        let record = recorded_record();

        let trace = trace_interaction_semantic_action_dispatch(&record);

        assert_eq!(trace.route_id, record.route_id);
        assert_eq!(trace.admitted_action_id, record.admitted_action_id);
        assert_eq!(trace.action, record.action);
        assert_eq!(trace.source_intent, record.source_intent);
        assert_eq!(trace.binding_id, record.binding_id);
        assert_eq!(trace.route, record.route);
    }

    #[test]
    fn trace_preserves_trace_effect_policy_metadata() {
        let record = recorded_record();

        let trace = trace_interaction_semantic_action_dispatch(&record);

        assert_eq!(trace.trace_requirement, record.trace_requirement);
        assert_eq!(trace.effect_relationship, record.effect_relationship);
        assert_eq!(trace.policy_gate_namespace, record.policy_gate_namespace);
        assert_eq!(trace.effect_eligibility, record.effect_eligibility);
    }

    #[test]
    fn dispatch_trace_is_not_authority() {
        let record = recorded_record();

        let trace = trace_interaction_semantic_action_dispatch(&record);

        assert!(!trace.is_dispatcher());
        assert!(!trace.is_execution_authority());
        assert!(!trace.is_effect_request());
        assert!(!trace.is_runtime_mutation());
        assert!(!trace.is_audit_authority());
    }

    #[test]
    fn dispatch_trace_generation_is_deterministic() {
        let record = recorded_record();

        let first = trace_interaction_semantic_action_dispatch(&record);
        let second = trace_interaction_semantic_action_dispatch(&record);

        assert_eq!(first, second);
    }
}
