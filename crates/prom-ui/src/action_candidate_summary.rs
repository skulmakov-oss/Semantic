//! Semantic UI interaction action candidate summary scaffold.
//!
//! This module summarizes action binding trace streams for diagnostics and
//! future UI consumption. It does not dispatch actions, perform admission,
//! request effects, call the VM, call Host ABI, or mutate runtime state.

use alloc::vec::Vec;

use crate::action_binding::{
    InteractionActionAdmissionPolicy, InteractionActionEffectPolicy, InteractionActionName,
};
use crate::action_binding_stream::InteractionActionBindingTraceStreamReport;
use crate::action_binding_trace::{
    InteractionActionBindingTraceReason, InteractionActionBindingTraceStatus,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InteractionActionCandidateCount {
    pub action: InteractionActionName,
    pub count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InteractionActionBindingReasonCount {
    pub reason: InteractionActionBindingTraceReason,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionActionCandidateSummary {
    pub total: usize,
    pub bound: usize,
    pub unbound: usize,
    pub actions: Vec<InteractionActionCandidateCount>,
    pub reasons: Vec<InteractionActionBindingReasonCount>,
    pub all_bound_require_admission: bool,
    pub effect_capable_candidates: usize,
}

pub fn summarize_interaction_action_candidates(
    stream: &InteractionActionBindingTraceStreamReport,
) -> InteractionActionCandidateSummary {
    let mut summary = InteractionActionCandidateSummary {
        total: stream.reports.len(),
        bound: 0,
        unbound: 0,
        actions: Vec::new(),
        reasons: Vec::new(),
        all_bound_require_admission: true,
        effect_capable_candidates: 0,
    };

    for report in &stream.reports {
        match report.binding_status {
            InteractionActionBindingTraceStatus::Bound => {
                summary.bound += 1;

                if report.admission_policy != Some(InteractionActionAdmissionPolicy::Required) {
                    summary.all_bound_require_admission = false;
                }

                if is_effect_capable(report.effect_policy) {
                    summary.effect_capable_candidates += 1;
                }

                if let Some(action) = report.action {
                    increment_action_count(&mut summary.actions, action);
                }
            }
            InteractionActionBindingTraceStatus::Unbound => {
                summary.unbound += 1;
            }
        }

        increment_reason_count(&mut summary.reasons, report.reason);
    }

    summary
}

fn increment_action_count(
    counts: &mut Vec<InteractionActionCandidateCount>,
    action: InteractionActionName,
) {
    if let Some(entry) = counts.iter_mut().find(|entry| entry.action == action) {
        entry.count += 1;
    } else {
        counts.push(InteractionActionCandidateCount { action, count: 1 });
    }
}

fn increment_reason_count(
    counts: &mut Vec<InteractionActionBindingReasonCount>,
    reason: InteractionActionBindingTraceReason,
) {
    if let Some(entry) = counts.iter_mut().find(|entry| entry.reason == reason) {
        entry.count += 1;
    } else {
        counts.push(InteractionActionBindingReasonCount { reason, count: 1 });
    }
}

fn is_effect_capable(policy: Option<InteractionActionEffectPolicy>) -> bool {
    matches!(
        policy,
        Some(InteractionActionEffectPolicy::MayRequestEffect)
            | Some(InteractionActionEffectPolicy::RequiresSeparateEffectAdmission)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action_binding::InteractionActionName;
    use crate::action_binding_stream::{
        trace_interaction_action_binding_stream, InteractionActionBindingTraceStreamModel,
        InteractionActionBindingTraceStreamReport, InteractionActionBindingTraceStreamStats,
    };
    use crate::action_binding_trace::InteractionActionBindingTraceReason;
    use crate::interaction::{
        ElementId, InteractionIntentId, InteractionIntentKind, InteractionSource, InteractionTarget,
    };
    use crate::raw_event::{RawUiEventId, RawUiEventKind};
    use crate::trace::{
        InteractionIntentMappingRule, InteractionIntentTraceReason, InteractionIntentTraceReport,
        InteractionIntentTraceStatus,
    };

    fn sample_stream() -> InteractionActionBindingTraceStreamReport {
        let model = InteractionActionBindingTraceStreamModel::new(vec![
            close_trace(1),
            select_trace(2),
            activate_trace(3),
            unknown_trace(4),
        ]);

        trace_interaction_action_binding_stream(&model)
    }

    fn close_trace(id: u64) -> InteractionIntentTraceReport {
        InteractionIntentTraceReport {
            raw_event_id: RawUiEventId(id),
            intent_id: InteractionIntentId(id),
            raw_kind: RawUiEventKind::WindowCloseRequested,
            intent_kind: InteractionIntentKind::Close,
            source: InteractionSource::Window,
            target: None,
            status: InteractionIntentTraceStatus::Classified,
            reason: InteractionIntentTraceReason::WindowMapping,
            rule: InteractionIntentMappingRule::WindowClose,
        }
    }

    fn select_trace(id: u64) -> InteractionIntentTraceReport {
        InteractionIntentTraceReport {
            raw_event_id: RawUiEventId(id),
            intent_id: InteractionIntentId(id),
            raw_kind: RawUiEventKind::PointerDown,
            intent_kind: InteractionIntentKind::Select,
            source: InteractionSource::Pointer,
            target: Some(InteractionTarget::Element(ElementId(7))),
            status: InteractionIntentTraceStatus::Classified,
            reason: InteractionIntentTraceReason::DirectMapping,
            rule: InteractionIntentMappingRule::PointerDownActivate,
        }
    }

    fn activate_trace(id: u64) -> InteractionIntentTraceReport {
        InteractionIntentTraceReport {
            raw_event_id: RawUiEventId(id),
            intent_id: InteractionIntentId(id),
            raw_kind: RawUiEventKind::PointerDown,
            intent_kind: InteractionIntentKind::Activate,
            source: InteractionSource::Pointer,
            target: Some(InteractionTarget::Element(ElementId(1))),
            status: InteractionIntentTraceStatus::Classified,
            reason: InteractionIntentTraceReason::DirectMapping,
            rule: InteractionIntentMappingRule::PointerDownActivate,
        }
    }

    fn unknown_trace(id: u64) -> InteractionIntentTraceReport {
        InteractionIntentTraceReport {
            raw_event_id: RawUiEventId(id),
            intent_id: InteractionIntentId(id),
            raw_kind: RawUiEventKind::Unknown,
            intent_kind: InteractionIntentKind::Unknown,
            source: InteractionSource::Unknown,
            target: None,
            status: InteractionIntentTraceStatus::Unclassified,
            reason: InteractionIntentTraceReason::UnknownRawEvent,
            rule: InteractionIntentMappingRule::Unknown,
        }
    }

    #[test]
    fn empty_stream_summary_is_zeroed() {
        let stream = InteractionActionBindingTraceStreamReport {
            reports: Vec::new(),
            stats: InteractionActionBindingTraceStreamStats::default(),
        };

        let summary = summarize_interaction_action_candidates(&stream);

        assert_eq!(summary.total, 0);
        assert_eq!(summary.bound, 0);
        assert_eq!(summary.unbound, 0);
        assert!(summary.actions.is_empty());
        assert!(summary.reasons.is_empty());
        assert!(summary.all_bound_require_admission);
        assert_eq!(summary.effect_capable_candidates, 0);
    }

    #[test]
    fn summary_counts_bound_and_unbound() {
        let stream = sample_stream();

        let summary = summarize_interaction_action_candidates(&stream);

        assert_eq!(summary.total, 4);
        assert_eq!(summary.bound, 2);
        assert_eq!(summary.unbound, 2);
    }

    #[test]
    fn summary_counts_action_candidates() {
        let stream = sample_stream();

        let summary = summarize_interaction_action_candidates(&stream);

        assert!(summary.actions.contains(&InteractionActionCandidateCount {
            action: InteractionActionName::CloseWindow,
            count: 1,
        }));

        assert!(summary.actions.contains(&InteractionActionCandidateCount {
            action: InteractionActionName::SelectTraceEvent,
            count: 1,
        }));
    }

    #[test]
    fn summary_counts_binding_reasons() {
        let stream = sample_stream();

        let summary = summarize_interaction_action_candidates(&stream);

        assert!(summary
            .reasons
            .contains(&InteractionActionBindingReasonCount {
                reason: InteractionActionBindingTraceReason::BindingIgnoresTarget,
                count: 1,
            }));

        assert!(summary
            .reasons
            .contains(&InteractionActionBindingReasonCount {
                reason: InteractionActionBindingTraceReason::BindingRequiresTarget,
                count: 1,
            }));

        assert!(summary
            .reasons
            .contains(&InteractionActionBindingReasonCount {
                reason: InteractionActionBindingTraceReason::NoBindingForIntent,
                count: 1,
            }));

        assert!(summary
            .reasons
            .contains(&InteractionActionBindingReasonCount {
                reason: InteractionActionBindingTraceReason::IntentUnclassified,
                count: 1,
            }));
    }

    #[test]
    fn summary_preserves_admission_invariant() {
        let stream = sample_stream();

        let summary = summarize_interaction_action_candidates(&stream);

        assert!(summary.all_bound_require_admission);
    }

    #[test]
    fn summary_effect_capable_count_is_zero_for_initial_bindings() {
        let stream = sample_stream();

        let summary = summarize_interaction_action_candidates(&stream);

        assert_eq!(summary.effect_capable_candidates, 0);
    }

    #[test]
    fn summary_is_deterministic_for_same_stream() {
        let stream = sample_stream();

        let first = summarize_interaction_action_candidates(&stream);
        let second = summarize_interaction_action_candidates(&stream);

        assert_eq!(first, second);
    }

    #[test]
    fn summary_preserves_first_appearance_order_for_counts() {
        let model = InteractionActionBindingTraceStreamModel::new(vec![
            select_trace(1),
            close_trace(2),
            select_trace(3),
            unknown_trace(4),
        ]);

        let stream = trace_interaction_action_binding_stream(&model);
        let summary = summarize_interaction_action_candidates(&stream);

        assert_eq!(
            summary.actions[0].action,
            InteractionActionName::SelectTraceEvent
        );
        assert_eq!(
            summary.actions[1].action,
            InteractionActionName::CloseWindow
        );
        assert_eq!(
            summary.reasons[0].reason,
            InteractionActionBindingTraceReason::BindingRequiresTarget
        );
    }
}
