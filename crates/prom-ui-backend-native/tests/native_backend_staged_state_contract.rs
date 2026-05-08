use prom_ui_backend_native::NativeBackend;
use prom_ui_runtime::{
    DesktopSession, DrawFrame, InputEvent, InputEventKind, UiBackendAdapter, WindowConfig,
};

#[test]
fn native_backend_starts_with_empty_staged_state() {
    let backend = NativeBackend::new();

    assert!(!backend.is_platform_wired());
    assert!(backend.window_config().is_none());
    assert_eq!(backend.pending_event_count(), 0);
    assert!(backend.pending_events().is_empty());
    assert_eq!(backend.submitted_frames(), 0);
    assert!(!backend.is_closed());
}

#[test]
fn create_window_stages_window_config_without_platform_wiring() {
    let mut backend = NativeBackend::new();
    let config = WindowConfig::new("Native Staged", 800, 600);

    backend.create_window(&config).unwrap();

    let stored = backend
        .window_config()
        .expect("window config must be staged");

    assert_eq!(stored.title, "Native Staged");
    assert_eq!(stored.width, 800);
    assert_eq!(stored.height, 600);
    assert!(!backend.is_platform_wired());
    assert!(!backend.is_closed());
}

#[test]
fn desktop_session_create_works_with_staged_native_backend() {
    let backend = NativeBackend::new();
    let config = WindowConfig::new("Session Native", 1024, 768);

    let session = DesktopSession::create(backend, config).unwrap();

    let stored = session
        .backend()
        .window_config()
        .expect("window config must be staged through DesktopSession");

    assert_eq!(stored.title, "Session Native");
    assert_eq!(stored.width, 1024);
    assert_eq!(stored.height, 768);
    assert!(!session.backend().is_platform_wired());
}

#[test]
fn pending_events_preserve_order_and_payload() {
    let mut backend = NativeBackend::new();

    backend.extend_pending_events([
        InputEvent::new(InputEventKind::KeyDown { key_code: 65 }),
        InputEvent::new(InputEventKind::KeyUp { key_code: 65 }),
        InputEvent::new(InputEventKind::CloseRequested),
    ]);

    assert_eq!(backend.pending_event_count(), 3);
    assert_eq!(
        backend.pending_events()[0].kind,
        InputEventKind::KeyDown { key_code: 65 }
    );
    assert_eq!(
        backend.pending_events()[1].kind,
        InputEventKind::KeyUp { key_code: 65 }
    );
    assert_eq!(
        backend.pending_events()[2].kind,
        InputEventKind::CloseRequested
    );
}

#[test]
fn drain_pending_events_returns_and_clears_queue() {
    let mut backend = NativeBackend::new();

    backend.push_pending_event(InputEvent::new(InputEventKind::KeyDown { key_code: 1 }));
    backend.push_pending_event(InputEvent::new(InputEventKind::CloseRequested));

    let drained = backend.drain_pending_events();

    assert_eq!(drained.len(), 2);
    assert_eq!(drained[0].kind, InputEventKind::KeyDown { key_code: 1 });
    assert_eq!(drained[1].kind, InputEventKind::CloseRequested);
    assert_eq!(backend.pending_event_count(), 0);
    assert!(backend.pending_events().is_empty());
}

#[test]
fn draw_frame_counts_submitted_frames_without_rendering() {
    let mut backend = NativeBackend::new();
    let frame = DrawFrame::new();

    backend.draw_frame(&frame).unwrap();
    backend.draw_frame(&frame).unwrap();

    assert_eq!(backend.submitted_frames(), 2);
    assert!(!backend.is_platform_wired());
}

#[test]
fn run_event_loop_is_staged_until_winit_wiring() {
    let mut backend = NativeBackend::new();
    let mut controls = Vec::new();

    backend
        .run_event_loop(|control| controls.push(control))
        .unwrap();

    assert_eq!(backend.run_loop_calls(), 1);
    assert_eq!(backend.run_loop_ticks(), 0);
    assert!(controls.is_empty());
    assert!(!backend.is_platform_wired());
}

#[test]
fn close_window_marks_backend_closed_without_platform_wiring() {
    let mut backend = NativeBackend::new();

    backend.close_window();

    assert!(backend.is_closed());
    assert!(!backend.is_platform_wired());
}
