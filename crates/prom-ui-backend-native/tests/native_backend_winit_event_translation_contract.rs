#![cfg(feature = "winit-backend")]

use prom_ui_backend_native::winit_placeholder::{
    translate_winit_close_requested, translate_winit_key_code, translate_winit_physical_key,
    translate_winit_window_event, winit_event_translation_available,
};
use prom_ui_runtime::InputEventKind;
use winit::{
    event::{ElementState, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

#[test]
fn winit_event_translation_scaffold_is_available() {
    assert!(prom_ui_backend_native::winit_backend_feature_enabled());
    assert!(winit_event_translation_available());
}

#[test]
fn close_requested_translates_to_close_requested_input_event() {
    let event = translate_winit_close_requested();

    assert_eq!(event.kind, InputEventKind::CloseRequested);
}

#[test]
fn key_code_translation_maps_selected_keys() {
    assert_eq!(translate_winit_key_code(KeyCode::KeyA), Some(65));
    assert_eq!(translate_winit_key_code(KeyCode::KeyW), Some(87));
    assert_eq!(translate_winit_key_code(KeyCode::Digit1), Some(49));
    assert_eq!(translate_winit_key_code(KeyCode::Enter), Some(13));
    assert_eq!(translate_winit_key_code(KeyCode::Escape), Some(27));
    assert_eq!(translate_winit_key_code(KeyCode::Space), Some(32));
}

#[test]
fn pressed_physical_key_translates_to_key_down() {
    let event =
        translate_winit_physical_key(ElementState::Pressed, PhysicalKey::Code(KeyCode::KeyA))
            .expect("KeyA must be supported");

    assert_eq!(event.kind, InputEventKind::KeyDown { key_code: 65 });
}

#[test]
fn released_physical_key_translates_to_key_up() {
    let event =
        translate_winit_physical_key(ElementState::Released, PhysicalKey::Code(KeyCode::KeyA))
            .expect("KeyA must be supported");

    assert_eq!(event.kind, InputEventKind::KeyUp { key_code: 65 });
}

#[test]
fn unsupported_physical_key_returns_none() {
    let event =
        translate_winit_physical_key(ElementState::Pressed, PhysicalKey::Code(KeyCode::F12));

    assert!(event.is_none());
}

#[test]
fn window_event_close_requested_translates() {
    let event = translate_winit_window_event(&WindowEvent::CloseRequested, 1.0)
        .expect("CloseRequested must translate");

    assert_eq!(event.kind, InputEventKind::CloseRequested);
}
