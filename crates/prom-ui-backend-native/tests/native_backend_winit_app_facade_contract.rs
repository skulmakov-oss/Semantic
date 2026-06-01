#![cfg(feature = "winit-backend")]

use prom_ui_backend_native::{
    winit_placeholder::{
        native_backend_winit_app_facade_available, NativeBackendWinitApp,
        NativeBackendWinitAppError, NativeBackendWinitAppStateSummary,
    },
    NativeBackend,
};
use prom_ui_runtime::{UiBackendAdapter, WindowConfig};

#[test]
fn native_backend_winit_app_facade_is_available() {
    assert!(prom_ui_backend_native::winit_backend_feature_enabled());
    assert!(native_backend_winit_app_facade_available());
}

#[test]
fn native_backend_winit_app_facade_requires_staged_window_config() {
    let backend = NativeBackend::new();

    let err =
        NativeBackendWinitApp::new(backend).expect_err("facade must require staged WindowConfig");

    assert!(matches!(
        err,
        NativeBackendWinitAppError::MissingWindowConfig
    ));
}

#[test]
fn native_backend_winit_app_facade_accepts_staged_window_config() {
    let mut backend = NativeBackend::new();
    let config = WindowConfig::new("Facade", 800, 600);

    backend.create_window(&config).unwrap();

    let facade =
        NativeBackendWinitApp::new(backend).expect("facade must accept staged WindowConfig");

    let staged = facade
        .backend()
        .window_config()
        .expect("WindowConfig must remain staged");

    assert_eq!(staged.title, "Facade");
    assert_eq!(staged.width, 800);
    assert_eq!(staged.height, 600);
    assert!(!facade.backend().is_platform_wired());
}

#[test]
fn native_backend_winit_app_facade_can_return_backend_before_run() {
    let mut backend = NativeBackend::new();
    let config = WindowConfig::new("Return Backend", 640, 480);

    backend.create_window(&config).unwrap();

    let facade = NativeBackendWinitApp::new(backend).unwrap();
    let backend = facade.into_backend();

    let staged = backend
        .window_config()
        .expect("WindowConfig must survive facade roundtrip");

    assert_eq!(staged.title, "Return Backend");
    assert_eq!(staged.width, 640);
    assert_eq!(staged.height, 480);
    assert!(!backend.is_platform_wired());
}

#[test]
fn native_backend_winit_app_facade_run_until_close_has_expected_shape() {
    let _f: fn(
        NativeBackendWinitApp,
    ) -> Result<NativeBackendWinitAppStateSummary, NativeBackendWinitAppError> =
        NativeBackendWinitApp::run_until_close;
}

#[test]
#[ignore = "manual NativeBackendWinitApp facade smoke; opens a native window"]
fn manual_native_backend_winit_app_facade_runs_until_close() {
    let mut backend = NativeBackend::new();
    let config = WindowConfig::new("Semantic Native App Facade", 800, 600);

    backend.create_window(&config).unwrap();

    let facade = NativeBackendWinitApp::new(backend).unwrap();

    let summary = facade
        .run_until_close()
        .expect("manual NativeBackendWinitApp facade run should complete");

    assert!(summary.resumed_calls >= 1);
    assert_eq!(summary.create_attempts, 1);
    assert_eq!(summary.create_failures, 0);
    assert!(summary.window_created);
    assert!(summary.close_requested);
    assert!(summary.window_event_calls >= 1);
    assert!(summary.staged_event_count >= 1);
}
