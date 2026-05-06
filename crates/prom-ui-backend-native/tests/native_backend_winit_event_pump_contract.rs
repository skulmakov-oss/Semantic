#![cfg(feature = "winit-backend")]

use prom_ui_backend_native::NativeBackend;
use prom_ui_runtime::InputEventKind;
use winit::{
    event::{ElementState, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

#[test]
fn staged_winit_close_request_enters_pending_queue() {
    let mut backend = NativeBackend::new();

    backend.stage_winit_close_requested();

    assert_eq!(backend.pending_event_count(), 1);
    assert_eq!(
        backend.pending_events()[0].kind,
        InputEventKind::CloseRequested
    );
}

#[test]
fn staged_window_event_close_requested_enters_pending_queue() {
    let mut backend = NativeBackend::new();

    let staged = backend.stage_winit_window_event(&WindowEvent::CloseRequested);

    assert!(staged);
    assert_eq!(backend.pending_event_count(), 1);
    assert_eq!(
        backend.pending_events()[0].kind,
        InputEventKind::CloseRequested
    );
}

#[test]
fn staged_pressed_physical_key_enters_pending_queue_as_key_down() {
    let mut backend = NativeBackend::new();

    let staged =
        backend.stage_winit_physical_key(ElementState::Pressed, PhysicalKey::Code(KeyCode::KeyA));

    assert!(staged);
    assert_eq!(backend.pending_event_count(), 1);
    assert_eq!(
        backend.pending_events()[0].kind,
        InputEventKind::KeyDown { key_code: 65 }
    );
}

#[test]
fn staged_released_physical_key_enters_pending_queue_as_key_up() {
    let mut backend = NativeBackend::new();

    let staged = backend
        .stage_winit_physical_key(ElementState::Released, PhysicalKey::Code(KeyCode::KeyA));

    assert!(staged);
    assert_eq!(backend.pending_event_count(), 1);
    assert_eq!(
        backend.pending_events()[0].kind,
        InputEventKind::KeyUp { key_code: 65 }
    );
}

#[test]
fn unsupported_winit_key_is_not_staged() {
    let mut backend = NativeBackend::new();

    let staged =
        backend.stage_winit_physical_key(ElementState::Pressed, PhysicalKey::Code(KeyCode::F12));

    assert!(!staged);
    assert_eq!(backend.pending_event_count(), 0);
    assert!(backend.pending_events().is_empty());
}

#[test]
fn multiple_staged_winit_events_preserve_order() {
    let mut backend = NativeBackend::new();

    assert!(backend.stage_winit_physical_key(
        ElementState::Pressed,
        PhysicalKey::Code(KeyCode::KeyA),
    ));

    assert!(backend.stage_winit_physical_key(
        ElementState::Released,
        PhysicalKey::Code(KeyCode::KeyA),
    ));

    backend.stage_winit_close_requested();

    assert_eq!(backend.pending_event_count(), 3);

    assert_eq!(
        backend.pending_events()[0].kind,
        InputEventKind::KeyDown { key_code: 65 }
    );
    assert_eq!(
        backend.pending_events()[1].kind,
        InputEventKind::KeyUp { key_code: 65 }
    );
    assert_eq!(
        backend.pending_events()[2].kind,
        InputEventKind::CloseRequested
    );
}

#[test]
fn staged_winit_events_can_be_drained() {
    let mut backend = NativeBackend::new();

    backend.stage_winit_physical_key(
        ElementState::Pressed,
        PhysicalKey::Code(KeyCode::KeyA),
    );
    backend.stage_winit_close_requested();

    let drained = backend.drain_pending_events();

    assert_eq!(drained.len(), 2);
    assert_eq!(drained[0].kind, InputEventKind::KeyDown { key_code: 65 });
    assert_eq!(drained[1].kind, InputEventKind::CloseRequested);

    assert_eq!(backend.pending_event_count(), 0);
}
