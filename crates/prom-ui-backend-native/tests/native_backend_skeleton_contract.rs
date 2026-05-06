use prom_ui::UiOperationId;
use prom_ui_backend_native::NativeBackend;
use prom_ui_runtime::{
    DesktopSession, DrawFrame, UiBackendAdapter, UiRuntimeError, WindowConfig,
};

#[test]
fn native_backend_skeleton_is_not_platform_wired() {
    let backend = NativeBackend::new();

    assert!(!backend.is_platform_wired());
}

#[test]
fn native_backend_create_window_fails_closed() {
    let backend = NativeBackend::new();
    let cfg = WindowConfig::new("NativeSkeleton", 640, 480);

    let err = match DesktopSession::create(backend, cfg) {
        Ok(_) => panic!("native skeleton must not create a real session yet"),
        Err(err) => err,
    };

    assert_eq!(
        err,
        UiRuntimeError::OperationNotAdmitted(UiOperationId::WindowCreate)
    );
}

#[test]
fn native_backend_run_event_loop_fails_closed() {
    let mut backend = NativeBackend::new();

    let err = backend
        .run_event_loop(|_| {})
        .expect_err("unwired native backend must reject run_event_loop");

    assert_eq!(
        err,
        UiRuntimeError::OperationNotAdmitted(UiOperationId::WindowRun)
    );
}

#[test]
fn native_backend_draw_frame_fails_closed() {
    let mut backend = NativeBackend::new();
    let frame = DrawFrame::new();

    let err = backend
        .draw_frame(&frame)
        .expect_err("unwired native backend must reject draw_frame");

    assert_eq!(
        err,
        UiRuntimeError::OperationNotAdmitted(UiOperationId::FrameSubmit)
    );
}

#[test]
fn native_backend_close_window_is_noop_before_platform_wiring() {
    let mut backend = NativeBackend::new();

    backend.close_window();

    assert!(!backend.is_platform_wired());
}
