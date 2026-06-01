//! Semantic UI deterministic action binding trace stream scaffold.
//!
//! This module batches action binding trace reports in stable order.
//! It does not implement a dispatcher, admission engine, runtime queue,
//! event loop, or any side effects.

use alloc::vec::Vec;

use crate::action_binding_trace::{
    trace_interaction_action_binding, InteractionActionBindingTraceReport,
    InteractionActionBindingTraceStatus,
};
use crate::trace::InteractionIntentTraceReport;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionActionBindingTraceStreamModel {
    pub intent_traces: Vec<InteractionIntentTraceReport>,
}

impl InteractionActionBindingTraceStreamModel {
    pub fn new(intent_traces: Vec<InteractionIntentTraceReport>) -> Self {
        Self { intent_traces }
    }

    pub fn is_empty(&self) -> bool {
        self.intent_traces.is_empty()
    }

    pub fn len(&self) -> usize {
        self.intent_traces.len()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct InteractionActionBindingTraceStreamStats {
    pub total: usize,
    pub bound: usize,
    pub unbound: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionActionBindingTraceStreamReport {
    pub reports: Vec<InteractionActionBindingTraceReport>,
    pub stats: InteractionActionBindingTraceStreamStats,
}

impl InteractionActionBindingTraceStreamReport {
    pub fn is_empty(&self) -> bool {
        self.reports.is_empty()
    }

    pub fn len(&self) -> usize {
        self.reports.len()
    }
}

pub fn trace_interaction_action_binding_stream(
    model: &InteractionActionBindingTraceStreamModel,
) -> InteractionActionBindingTraceStreamReport {
    let mut reports = Vec::with_capacity(model.intent_traces.len());
    let mut stats = InteractionActionBindingTraceStreamStats {
        total: model.intent_traces.len(),
        bound: 0,
        unbound: 0,
    };

    for intent_trace in &model.intent_traces {
        let report = trace_interaction_action_binding(intent_trace);

        match report.binding_status {
            InteractionActionBindingTraceStatus::Bound => {
                stats.bound += 1;
            }
            InteractionActionBindingTraceStatus::Unbound => {
                stats.unbound += 1;
            }
        }

        reports.push(report);
    }

    InteractionActionBindingTraceStreamReport { reports, stats }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action_binding::InteractionActionName;
    use crate::action_binding_trace::InteractionActionBindingTraceStatus;
    use crate::interaction::{
        ElementId, InteractionIntentId, InteractionIntentKind, InteractionSource, InteractionTarget,
    };
    use crate::raw_event::{RawUiEventId, RawUiEventKind};
    use crate::trace::{
        InteractionIntentMappingRule, InteractionIntentTraceReason, InteractionIntentTraceReport,
        InteractionIntentTraceStatus,
    };

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
    fn empty_stream_produces_empty_report() {
        let model = InteractionActionBindingTraceStreamModel::new(Vec::new());

        let report = trace_interaction_action_binding_stream(&model);

        assert!(model.is_empty());
        assert!(report.is_empty());
        assert_eq!(report.stats.total, 0);
        assert_eq!(report.stats.bound, 0);
        assert_eq!(report.stats.unbound, 0);
    }

    #[test]
    fn stream_preserves_input_order() {
        let model = InteractionActionBindingTraceStreamModel::new(vec![
            close_trace(1),
            activate_trace(2),
            unknown_trace(3),
        ]);

        let report = trace_interaction_action_binding_stream(&model);

        let ids: Vec<u64> = report.reports.iter().map(|r| r.intent_id.0).collect();

        assert_eq!(ids, vec![1, 2, 3]);
    }

    #[test]
    fn stream_trace_is_deterministic_for_same_input() {
        let model = InteractionActionBindingTraceStreamModel::new(vec![
            close_trace(1),
            select_trace(2),
            activate_trace(3),
            unknown_trace(4),
        ]);

        let first = trace_interaction_action_binding_stream(&model);
        let second = trace_interaction_action_binding_stream(&model);

        assert_eq!(first, second);
    }

    #[test]
    fn stream_stats_count_bound_and_unbound_reports() {
        let model = InteractionActionBindingTraceStreamModel::new(vec![
            close_trace(1),
            select_trace(2),
            activate_trace(3),
            unknown_trace(4),
        ]);

        let report = trace_interaction_action_binding_stream(&model);

        assert_eq!(report.stats.total, 4);
        assert_eq!(report.stats.bound, 2);
        assert_eq!(report.stats.unbound, 2);
    }

    #[test]
    fn stream_does_not_filter_unbound_reports() {
        let model = InteractionActionBindingTraceStreamModel::new(vec![
            activate_trace(1),
            unknown_trace(2),
        ]);

        let report = trace_interaction_action_binding_stream(&model);

        assert_eq!(report.len(), 2);
        assert_eq!(report.reports[0].intent_id.0, 1);
        assert_eq!(report.reports[1].intent_id.0, 2);
    }

    #[test]
    fn stream_reports_bound_and_unbound_counts_consistently() {
        let model = InteractionActionBindingTraceStreamModel::new(vec![
            close_trace(1),
            select_trace(2),
            activate_trace(3),
            unknown_trace(4),
        ]);

        let report = trace_interaction_action_binding_stream(&model);

        assert_eq!(
            report.reports[0].action,
            Some(InteractionActionName::CloseWindow)
        );
        assert_eq!(
            report.reports[2].binding_status,
            InteractionActionBindingTraceStatus::Unbound
        );
    }
}
