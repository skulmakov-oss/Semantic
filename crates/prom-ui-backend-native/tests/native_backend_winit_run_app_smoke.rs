#![cfg(feature = "winit-backend")]

use prom_ui_backend_native::winit_placeholder::{
    run_winit_window_creation_smoke, winit_run_app_smoke_available, WinitRunAppSmokeResult,
    WinitRunAppSmokeScaffold,
};
use prom_ui_runtime::WindowConfig;

#[test]
fn winit_run_app_smoke_scaffold_is_available() {
    assert!(prom_ui_backend_native::winit_backend_feature_enabled());
    assert!(winit_run_app_smoke_available());
}

#[test]
fn run_app_smoke_scaffold_starts_empty() {
    let config = WindowConfig::new("RunApp Smoke", 640, 480);
    let scaffold = WinitRunAppSmokeScaffold::new(config);

    let result = scaffold.result();

    assert_eq!(
        result,
        WinitRunAppSmokeResult {
            resumed_calls: 0,
            create_attempts: 0,
            create_failures: 0,
            window_created: false,
        }
    );
    assert!(!scaffold.has_window());
}

#[test]
fn run_app_smoke_scaffold_implements_application_handler() {
    fn assert_handler<T: winit::application::ApplicationHandler>() {}

    assert_handler::<WinitRunAppSmokeScaffold>();
}

#[test]
fn run_winit_window_creation_smoke_has_expected_function_shape() {
    let _f: fn(WindowConfig) -> Result<WinitRunAppSmokeResult, winit::error::EventLoopError> =
        run_winit_window_creation_smoke;
}

#[test]
#[ignore = "manual native window smoke; may require desktop session"]
fn manual_run_app_creates_window_and_exits() {
    let config = WindowConfig::new("Semantic Native Smoke", 640, 480);

    let result = run_winit_window_creation_smoke(config)
        .expect("manual native run_app smoke should complete");

    assert!(result.resumed_calls >= 1);
    assert_eq!(result.create_attempts, 1);
    assert_eq!(result.create_failures, 0);
    assert!(result.window_created);
}
