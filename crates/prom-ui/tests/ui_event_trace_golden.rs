use prom_ui::interaction::{InteractionIntentKind, InteractionSource};
use prom_ui::intent_stream::InteractionIntentStreamModel;
use prom_ui::raw_event::{
    map_raw_event_to_interaction_intent, RawPointerButton, RawUiEvent, RawUiEventId,
    RawUiEventKind, RawUiEventPayload, RawUiEventTarget,
};
use prom_ui::interaction::InteractionModifiers;

#[test]
fn ui_event_trace_golden_stream() {
    let mut stream_model = InteractionIntentStreamModel {
        events: vec![],
    };

    // 1. PointerMove -> Hover
    stream_model.events.push(RawUiEvent::new(
        RawUiEventId(1),
        RawUiEventKind::PointerMove,
        RawUiEventTarget::Unknown,
        InteractionModifiers::default(),
        RawUiEventPayload::CursorMove { x: 100.0, y: 100.0 },
    ));

    // 2. PointerDown -> Activate
    stream_model.events.push(RawUiEvent::new(
        RawUiEventId(2),
        RawUiEventKind::PointerDown,
        RawUiEventTarget::Unknown,
        InteractionModifiers::default(),
        RawUiEventPayload::Pointer { button: RawPointerButton::Primary },
    ));

    // 3. PointerUp -> Unknown (for now)
    stream_model.events.push(RawUiEvent::new(
        RawUiEventId(3),
        RawUiEventKind::PointerUp,
        RawUiEventTarget::Unknown,
        InteractionModifiers::default(),
        RawUiEventPayload::Pointer { button: RawPointerButton::Primary },
    ));

    // Map each raw event to an interaction intent descriptor
    let intents: Vec<_> = stream_model
        .events
        .iter()
        .map(map_raw_event_to_interaction_intent)
        .collect();

    assert_eq!(intents.len(), 3);

    // Verify 1: PointerMove -> Hover
    assert_eq!(intents[0].source, InteractionSource::Pointer);
    assert_eq!(intents[0].kind, InteractionIntentKind::Hover);

    // Verify 2: PointerDown -> Activate
    assert_eq!(intents[1].source, InteractionSource::Pointer);
    assert_eq!(intents[1].kind, InteractionIntentKind::Activate);

    // Verify 3: PointerUp -> Unknown
    assert_eq!(intents[2].source, InteractionSource::Pointer);
    assert_eq!(intents[2].kind, InteractionIntentKind::Unknown);
}
