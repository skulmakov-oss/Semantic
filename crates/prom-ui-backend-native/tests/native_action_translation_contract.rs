#![cfg_attr(not(feature = "std"), no_std)]

use prom_ui::{InteractionIntentKind, UiProjectedNodeId};
use prom_ui_backend_native::{
    DefaultNativeActionTranslator, NativeActionTranslator, RawBackendEvent, RawButtonState,
    RawKeyCode, RoutedInteraction,
};

#[test]
fn translates_keyboard_pressed_to_activate() {
    let translator = DefaultNativeActionTranslator::new();
    let target = UiProjectedNodeId::new(1);
    let event = RawBackendEvent::KeyboardInput {
        key: RawKeyCode::Enter,
        state: RawButtonState::Pressed,
    };
    let interaction = RoutedInteraction::new(target, event);

    assert_eq!(
        translator.translate(&interaction),
        InteractionIntentKind::Activate
    );
}

#[test]
fn translates_pointer_moved_to_unknown() {
    let translator = DefaultNativeActionTranslator::new();
    let target = UiProjectedNodeId::new(1);
    let event = RawBackendEvent::PointerMoved { x: 10.0, y: 20.0 };
    let interaction = RoutedInteraction::new(target, event);

    assert_eq!(
        translator.translate(&interaction),
        InteractionIntentKind::Unknown
    );
}
