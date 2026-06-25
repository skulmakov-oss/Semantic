use prom_ui_backend_native::NativeBackend;
use prom_ui_runtime::{DesktopSession, DrawFrame, UiBackendAdapter, WindowConfig};

#[test]
fn native_backend_skeleton_is_not_platform_wired() {
    let backend = NativeBackend::new();

    assert!(!backend.is_platform_wired());
}

#[test]
fn native_backend_create_window_stages_config_without_platform_window() {
    let backend = NativeBackend::new();
    let cfg = WindowConfig::new("NativeSkeleton", 640, 480);

    let session = DesktopSession::create(backend, cfg).unwrap();

    let stored = session
        .backend()
        .window_config()
        .expect("window config must be staged");

    assert_eq!(stored.title, "NativeSkeleton");
    assert_eq!(stored.width, 640);
    assert_eq!(stored.height, 480);
    assert!(!session.backend().is_platform_wired());
    assert!(!session.backend().is_closed());
}

#[test]
fn native_backend_run_event_loop_is_staged_not_platform_wired() {
    let mut backend = NativeBackend::new();
    let mut controls = Vec::new();

    backend
        .run_event_loop(|control, _frame| controls.push(control))
        .unwrap();

    assert_eq!(backend.run_loop_calls(), 1);
    assert_eq!(backend.run_loop_ticks(), 0);
    assert!(controls.is_empty());
    assert!(!backend.is_platform_wired());
}

#[test]
fn native_backend_draw_frame_is_staged_not_rendered() {
    let mut backend = NativeBackend::new();
    let frame = DrawFrame::new();

    backend.draw_frame(&frame).unwrap();

    assert_eq!(backend.submitted_frames(), 1);
    assert!(!backend.is_platform_wired());
}

#[test]
fn native_backend_close_window_marks_closed_before_platform_wiring() {
    let mut backend = NativeBackend::new();

    backend.close_window();

    assert!(backend.is_closed());
    assert!(!backend.is_platform_wired());
}
