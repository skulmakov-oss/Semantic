//! Semantic UI action dispatch summary scaffold.
//!
//! This module aggregates dispatch trace reports into a compact summary for
//! future UI/debug/Workbench consumption. It does not implement a dispatcher,
//! action execution, effect requests, VM/Host ABI calls, audit authority, or
//! runtime mutation.

use alloc::vec::Vec;

use crate::action_dispatch_route::{
    InteractionSemanticActionDispatchEffectEligibility, InteractionSemanticActionDispatchRouteKind,
};
use crate::action_dispatch_trace::{
    InteractionSemanticActionDispatchTraceReason, InteractionSemanticActionDispatchTraceReport,
    InteractionSemanticActionDispatchTraceStatus,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InteractionSemanticActionDispatchRouteCount {
    pub route: InteractionSemanticActionDispatchRouteKind,
    pub count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InteractionSemanticActionDispatchTraceReasonCount {
    pub reason: InteractionSemanticActionDispatchTraceReason,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionSemanticActionDispatchSummary {
    pub total: usize,
    pub recorded: usize,
    pub blocked: usize,
    pub effect_candidate_count: usize,
    pub routes: Vec<InteractionSemanticActionDispatchRouteCount>,
    pub reasons: Vec<InteractionSemanticActionDispatchTraceReasonCount>,
}

pub fn summarize_interaction_semantic_action_dispatches(
    traces: &[InteractionSemanticActionDispatchTraceReport],
) -> InteractionSemanticActionDispatchSummary {
    let mut summary = InteractionSemanticActionDispatchSummary {
        total: traces.len(),
        recorded: 0,
        blocked: 0,
        effect_candidate_count: 0,
        routes: Vec::new(),
        reasons: Vec::new(),
    };

    for trace in traces {
        match trace.trace_status {
            InteractionSemanticActionDispatchTraceStatus::Recorded => {
                summary.recorded += 1;
            }
            InteractionSemanticActionDispatchTraceStatus::Blocked => {
                summary.blocked += 1;
            }
        }

        if is_effect_candidate(trace.effect_eligibility) {
            summary.effect_candidate_count += 1;
        }

        increment_route_count(&mut summary.routes, trace.route);
        increment_reason_count(&mut summary.reasons, trace.trace_reason);
    }

    summary
}

fn increment_route_count(
    counts: &mut Vec<InteractionSemanticActionDispatchRouteCount>,
    route: InteractionSemanticActionDispatchRouteKind,
) {
    if let Some(entry) = counts.iter_mut().find(|entry| entry.route == route) {
        entry.count += 1;
    } else {
        counts.push(InteractionSemanticActionDispatchRouteCount { route, count: 1 });
    }
}

fn increment_reason_count(
    counts: &mut Vec<InteractionSemanticActionDispatchTraceReasonCount>,
    reason: InteractionSemanticActionDispatchTraceReason,
) {
    if let Some(entry) = counts.iter_mut().find(|entry| entry.reason == reason) {
        entry.count += 1;
    } else {
        counts.push(InteractionSemanticActionDispatchTraceReasonCount { reason, count: 1 });
    }
}

const fn is_effect_candidate(
    eligibility: InteractionSemanticActionDispatchEffectEligibility,
) -> bool {
    matches!(
        eligibility,
        InteractionSemanticActionDispatchEffectEligibility::RequiresFutureEffectBoundary
    )
}

impl InteractionSemanticActionDispatchSummary {
    pub fn is_empty(&self) -> bool {
        self.total == 0
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
    use crate::interaction::InteractionIntentKind;

    fn recorded_trace(id: u64) -> InteractionSemanticActionDispatchTraceReport {
        InteractionSemanticActionDispatchTraceReport {
            record_id: InteractionSemanticActionDispatchRecordId(id),
            route_id: InteractionSemanticActionDispatchRouteId(id),
            admitted_action_id: InteractionAdmittedSemanticActionId(id),
            action: InteractionActionName::CloseWindow,
            source_intent: InteractionIntentKind::Close,
            binding_id: InteractionActionBindingId(id),
            route: InteractionSemanticActionDispatchRouteKind::LocalUiStateCandidate,
            record_status: InteractionSemanticActionDispatchRecordStatus::Recorded,
            trace_status: InteractionSemanticActionDispatchTraceStatus::Recorded,
            block_reason: InteractionSemanticActionDispatchBlockReason::None,
            trace_reason: InteractionSemanticActionDispatchTraceReason::RouteRecorded,
            trace_requirement: InteractionActionAdmissionTraceRequirement::Required,
            effect_relationship: InteractionActionAdmissionEffectRelationship::NoEffect,
            policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace::CoreUi,
            effect_eligibility: InteractionSemanticActionDispatchEffectEligibility::NoEffect,
        }
    }

    fn blocked_missing_route_trace(id: u64) -> InteractionSemanticActionDispatchTraceReport {
        InteractionSemanticActionDispatchTraceReport {
            record_id: InteractionSemanticActionDispatchRecordId(id),
            route_id: InteractionSemanticActionDispatchRouteId(id),
            admitted_action_id: InteractionAdmittedSemanticActionId(id),
            action: InteractionActionName::Unknown,
            source_intent: InteractionIntentKind::Unknown,
            binding_id: InteractionActionBindingId(id),
            route: InteractionSemanticActionDispatchRouteKind::Unknown,
            record_status: InteractionSemanticActionDispatchRecordStatus::BlockedMissingRoute,
            trace_status: InteractionSemanticActionDispatchTraceStatus::Blocked,
            block_reason: InteractionSemanticActionDispatchBlockReason::MissingRoute,
            trace_reason: InteractionSemanticActionDispatchTraceReason::MissingRoute,
            trace_requirement: InteractionActionAdmissionTraceRequirement::Required,
            effect_relationship: InteractionActionAdmissionEffectRelationship::NoEffect,
            policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace::CoreUi,
            effect_eligibility: InteractionSemanticActionDispatchEffectEligibility::NoEffect,
        }
    }

    fn effect_candidate_trace(id: u64) -> InteractionSemanticActionDispatchTraceReport {
        InteractionSemanticActionDispatchTraceReport {
            record_id: InteractionSemanticActionDispatchRecordId(id),
            route_id: InteractionSemanticActionDispatchRouteId(id),
            admitted_action_id: InteractionAdmittedSemanticActionId(id),
            action: InteractionActionName::PrepareEffect,
            source_intent: InteractionIntentKind::Submit,
            binding_id: InteractionActionBindingId(id),
            route: InteractionSemanticActionDispatchRouteKind::EffectRequestCandidate,
            record_status: InteractionSemanticActionDispatchRecordStatus::Recorded,
            trace_status: InteractionSemanticActionDispatchTraceStatus::Recorded,
            block_reason: InteractionSemanticActionDispatchBlockReason::None,
            trace_reason: InteractionSemanticActionDispatchTraceReason::RouteRecorded,
            trace_requirement: InteractionActionAdmissionTraceRequirement::Required,
            effect_relationship:
                InteractionActionAdmissionEffectRelationship::RequiresSeparateEffectAdmission,
            policy_gate_namespace: InteractionActionAdmissionPolicyGateNamespace::CoreUi,
            effect_eligibility:
                InteractionSemanticActionDispatchEffectEligibility::RequiresFutureEffectBoundary,
        }
    }

    #[test]
    fn empty_dispatch_summary_is_zeroed() {
        let summary = summarize_interaction_semantic_action_dispatches(&[]);

        assert!(summary.is_empty());
        assert_eq!(summary.total, 0);
        assert_eq!(summary.recorded, 0);
        assert_eq!(summary.blocked, 0);
        assert_eq!(summary.effect_candidate_count, 0);
        assert!(summary.routes.is_empty());
        assert!(summary.reasons.is_empty());
    }

    #[test]
    fn summary_counts_recorded_and_blocked() {
        let traces = vec![
            recorded_trace(1),
            blocked_missing_route_trace(2),
            effect_candidate_trace(3),
        ];

        let summary = summarize_interaction_semantic_action_dispatches(&traces);

        assert_eq!(summary.total, 3);
        assert_eq!(summary.recorded, 2);
        assert_eq!(summary.blocked, 1);
    }

    #[test]
    fn summary_counts_effect_candidates() {
        let traces = vec![recorded_trace(1), effect_candidate_trace(2)];

        let summary = summarize_interaction_semantic_action_dispatches(&traces);

        assert_eq!(summary.effect_candidate_count, 1);
    }

    #[test]
    fn summary_counts_routes() {
        let traces = vec![
            recorded_trace(1),
            recorded_trace(2),
            blocked_missing_route_trace(3),
        ];

        let summary = summarize_interaction_semantic_action_dispatches(&traces);

        assert!(summary
            .routes
            .contains(&InteractionSemanticActionDispatchRouteCount {
                route: InteractionSemanticActionDispatchRouteKind::LocalUiStateCandidate,
                count: 2,
            }));

        assert!(summary
            .routes
            .contains(&InteractionSemanticActionDispatchRouteCount {
                route: InteractionSemanticActionDispatchRouteKind::Unknown,
                count: 1,
            }));
    }

    #[test]
    fn summary_counts_trace_reasons() {
        let traces = vec![recorded_trace(1), blocked_missing_route_trace(2)];

        let summary = summarize_interaction_semantic_action_dispatches(&traces);

        assert!(summary
            .reasons
            .contains(&InteractionSemanticActionDispatchTraceReasonCount {
                reason: InteractionSemanticActionDispatchTraceReason::RouteRecorded,
                count: 1,
            }));

        assert!(summary
            .reasons
            .contains(&InteractionSemanticActionDispatchTraceReasonCount {
                reason: InteractionSemanticActionDispatchTraceReason::MissingRoute,
                count: 1,
            }));
    }

    #[test]
    fn summary_preserves_first_appearance_order() {
        let traces = vec![
            effect_candidate_trace(1),
            recorded_trace(2),
            blocked_missing_route_trace(3),
        ];

        let summary = summarize_interaction_semantic_action_dispatches(&traces);

        assert_eq!(
            summary.routes[0].route,
            InteractionSemanticActionDispatchRouteKind::EffectRequestCandidate
        );
        assert_eq!(
            summary.routes[1].route,
            InteractionSemanticActionDispatchRouteKind::LocalUiStateCandidate
        );
        assert_eq!(
            summary.routes[2].route,
            InteractionSemanticActionDispatchRouteKind::Unknown
        );
    }

    #[test]
    fn dispatch_summary_is_not_authority() {
        let traces = vec![recorded_trace(1)];

        let summary = summarize_interaction_semantic_action_dispatches(&traces);

        assert!(!summary.is_dispatcher());
        assert!(!summary.is_execution_authority());
        assert!(!summary.is_effect_request());
        assert!(!summary.is_runtime_mutation());
        assert!(!summary.is_audit_authority());
    }

    #[test]
    fn dispatch_summary_generation_is_deterministic() {
        let traces = vec![
            recorded_trace(1),
            blocked_missing_route_trace(2),
            effect_candidate_trace(3),
        ];

        let first = summarize_interaction_semantic_action_dispatches(&traces);
        let second = summarize_interaction_semantic_action_dispatches(&traces);

        assert_eq!(first, second);
    }
}
