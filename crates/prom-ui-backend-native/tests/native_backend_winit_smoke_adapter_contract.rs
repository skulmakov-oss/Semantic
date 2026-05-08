#![cfg(feature = "winit-backend")]

use prom_ui_backend_native::{NativeBackend, NativeBackendWinitSmokeError};
use prom_ui_runtime::{UiBackendAdapter, WindowConfig};

#[test]
fn native_backend_winit_smoke_adapter_is_available() {
    assert!(prom_ui_backend_native::winit_backend_feature_enabled());
    assert!(NativeBackend::winit_smoke_adapter_available());
}

#[test]
fn native_backend_winit_smoke_requires_staged_window_config() {
    let backend = NativeBackend::new();

    let err = backend
        .run_winit_smoke_from_staged_config()
        .expect_err("smoke adapter must require staged WindowConfig");

    assert!(matches!(
        err,
        NativeBackendWinitSmokeError::MissingWindowConfig
    ));

    assert!(!backend.is_platform_wired());
}

#[test]
fn native_backend_can_stage_config_for_winit_smoke_adapter() {
    let mut backend = NativeBackend::new();
    let config = WindowConfig::new("NativeBackend Smoke", 640, 480);

    backend.create_window(&config).unwrap();

    let staged = backend
        .window_config()
        .expect("WindowConfig must be staged");

    assert_eq!(staged.title, "NativeBackend Smoke");
    assert_eq!(staged.width, 640);
    assert_eq!(staged.height, 480);
    assert!(!backend.is_platform_wired());
}

#[test]
fn native_backend_winit_smoke_adapter_has_expected_method_shape() {
    let _f: fn(
        &NativeBackend,
    ) -> Result<
        prom_ui_backend_native::winit_placeholder::WinitRunAppSmokeResult,
        NativeBackendWinitSmokeError,
    > = NativeBackend::run_winit_smoke_from_staged_config;
}

#[test]
#[ignore = "manual NativeBackend winit smoke; may require desktop session"]
fn manual_native_backend_winit_smoke_from_staged_config() {
    let mut backend = NativeBackend::new();
    let config = WindowConfig::new("Semantic NativeBackend Smoke", 640, 480);

    backend.create_window(&config).unwrap();

    let result = backend
        .run_winit_smoke_from_staged_config()
        .expect("manual NativeBackend winit smoke should complete");

    assert!(result.resumed_calls >= 1);
    assert_eq!(result.create_attempts, 1);
    assert_eq!(result.create_failures, 0);
    assert!(result.window_created);

    // G13 is still a manual smoke adapter, not persistent platform wiring.
    assert!(!backend.is_platform_wired());
}
