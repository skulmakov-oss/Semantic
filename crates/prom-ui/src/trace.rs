//! Semantic UI interaction intent trace report scaffold.
//!
//! This module explains how raw input becomes an interaction intent.
//! It does not dispatch actions, call the VM, call Host ABI, or mutate
//! runtime state.

use crate::interaction::{
    InteractionIntentDescriptor, InteractionIntentId, InteractionIntentKind, InteractionSource,
    InteractionTarget,
};
use crate::raw_event::{
    map_raw_event_to_interaction_intent, RawKeyCode, RawUiEvent, RawUiEventId, RawUiEventKind,
    RawUiEventPayload,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionIntentTraceStatus {
    Classified,
    Unclassified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionIntentTraceReason {
    DirectMapping,
    KeyMapping,
    TextInputMapping,
    WindowMapping,
    FocusMapping,
    UnknownRawEvent,
    UnknownSource,
    UnknownTarget,
    UnsupportedKey,
    PayloadMismatch,
    DeliberatelyUnmapped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionIntentMappingRule {
    PointerDownActivate,
    PointerEnterFocus,
    PointerLeaveBlur,
    KeyEnterSubmit,
    KeyEscapeCancel,
    KeySpaceActivate,
    KeyNavigation,
    TextInputEdit,
    WheelScroll,
    WindowClose,
    WindowResize,
    FocusGained,
    FocusLost,
    Unknown,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InteractionIntentTraceReport {
    pub raw_event_id: RawUiEventId,
    pub intent_id: InteractionIntentId,
    pub raw_kind: RawUiEventKind,
    pub intent_kind: InteractionIntentKind,
    pub source: InteractionSource,
    pub target: Option<InteractionTarget>,
    pub status: InteractionIntentTraceStatus,
    pub reason: InteractionIntentTraceReason,
    pub rule: InteractionIntentMappingRule,
}

pub fn trace_raw_event_to_interaction_intent(event: &RawUiEvent) -> InteractionIntentTraceReport {
    let intent = map_raw_event_to_interaction_intent(event);
    let status = if intent.is_classified() {
        InteractionIntentTraceStatus::Classified
    } else {
        InteractionIntentTraceStatus::Unclassified
    };
    let (reason, rule) = classify_trace(event, &intent);

    InteractionIntentTraceReport {
        raw_event_id: event.id,
        intent_id: intent.id,
        raw_kind: event.kind,
        intent_kind: intent.kind,
        source: intent.source,
        target: intent.target,
        status,
        reason,
        rule,
    }
}

fn classify_trace(
    event: &RawUiEvent,
    intent: &InteractionIntentDescriptor,
) -> (InteractionIntentTraceReason, InteractionIntentMappingRule) {
    if matches!(event.kind, RawUiEventKind::Unknown) {
        return (
            InteractionIntentTraceReason::UnknownRawEvent,
            InteractionIntentMappingRule::Unknown,
        );
    }

    if matches!(intent.source, InteractionSource::Unknown) {
        return (
            InteractionIntentTraceReason::UnknownSource,
            InteractionIntentMappingRule::Unknown,
        );
    }

    match event.kind {
        RawUiEventKind::PointerDown => classify_direct_mapping(
            intent.target,
            InteractionIntentTraceReason::DirectMapping,
            InteractionIntentMappingRule::PointerDownActivate,
        ),
        RawUiEventKind::PointerEnter => classify_direct_mapping(
            intent.target,
            InteractionIntentTraceReason::FocusMapping,
            InteractionIntentMappingRule::PointerEnterFocus,
        ),
        RawUiEventKind::PointerLeave => classify_direct_mapping(
            intent.target,
            InteractionIntentTraceReason::FocusMapping,
            InteractionIntentMappingRule::PointerLeaveBlur,
        ),
        RawUiEventKind::PointerUp | RawUiEventKind::PointerMove | RawUiEventKind::KeyUp => (
            InteractionIntentTraceReason::DeliberatelyUnmapped,
            InteractionIntentMappingRule::None,
        ),
        RawUiEventKind::KeyDown => classify_key_trace(event, intent.target),
        RawUiEventKind::TextInput => classify_direct_mapping(
            intent.target,
            InteractionIntentTraceReason::TextInputMapping,
            InteractionIntentMappingRule::TextInputEdit,
        ),
        RawUiEventKind::Wheel => classify_direct_mapping(
            intent.target,
            InteractionIntentTraceReason::DirectMapping,
            InteractionIntentMappingRule::WheelScroll,
        ),
        RawUiEventKind::WindowCloseRequested => classify_direct_mapping(
            intent.target,
            InteractionIntentTraceReason::WindowMapping,
            InteractionIntentMappingRule::WindowClose,
        ),
        RawUiEventKind::WindowResized => classify_direct_mapping(
            intent.target,
            InteractionIntentTraceReason::WindowMapping,
            InteractionIntentMappingRule::WindowResize,
        ),
        RawUiEventKind::FocusGained => classify_direct_mapping(
            intent.target,
            InteractionIntentTraceReason::FocusMapping,
            InteractionIntentMappingRule::FocusGained,
        ),
        RawUiEventKind::FocusLost => classify_direct_mapping(
            intent.target,
            InteractionIntentTraceReason::FocusMapping,
            InteractionIntentMappingRule::FocusLost,
        ),
        RawUiEventKind::Unknown => (
            InteractionIntentTraceReason::UnknownRawEvent,
            InteractionIntentMappingRule::Unknown,
        ),
    }
}

fn classify_direct_mapping(
    target: Option<InteractionTarget>,
    mapped_reason: InteractionIntentTraceReason,
    rule: InteractionIntentMappingRule,
) -> (InteractionIntentTraceReason, InteractionIntentMappingRule) {
    if target.is_none() {
        (InteractionIntentTraceReason::UnknownTarget, rule)
    } else {
        (mapped_reason, rule)
    }
}

fn classify_key_trace(
    event: &RawUiEvent,
    target: Option<InteractionTarget>,
) -> (InteractionIntentTraceReason, InteractionIntentMappingRule) {
    match &event.payload {
        RawUiEventPayload::Key {
            key: RawKeyCode::Enter,
        } => classify_direct_mapping(
            target,
            InteractionIntentTraceReason::KeyMapping,
            InteractionIntentMappingRule::KeyEnterSubmit,
        ),
        RawUiEventPayload::Key {
            key: RawKeyCode::Escape,
        } => classify_direct_mapping(
            target,
            InteractionIntentTraceReason::KeyMapping,
            InteractionIntentMappingRule::KeyEscapeCancel,
        ),
        RawUiEventPayload::Key {
            key: RawKeyCode::Space,
        } => classify_direct_mapping(
            target,
            InteractionIntentTraceReason::KeyMapping,
            InteractionIntentMappingRule::KeySpaceActivate,
        ),
        RawUiEventPayload::Key {
            key:
                RawKeyCode::ArrowUp
                | RawKeyCode::ArrowDown
                | RawKeyCode::ArrowLeft
                | RawKeyCode::ArrowRight
                | RawKeyCode::Tab,
        } => classify_direct_mapping(
            target,
            InteractionIntentTraceReason::KeyMapping,
            InteractionIntentMappingRule::KeyNavigation,
        ),
        RawUiEventPayload::Key { .. } => (
            InteractionIntentTraceReason::UnsupportedKey,
            InteractionIntentMappingRule::None,
        ),
        _ => (
            InteractionIntentTraceReason::PayloadMismatch,
            InteractionIntentMappingRule::None,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interaction::{ElementId, InteractionModifiers};
    use crate::raw_event::{
        RawPointerButton, RawUiEvent, RawUiEventId, RawUiEventKind, RawUiEventPayload,
        RawUiEventTarget,
    };

    #[test]
    fn trace_pointer_down_activate() {
        let event = RawUiEvent::new(
            RawUiEventId(1),
            RawUiEventKind::PointerDown,
            RawUiEventTarget::Element(ElementId(7)),
            InteractionModifiers::default(),
            RawUiEventPayload::Pointer {
                button: RawPointerButton::Primary,
            },
        );

        let report = trace_raw_event_to_interaction_intent(&event);

        assert_eq!(report.status, InteractionIntentTraceStatus::Classified);
        assert_eq!(report.intent_kind, InteractionIntentKind::Activate);
        assert_eq!(report.reason, InteractionIntentTraceReason::DirectMapping);
        assert_eq!(
            report.rule,
            InteractionIntentMappingRule::PointerDownActivate
        );
    }

    #[test]
    fn trace_enter_submit() {
        let event = RawUiEvent::new(
            RawUiEventId(2),
            RawUiEventKind::KeyDown,
            RawUiEventTarget::Element(ElementId(1)),
            InteractionModifiers::default(),
            RawUiEventPayload::Key {
                key: RawKeyCode::Enter,
            },
        );

        let report = trace_raw_event_to_interaction_intent(&event);

        assert_eq!(report.status, InteractionIntentTraceStatus::Classified);
        assert_eq!(report.intent_kind, InteractionIntentKind::Submit);
        assert_eq!(report.reason, InteractionIntentTraceReason::KeyMapping);
        assert_eq!(report.rule, InteractionIntentMappingRule::KeyEnterSubmit);
    }

    #[test]
    fn trace_unsupported_key_is_unclassified() {
        let event = RawUiEvent::new(
            RawUiEventId(3),
            RawUiEventKind::KeyDown,
            RawUiEventTarget::Element(ElementId(1)),
            InteractionModifiers::default(),
            RawUiEventPayload::Key {
                key: RawKeyCode::Character('x'),
            },
        );

        let report = trace_raw_event_to_interaction_intent(&event);

        assert_eq!(report.status, InteractionIntentTraceStatus::Unclassified);
        assert_eq!(report.intent_kind, InteractionIntentKind::Unknown);
        assert_eq!(report.reason, InteractionIntentTraceReason::UnsupportedKey);
        assert_eq!(report.rule, InteractionIntentMappingRule::None);
    }

    #[test]
    fn trace_key_down_payload_mismatch() {
        let event = RawUiEvent::new(
            RawUiEventId(4),
            RawUiEventKind::KeyDown,
            RawUiEventTarget::Element(ElementId(1)),
            InteractionModifiers::default(),
            RawUiEventPayload::None,
        );

        let report = trace_raw_event_to_interaction_intent(&event);

        assert_eq!(report.status, InteractionIntentTraceStatus::Unclassified);
        assert_eq!(report.reason, InteractionIntentTraceReason::PayloadMismatch);
        assert_eq!(report.rule, InteractionIntentMappingRule::None);
    }

    #[test]
    fn trace_unknown_raw_event() {
        let event = RawUiEvent::new(
            RawUiEventId(5),
            RawUiEventKind::Unknown,
            RawUiEventTarget::Unknown,
            InteractionModifiers::default(),
            RawUiEventPayload::None,
        );

        let report = trace_raw_event_to_interaction_intent(&event);

        assert_eq!(report.status, InteractionIntentTraceStatus::Unclassified);
        assert_eq!(report.reason, InteractionIntentTraceReason::UnknownRawEvent);
        assert_eq!(report.rule, InteractionIntentMappingRule::Unknown);
        assert_eq!(report.target, None);
    }

    #[test]
    fn trace_pointer_move_is_deliberately_unmapped() {
        let event = RawUiEvent::new(
            RawUiEventId(6),
            RawUiEventKind::PointerMove,
            RawUiEventTarget::Element(ElementId(8)),
            InteractionModifiers::default(),
            RawUiEventPayload::Pointer {
                button: RawPointerButton::Unknown,
            },
        );

        let report = trace_raw_event_to_interaction_intent(&event);

        assert_eq!(report.status, InteractionIntentTraceStatus::Unclassified);
        assert_eq!(
            report.reason,
            InteractionIntentTraceReason::DeliberatelyUnmapped
        );
        assert_eq!(report.rule, InteractionIntentMappingRule::None);
    }

    #[test]
    fn trace_unknown_target_is_reported_without_blocking_classification() {
        let event = RawUiEvent::new(
            RawUiEventId(7),
            RawUiEventKind::PointerDown,
            RawUiEventTarget::Unknown,
            InteractionModifiers::default(),
            RawUiEventPayload::Pointer {
                button: RawPointerButton::Primary,
            },
        );

        let report = trace_raw_event_to_interaction_intent(&event);

        assert_eq!(report.status, InteractionIntentTraceStatus::Classified);
        assert_eq!(report.reason, InteractionIntentTraceReason::UnknownTarget);
        assert_eq!(
            report.rule,
            InteractionIntentMappingRule::PointerDownActivate
        );
    }
}
