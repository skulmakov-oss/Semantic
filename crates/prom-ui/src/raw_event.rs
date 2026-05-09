//! Semantic UI raw input event scaffold.
//!
//! This module defines platform-neutral raw UI event descriptors and a pure
//! mapping into `InteractionIntentDescriptor`.
//! It does not implement an event loop, dispatcher, platform backend, VM call,
//! Host ABI call, capability check, or widget/layout framework.

use alloc::string::String;

use crate::interaction::{
    ElementId, InteractionIntentDescriptor, InteractionIntentId, InteractionIntentKind,
    InteractionModifiers, InteractionSource, InteractionTarget, RegionId, SurfaceId,
    WindowId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RawUiEventId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RawUiEventKind {
    PointerDown,
    PointerUp,
    PointerMove,
    PointerEnter,
    PointerLeave,
    KeyDown,
    KeyUp,
    TextInput,
    Wheel,
    WindowCloseRequested,
    WindowResized,
    FocusGained,
    FocusLost,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RawPointerButton {
    Primary,
    Secondary,
    Middle,
    Other(u16),
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RawKeyCode {
    Enter,
    Escape,
    Space,
    Tab,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Backspace,
    Delete,
    Character(char),
    Other(u32),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RawTextInput {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RawUiEventPayload {
    None,
    Pointer {
        button: RawPointerButton,
    },
    Key {
        key: RawKeyCode,
    },
    Text(RawTextInput),
    Wheel {
        delta_x: i32,
        delta_y: i32,
    },
    Resize {
        width: u32,
        height: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RawUiEventTarget {
    Window(WindowId),
    Surface(SurfaceId),
    Element(ElementId),
    Region(RegionId),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RawUiEvent {
    pub id: RawUiEventId,
    pub kind: RawUiEventKind,
    pub target: RawUiEventTarget,
    pub modifiers: InteractionModifiers,
    pub payload: RawUiEventPayload,
}

impl RawUiEvent {
    pub const fn new(
        id: RawUiEventId,
        kind: RawUiEventKind,
        target: RawUiEventTarget,
        modifiers: InteractionModifiers,
        payload: RawUiEventPayload,
    ) -> Self {
        Self {
            id,
            kind,
            target,
            modifiers,
            payload,
        }
    }
}

pub fn map_raw_event_to_interaction_intent(
    event: &RawUiEvent,
) -> InteractionIntentDescriptor {
    let source = map_source(event.kind);
    let kind = map_kind(event.kind, &event.payload);
    let target = map_target(event.target);

    InteractionIntentDescriptor::new(
        InteractionIntentId(event.id.0),
        source,
        kind,
        target,
        event.modifiers,
    )
}

const fn map_source(kind: RawUiEventKind) -> InteractionSource {
    match kind {
        RawUiEventKind::PointerDown
        | RawUiEventKind::PointerUp
        | RawUiEventKind::PointerMove
        | RawUiEventKind::PointerEnter
        | RawUiEventKind::PointerLeave => InteractionSource::Pointer,
        RawUiEventKind::KeyDown | RawUiEventKind::KeyUp => InteractionSource::Keyboard,
        RawUiEventKind::TextInput => InteractionSource::TextInput,
        RawUiEventKind::Wheel => InteractionSource::Wheel,
        RawUiEventKind::WindowCloseRequested
        | RawUiEventKind::WindowResized
        | RawUiEventKind::FocusGained
        | RawUiEventKind::FocusLost => InteractionSource::Window,
        RawUiEventKind::Unknown => InteractionSource::Unknown,
    }
}

fn map_kind(kind: RawUiEventKind, payload: &RawUiEventPayload) -> InteractionIntentKind {
    match kind {
        RawUiEventKind::PointerDown => InteractionIntentKind::Activate,
        RawUiEventKind::PointerEnter => InteractionIntentKind::Focus,
        RawUiEventKind::PointerLeave => InteractionIntentKind::Blur,
        RawUiEventKind::KeyDown => match payload {
            RawUiEventPayload::Key { key: RawKeyCode::Enter } => {
                InteractionIntentKind::Submit
            }
            RawUiEventPayload::Key { key: RawKeyCode::Escape } => {
                InteractionIntentKind::Cancel
            }
            RawUiEventPayload::Key { key: RawKeyCode::Space } => {
                InteractionIntentKind::Activate
            }
            RawUiEventPayload::Key {
                key:
                    RawKeyCode::ArrowUp
                    | RawKeyCode::ArrowDown
                    | RawKeyCode::ArrowLeft
                    | RawKeyCode::ArrowRight
                    | RawKeyCode::Tab,
            } => InteractionIntentKind::Navigate,
            _ => InteractionIntentKind::Unknown,
        },
        RawUiEventKind::TextInput => InteractionIntentKind::Edit,
        RawUiEventKind::Wheel => InteractionIntentKind::Scroll,
        RawUiEventKind::WindowCloseRequested => InteractionIntentKind::Close,
        RawUiEventKind::WindowResized => InteractionIntentKind::Resize,
        RawUiEventKind::FocusGained => InteractionIntentKind::Focus,
        RawUiEventKind::FocusLost => InteractionIntentKind::Blur,
        RawUiEventKind::PointerUp
        | RawUiEventKind::PointerMove
        | RawUiEventKind::KeyUp
        | RawUiEventKind::Unknown => InteractionIntentKind::Unknown,
    }
}

const fn map_target(target: RawUiEventTarget) -> Option<InteractionTarget> {
    match target {
        RawUiEventTarget::Window(id) => Some(InteractionTarget::Window(id)),
        RawUiEventTarget::Surface(id) => Some(InteractionTarget::Surface(id)),
        RawUiEventTarget::Element(id) => Some(InteractionTarget::Element(id)),
        RawUiEventTarget::Region(id) => Some(InteractionTarget::Region(id)),
        RawUiEventTarget::Unknown => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_down_maps_to_activate_intent() {
        let event = RawUiEvent::new(
            RawUiEventId(10),
            RawUiEventKind::PointerDown,
            RawUiEventTarget::Element(ElementId(7)),
            InteractionModifiers::default(),
            RawUiEventPayload::Pointer {
                button: RawPointerButton::Primary,
            },
        );

        let intent = map_raw_event_to_interaction_intent(&event);

        assert_eq!(intent.id, InteractionIntentId(10));
        assert_eq!(intent.source, InteractionSource::Pointer);
        assert_eq!(intent.kind, InteractionIntentKind::Activate);
        assert_eq!(intent.target, Some(InteractionTarget::Element(ElementId(7))));
        assert!(intent.is_classified());
    }

    #[test]
    fn enter_key_down_maps_to_submit_intent() {
        let event = RawUiEvent::new(
            RawUiEventId(11),
            RawUiEventKind::KeyDown,
            RawUiEventTarget::Element(ElementId(2)),
            InteractionModifiers::default(),
            RawUiEventPayload::Key {
                key: RawKeyCode::Enter,
            },
        );

        let intent = map_raw_event_to_interaction_intent(&event);

        assert_eq!(intent.source, InteractionSource::Keyboard);
        assert_eq!(intent.kind, InteractionIntentKind::Submit);
        assert!(intent.is_classified());
    }

    #[test]
    fn escape_key_down_maps_to_cancel_intent() {
        let event = RawUiEvent::new(
            RawUiEventId(12),
            RawUiEventKind::KeyDown,
            RawUiEventTarget::Element(ElementId(2)),
            InteractionModifiers::default(),
            RawUiEventPayload::Key {
                key: RawKeyCode::Escape,
            },
        );

        let intent = map_raw_event_to_interaction_intent(&event);

        assert_eq!(intent.kind, InteractionIntentKind::Cancel);
    }

    #[test]
    fn text_input_maps_to_edit_intent() {
        let event = RawUiEvent::new(
            RawUiEventId(13),
            RawUiEventKind::TextInput,
            RawUiEventTarget::Element(ElementId(3)),
            InteractionModifiers::default(),
            RawUiEventPayload::Text(RawTextInput {
                text: "abc".into(),
            }),
        );

        let intent = map_raw_event_to_interaction_intent(&event);

        assert_eq!(intent.source, InteractionSource::TextInput);
        assert_eq!(intent.kind, InteractionIntentKind::Edit);
    }

    #[test]
    fn unknown_event_maps_to_unknown_intent() {
        let event = RawUiEvent::new(
            RawUiEventId(14),
            RawUiEventKind::Unknown,
            RawUiEventTarget::Unknown,
            InteractionModifiers::default(),
            RawUiEventPayload::None,
        );

        let intent = map_raw_event_to_interaction_intent(&event);

        assert_eq!(intent.source, InteractionSource::Unknown);
        assert_eq!(intent.kind, InteractionIntentKind::Unknown);
        assert_eq!(intent.target, None);
        assert!(!intent.is_classified());
    }

    #[test]
    fn mapping_preserves_modifiers() {
        let modifiers = InteractionModifiers {
            shift: true,
            ctrl: true,
            alt: false,
            meta: false,
        };

        let event = RawUiEvent::new(
            RawUiEventId(15),
            RawUiEventKind::KeyDown,
            RawUiEventTarget::Element(ElementId(4)),
            modifiers,
            RawUiEventPayload::Key {
                key: RawKeyCode::Enter,
            },
        );

        let intent = map_raw_event_to_interaction_intent(&event);

        assert_eq!(intent.modifiers, modifiers);
    }
}
