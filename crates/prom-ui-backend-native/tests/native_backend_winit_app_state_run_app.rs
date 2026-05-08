#![cfg(feature = "winit-backend")]

use prom_ui_backend_native::{
    winit_placeholder::{
        native_backend_winit_app_state_run_app_available,
        run_native_backend_winit_app_state_until_close,
        NativeBackendWinitAppStateRunError,
        NativeBackendWinitAppStateSummary,
    },
    NativeBackend,
};
use prom_ui_runtime::{UiBackendAdapter, WindowConfig};

#[test]
fn native_backend_winit_app_state_run_app_is_available() {
    assert!(prom_ui_backend_native::winit_backend_feature_enabled());
    assert!(native_backend_winit_app_state_run_app_available());
}

#[test]
fn native_backend_winit_app_state_run_app_requires_staged_config() {
    let backend = NativeBackend::new();

    let err = run_native_backend_winit_app_state_until_close(backend)
        .expect_err("run_app helper must require staged WindowConfig");

    assert!(matches!(
        err,
        NativeBackendWinitAppStateRunError::MissingWindowConfig
    ));
}

#[test]
fn native_backend_winit_app_state_run_app_has_expected_function_shape() {
    let _f: fn(
        NativeBackend,
    ) -> Result<
        NativeBackendWinitAppStateSummary,
        NativeBackendWinitAppStateRunError,
    > = run_native_backend_winit_app_state_until_close;
}

#[test]
fn native_backend_can_prepare_config_for_app_state_run_app() {
    let mut backend = NativeBackend::new();
    let config = WindowConfig::new("Semantic AppState RunApp", 800, 600);

    backend.create_window(&config).unwrap();

    let staged = backend
        .window_config()
        .expect("WindowConfig must be staged");

    assert_eq!(staged.title, "Semantic AppState RunApp");
    assert_eq!(staged.width, 800);
    assert_eq!(staged.height, 600);
    assert!(!backend.is_platform_wired());
}

#[test]
#[ignore = "manual NativeBackendWinitAppState run_app smoke; opens a native window"]
fn manual_native_backend_winit_app_state_runs_until_window_close() {
    let mut backend = NativeBackend::new();
    let config = WindowConfig::new("Semantic Native AppState", 800, 600);

    backend.create_window(&config).unwrap();

    let summary = run_native_backend_winit_app_state_until_close(backend)
        .expect("manual NativeBackendWinitAppState run_app should complete");

    assert!(summary.resumed_calls >= 1);
    assert_eq!(summary.create_attempts, 1);
    assert_eq!(summary.create_failures, 0);
    assert!(summary.window_created);
    assert!(summary.close_requested);
    assert!(summary.window_event_calls >= 1);
    assert!(summary.staged_event_count >= 1);
}
