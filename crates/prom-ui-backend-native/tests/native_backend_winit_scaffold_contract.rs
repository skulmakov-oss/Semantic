#[cfg(not(feature = "winit-backend"))]
#[test]
fn winit_event_loop_scaffold_is_disabled_by_default() {
    assert!(!prom_ui_backend_native::winit_backend_feature_enabled());
}

#[cfg(feature = "winit-backend")]
use prom_ui_backend_native::winit_placeholder::{
    winit_event_loop_scaffold_available, WinitEventLoopScaffold,
};

#[cfg(feature = "winit-backend")]
#[test]
fn winit_event_loop_scaffold_is_available_under_feature() {
    assert!(prom_ui_backend_native::winit_backend_feature_enabled());
    assert!(winit_event_loop_scaffold_available());
}

#[cfg(feature = "winit-backend")]
#[test]
fn winit_event_loop_scaffold_starts_empty() {
    let scaffold = WinitEventLoopScaffold::new();

    assert_eq!(scaffold.resumed_calls(), 0);
    assert_eq!(scaffold.window_event_calls(), 0);
    assert!(!scaffold.close_requested());
}

#[cfg(feature = "winit-backend")]
#[test]
fn winit_event_loop_scaffold_implements_application_handler() {
    fn assert_handler<T: winit::application::ApplicationHandler>() {}

    assert_handler::<WinitEventLoopScaffold>();
}
