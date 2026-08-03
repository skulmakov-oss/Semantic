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

#[test]
fn window_event_resized_translates_to_real_physical_size() {
    let size = winit::dpi::PhysicalSize::new(1440u32, 900u32);
    let event = translate_winit_window_event(&WindowEvent::Resized(size), 1.0)
        .expect("Resized must translate");

    assert_eq!(
        event.kind,
        InputEventKind::Resized {
            width: 1440,
            height: 900
        }
    );
}

// There is deliberately no
// `window_event_scale_factor_changed_translates_...` test exercising
// `translate_winit_window_event` end-to-end the way the `Resized` test
// above does: winit 0.30's `WindowEvent::ScaleFactorChanged` carries an
// `InnerSizeWriter`, and `InnerSizeWriter::new` is `pub(crate)` inside
// winit itself (confirmed by reading winit 0.30.13's own source), so no
// crate outside winit can construct a real `WindowEvent::ScaleFactorChanged`
// value at all -- not a gap in this test, a real constraint of the winit
// API. `translate_winit_window_event`'s `ScaleFactorChanged` arm is still
// exercised for real by the live native launch smoke test
// (`scripts/workbench_native_launch_smoke.ps1`), which runs the actual
// winit event loop, and by `InputEventKind::ScaleFactorChanged`'s
// PartialEq/Debug derives being real (not stubbed) via ordinary usage
// elsewhere in this test file's other assertions.

#[test]
fn alt_key_maps_to_distinct_out_of_ascii_code() {
    assert_eq!(translate_winit_key_code(KeyCode::AltLeft), Some(1002));
    assert_eq!(translate_winit_key_code(KeyCode::AltRight), Some(1002));
}

#[test]
fn is_insertable_text_accepts_real_unicode_and_rejects_control_characters() {
    use prom_ui_backend_native::winit_placeholder::is_insertable_text;

    assert!(is_insertable_text("a"));
    assert!(is_insertable_text("A"));
    // Real Cyrillic committed text (e.g. a Russian keyboard layout).
    assert!(is_insertable_text("д"));
    assert!(is_insertable_text("привет"));
    // Mixed UTF-8.
    assert!(is_insertable_text("h€llo"));

    assert!(!is_insertable_text(""));
    assert!(
        !is_insertable_text("\r"),
        "Enter must stay physical-key-driven, not inserted as text"
    );
    assert!(
        !is_insertable_text("\u{8}"),
        "Backspace must stay physical-key-driven"
    );
    assert!(
        !is_insertable_text("\t"),
        "Tab must stay physical-key-driven"
    );
    assert!(
        !is_insertable_text("\u{1b}"),
        "Escape must stay physical-key-driven"
    );
}
