#![cfg(feature = "winit-backend")]

use prom_ui_backend_native::winit_placeholder::{
    create_winit_window_from_config, winit_window_creation_scaffold_available,
    WinitWindowCreationScaffold,
};
use prom_ui_runtime::WindowConfig;

#[test]
fn winit_window_creation_scaffold_is_available() {
    assert!(prom_ui_backend_native::winit_backend_feature_enabled());
    assert!(winit_window_creation_scaffold_available());
}

#[test]
fn window_creation_scaffold_starts_without_window() {
    let config = WindowConfig::new("Native Window Scaffold", 800, 600);
    let scaffold = WinitWindowCreationScaffold::new(config);

    assert!(!scaffold.has_window());
    assert!(scaffold.window_id().is_none());
    assert_eq!(scaffold.create_attempts(), 0);
    assert_eq!(scaffold.create_failures(), 0);
    assert_eq!(scaffold.config().title, "Native Window Scaffold");
    assert_eq!(scaffold.config().width, 800);
    assert_eq!(scaffold.config().height, 600);
}

#[test]
fn window_creation_scaffold_implements_application_handler() {
    fn assert_handler<T: winit::application::ApplicationHandler>() {}

    assert_handler::<WinitWindowCreationScaffold>();
}

#[test]
fn create_winit_window_from_config_has_expected_function_shape() {
    let _f: fn(
        &winit::event_loop::ActiveEventLoop,
        &WindowConfig,
    ) -> Result<winit::window::Window, winit::error::OsError> = create_winit_window_from_config;
}
