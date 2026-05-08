#![cfg(feature = "winit-backend")]

use prom_ui_backend_native::winit_placeholder::{
    create_winit_event_loop, winit_event_loop_creation_available,
};

#[test]
fn winit_event_loop_creation_scaffold_is_available() {
    assert!(prom_ui_backend_native::winit_backend_feature_enabled());
    assert!(winit_event_loop_creation_available());
}

#[test]
fn can_call_winit_event_loop_creation_helper_without_panicking() {
    let _ = create_winit_event_loop();
}
