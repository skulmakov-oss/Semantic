#![cfg(feature = "winit-backend")]
#![allow(unused_imports)]

use prom_ui_backend_native::winit_placeholder::translate_winit_to_raw_backend_event;
use prom_ui_backend_native::{RawBackendEvent, RawButtonState, RawKeyCode};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

#[test]
fn close_requested_translates_to_raw_close_requested() {
    let event = translate_winit_to_raw_backend_event(&WindowEvent::CloseRequested)
        .expect("CloseRequested must translate");

    assert_eq!(event, RawBackendEvent::CloseRequested);
}

#[test]
fn resized_translates_to_raw_window_resized() {
    let size = PhysicalSize::new(800, 600);
    let event = translate_winit_to_raw_backend_event(&WindowEvent::Resized(size))
        .expect("Resized must translate");

    assert_eq!(
        event,
        RawBackendEvent::WindowResized {
            width: 800,
            height: 600
        }
    );
}

#[test]
#[ignore = "winit 0.30 KeyEvent cannot be constructed due to private platform_specific field"]
fn pressed_physical_key_translates_to_raw_keyboard_input_pressed() {
    /*
    let winit_event = WindowEvent::KeyboardInput {
        device_id: winit::event::DeviceId::dummy(),
        event: KeyEvent {
            physical_key: PhysicalKey::Code(KeyCode::KeyA),
            logical_key: winit::keyboard::Key::Unidentified(
                winit::keyboard::NativeKey::Unidentified,
            ),
            text: None,
            location: winit::keyboard::KeyLocation::Standard,
            state: ElementState::Pressed,
            repeat: false,
            ..unsafe { std::mem::zeroed() }
        },
        is_synthetic: false,
    };

    let event = translate_winit_to_raw_backend_event(&winit_event).expect("KeyA must translate");

    assert_eq!(
        event,
        RawBackendEvent::KeyboardInput {
            key: RawKeyCode::KeyA,
            state: RawButtonState::Pressed,
        }
    );
    */
}

#[test]
#[ignore = "winit 0.30 KeyEvent cannot be constructed due to private platform_specific field"]
fn released_physical_key_translates_to_raw_keyboard_input_released() {
    /*
    let winit_event = WindowEvent::KeyboardInput {
        device_id: winit::event::DeviceId::dummy(),
        event: KeyEvent {
            physical_key: PhysicalKey::Code(KeyCode::Space),
            logical_key: winit::keyboard::Key::Unidentified(
                winit::keyboard::NativeKey::Unidentified,
            ),
            text: None,
            location: winit::keyboard::KeyLocation::Standard,
            state: ElementState::Released,
            repeat: false,
            ..unsafe { std::mem::zeroed() }
        },
        is_synthetic: false,
    };

    let event = translate_winit_to_raw_backend_event(&winit_event).expect("Space must translate");

    assert_eq!(
        event,
        RawBackendEvent::KeyboardInput {
            key: RawKeyCode::Space,
            state: RawButtonState::Released,
        }
    );
    */
}

#[test]
fn cursor_moved_translates_to_raw_pointer_moved() {
    let position = PhysicalPosition::new(100.0, 200.0);
    let winit_event = WindowEvent::CursorMoved {
        device_id: winit::event::DeviceId::dummy(),
        position,
    };

    let event =
        translate_winit_to_raw_backend_event(&winit_event).expect("CursorMoved must translate");

    assert_eq!(event, RawBackendEvent::PointerMoved { x: 100.0, y: 200.0 });
}

#[test]
#[ignore = "winit 0.30 KeyEvent cannot be constructed due to private platform_specific field"]
fn unidentified_physical_key_returns_none() {
    /*
    let winit_event = WindowEvent::KeyboardInput {
        device_id: winit::event::DeviceId::dummy(),
        event: KeyEvent {
            physical_key: PhysicalKey::Unidentified(winit::keyboard::NativeKeyCode::Unidentified),
            logical_key: winit::keyboard::Key::Unidentified(
                winit::keyboard::NativeKey::Unidentified,
            ),
            text: None,
            location: winit::keyboard::KeyLocation::Standard,
            state: ElementState::Pressed,
            repeat: false,
            ..unsafe { std::mem::zeroed() }
        },
        is_synthetic: false,
    };

    let event = translate_winit_to_raw_backend_event(&winit_event);

    assert!(event.is_none());
    */
}
