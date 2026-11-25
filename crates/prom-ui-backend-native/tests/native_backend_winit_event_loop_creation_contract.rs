#![cfg(feature = "winit-backend")]

use prom_ui_backend_native::winit_placeholder::{
    create_winit_event_loop, winit_event_loop_creation_available,
};

#[test]
fn winit_event_loop_creation_scaffold_is_available() {
    assert!(prom_ui_backend_native::winit_backend_feature_enabled());
    assert!(winit_event_loop_creation_available());
}

/// Windows and X11 allow creating the event loop on any thread
/// (`with_any_thread`), which is what the libtest worker thread needs.
#[cfg(not(target_os = "macos"))]
#[test]
fn can_call_winit_event_loop_creation_helper_without_panicking() {
    let _ = create_winit_event_loop();
}

/// macOS (AppKit) only allows the event loop on the process main thread and
/// winit offers no any-thread mode there; libtest always runs tests on
/// worker threads. The helper's macOS path is therefore covered by asserting
/// the platform's documented refusal, deterministically and with its exact
/// diagnostic, instead of reporting an unconditional pass.
#[cfg(target_os = "macos")]
#[test]
fn macos_event_loop_creation_off_the_main_thread_is_refused_by_the_platform() {
    let outcome = std::panic::catch_unwind(create_winit_event_loop);
    let payload = outcome.expect_err("winit must refuse event-loop creation off the main thread");
    let message = payload
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_default();
    assert!(
        message.contains("must be created on the main thread"),
        "unexpected refusal: {message}"
    );
}
