//! Semantic UI deterministic interaction intent stream test model.
//!
//! This module batches raw UI events into trace reports in stable order.
//! It does not implement an event loop, async stream, dispatcher, runtime
//! queue, or any side effects.

use alloc::vec::Vec;

use crate::raw_event::RawUiEvent;
use crate::trace::{
    trace_raw_event_to_interaction_intent, InteractionIntentTraceReport,
    InteractionIntentTraceStatus,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionIntentStreamModel {
    pub events: Vec<RawUiEvent>,
}

impl InteractionIntentStreamModel {
    pub fn new(events: Vec<RawUiEvent>) -> Self {
        Self { events }
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct InteractionIntentStreamStats {
    pub total: usize,
    pub classified: usize,
    pub unclassified: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionIntentStreamReport {
    pub reports: Vec<InteractionIntentTraceReport>,
    pub stats: InteractionIntentStreamStats,
}

impl InteractionIntentStreamReport {
    pub fn is_empty(&self) -> bool {
        self.reports.is_empty()
    }

    pub fn len(&self) -> usize {
        self.reports.len()
    }
}

pub fn trace_interaction_intent_stream(
    model: &InteractionIntentStreamModel,
) -> InteractionIntentStreamReport {
    let mut reports = Vec::with_capacity(model.events.len());
    let mut stats = InteractionIntentStreamStats {
        total: model.events.len(),
        classified: 0,
        unclassified: 0,
    };

    for event in &model.events {
        let report = trace_raw_event_to_interaction_intent(event);

        match report.status {
            InteractionIntentTraceStatus::Classified => {
                stats.classified += 1;
            }
            InteractionIntentTraceStatus::Unclassified => {
                stats.unclassified += 1;
            }
        }

        reports.push(report);
    }

    InteractionIntentStreamReport { reports, stats }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interaction::{ElementId, InteractionModifiers};
    use crate::raw_event::{
        RawKeyCode, RawPointerButton, RawUiEvent, RawUiEventId, RawUiEventKind,
        RawUiEventPayload, RawUiEventTarget,
    };

    fn pointer_down(id: u64) -> RawUiEvent {
        RawUiEvent::new(
            RawUiEventId(id),
            RawUiEventKind::PointerDown,
            RawUiEventTarget::Element(ElementId(10)),
            InteractionModifiers::default(),
            RawUiEventPayload::Pointer {
                button: RawPointerButton::Primary,
            },
        )
    }

    fn enter_key(id: u64) -> RawUiEvent {
        RawUiEvent::new(
            RawUiEventId(id),
            RawUiEventKind::KeyDown,
            RawUiEventTarget::Element(ElementId(11)),
            InteractionModifiers::default(),
            RawUiEventPayload::Key {
                key: RawKeyCode::Enter,
            },
        )
    }

    fn unsupported_key(id: u64) -> RawUiEvent {
        RawUiEvent::new(
            RawUiEventId(id),
            RawUiEventKind::KeyDown,
            RawUiEventTarget::Element(ElementId(12)),
            InteractionModifiers::default(),
            RawUiEventPayload::Key {
                key: RawKeyCode::Character('x'),
            },
        )
    }

    fn unknown_event(id: u64) -> RawUiEvent {
        RawUiEvent::new(
            RawUiEventId(id),
            RawUiEventKind::Unknown,
            RawUiEventTarget::Unknown,
            InteractionModifiers::default(),
            RawUiEventPayload::None,
        )
    }

    #[test]
    fn empty_stream_produces_empty_report() {
        let model = InteractionIntentStreamModel::new(Vec::new());

        let report = trace_interaction_intent_stream(&model);

        assert!(model.is_empty());
        assert!(report.is_empty());
        assert_eq!(report.stats.total, 0);
        assert_eq!(report.stats.classified, 0);
        assert_eq!(report.stats.unclassified, 0);
    }

    #[test]
    fn stream_preserves_input_order() {
        let model = InteractionIntentStreamModel::new(vec![
            pointer_down(1),
            enter_key(2),
            unknown_event(3),
        ]);

        let report = trace_interaction_intent_stream(&model);

        let ids: Vec<u64> = report.reports.iter().map(|r| r.raw_event_id.0).collect();

        assert_eq!(ids, vec![1, 2, 3]);
    }

    #[test]
    fn stream_trace_is_deterministic_for_same_input() {
        let model = InteractionIntentStreamModel::new(vec![
            pointer_down(1),
            enter_key(2),
            unsupported_key(3),
            unknown_event(4),
        ]);

        let first = trace_interaction_intent_stream(&model);
        let second = trace_interaction_intent_stream(&model);

        assert_eq!(first, second);
    }

    #[test]
    fn stream_stats_count_classified_and_unclassified_reports() {
        let model = InteractionIntentStreamModel::new(vec![
            pointer_down(1),
            enter_key(2),
            unsupported_key(3),
            unknown_event(4),
        ]);

        let report = trace_interaction_intent_stream(&model);

        assert_eq!(report.stats.total, 4);
        assert_eq!(report.stats.classified, 2);
        assert_eq!(report.stats.unclassified, 2);
    }

    #[test]
    fn stream_does_not_filter_unclassified_reports() {
        let model = InteractionIntentStreamModel::new(vec![
            unknown_event(1),
            unsupported_key(2),
        ]);

        let report = trace_interaction_intent_stream(&model);

        assert_eq!(report.len(), 2);
        assert_eq!(report.reports[0].raw_event_id.0, 1);
        assert_eq!(report.reports[1].raw_event_id.0, 2);
    }
}
