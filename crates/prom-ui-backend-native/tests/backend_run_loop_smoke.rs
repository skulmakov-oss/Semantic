#![cfg(feature = "winit-backend")]

use prom_ui_backend_native::NativeBackend;
use prom_ui_runtime::{LoopControl, UiBackendAdapter, UiRuntimeError, WindowConfig};

#[test]
fn native_backend_winit_run_event_loop_can_be_invoked() {
    let mut backend = NativeBackend::new();
    let config = WindowConfig {
        title: "Test Window".into(),
        width: 800,
        height: 600,
    };
    backend.create_window(&config).unwrap();

    // Since winit's `run_app` blocks until the window is closed by the user,
    // we cannot execute the loop directly in a headless automated test without it hanging.
    // We simply assert that `NativeBackend` implements the trait method as expected.
    #[allow(clippy::type_complexity)]
    let _run_fn: fn(
        &mut NativeBackend,
        fn(&[prom_ui_runtime::InputEvent], LoopControl, &mut prom_ui_runtime::DrawFrame),
    ) -> Result<(), UiRuntimeError> = NativeBackend::run_event_loop;
}
