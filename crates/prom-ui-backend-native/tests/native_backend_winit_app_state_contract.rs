#![cfg(feature = "winit-backend")]

use prom_ui_backend_native::{
    winit_placeholder::{
        native_backend_winit_app_state_available, NativeBackendWinitAppState,
        NativeBackendWinitAppStateSummary,
    },
    NativeBackend,
};
use prom_ui_runtime::{UiBackendAdapter, WindowConfig};

#[test]
fn native_backend_winit_app_state_is_available() {
    assert!(prom_ui_backend_native::winit_backend_feature_enabled());
    assert!(native_backend_winit_app_state_available());
}

#[test]
fn native_backend_winit_app_state_starts_empty() {
    let backend = NativeBackend::new();
    let app = NativeBackendWinitAppState::new(backend);

    assert!(!app.has_window());
    assert!(app.window_id().is_none());
    assert_eq!(app.resumed_calls(), 0);
    assert_eq!(app.window_event_calls(), 0);
    assert_eq!(app.create_attempts(), 0);
    assert_eq!(app.create_failures(), 0);
    assert!(!app.window_created());
    assert!(!app.close_requested());
    assert_eq!(app.backend().pending_event_count(), 0);
}

#[test]
fn native_backend_winit_app_state_preserves_staged_config() {
    let mut backend = NativeBackend::new();
    let config = WindowConfig::new("Semantic App State", 800, 600);

    backend.create_window(&config).unwrap();

    let app = NativeBackendWinitAppState::new(backend);

    let staged = app
        .backend()
        .window_config()
        .expect("staged WindowConfig must remain available");

    assert_eq!(staged.title, "Semantic App State");
    assert_eq!(staged.width, 800);
    assert_eq!(staged.height, 600);
    assert!(!app.backend().is_platform_wired());
}

#[test]
fn native_backend_winit_app_state_summary_starts_empty() {
    let backend = NativeBackend::new();
    let app = NativeBackendWinitAppState::new(backend);

    assert_eq!(
        app.summary(),
        NativeBackendWinitAppStateSummary {
            resumed_calls: 0,
            window_event_calls: 0,
            create_attempts: 0,
            create_failures: 0,
            window_created: false,
            close_requested: false,
            staged_event_count: 0,
        }
    );
}

#[test]
fn native_backend_winit_app_state_implements_application_handler() {
    fn assert_handler<T: winit::application::ApplicationHandler>() {}

    assert_handler::<NativeBackendWinitAppState>();
}

#[test]
fn native_backend_winit_app_state_backend_mut_can_stage_events() {
    let backend = NativeBackend::new();
    let mut app = NativeBackendWinitAppState::new(backend);

    app.backend_mut().stage_winit_close_requested();

    assert_eq!(app.backend().pending_event_count(), 1);
    assert_eq!(app.summary().staged_event_count, 1);
}
