use prom_ui_backend_native::NativeBackend;
use prom_ui_runtime::{
    DesktopSession, InputEvent, InputEventKind, LoopControl, SessionState, UiBackendAdapter,
    WindowConfig,
};

#[test]
fn run_loop_starts_with_zero_counters() {
    let backend = NativeBackend::new();

    assert_eq!(backend.run_loop_calls(), 0);
    assert_eq!(backend.run_loop_ticks(), 0);
}

#[test]
fn run_loop_with_no_pending_events_calls_no_ticks() {
    let mut backend = NativeBackend::new();
    let mut controls = Vec::new();

    backend
        .run_event_loop(|control| controls.push(control))
        .unwrap();

    assert_eq!(backend.run_loop_calls(), 1);
    assert_eq!(backend.run_loop_ticks(), 0);
    assert!(controls.is_empty());
}

#[test]
fn run_loop_drains_pending_key_events_as_continue_ticks() {
    let mut backend = NativeBackend::new();

    backend.extend_pending_events([
        InputEvent::new(InputEventKind::KeyDown { key_code: 65 }),
        InputEvent::new(InputEventKind::KeyUp { key_code: 65 }),
    ]);

    let mut controls = Vec::new();

    backend
        .run_event_loop(|control| controls.push(control))
        .unwrap();

    assert_eq!(
        controls,
        vec![LoopControl::Continue, LoopControl::Continue]
    );

    assert_eq!(backend.run_loop_calls(), 1);
    assert_eq!(backend.run_loop_ticks(), 2);
    assert_eq!(backend.pending_event_count(), 0);
}

#[test]
fn run_loop_emits_exit_requested_for_close_event() {
    let mut backend = NativeBackend::new();

    backend.push_pending_event(InputEvent::new(InputEventKind::CloseRequested));

    let mut controls = Vec::new();

    backend
        .run_event_loop(|control| controls.push(control))
        .unwrap();

    assert_eq!(controls, vec![LoopControl::ExitRequested]);
    assert_eq!(backend.run_loop_calls(), 1);
    assert_eq!(backend.run_loop_ticks(), 1);
    assert_eq!(backend.pending_event_count(), 0);
}

#[test]
fn run_loop_stops_after_close_requested() {
    let mut backend = NativeBackend::new();

    backend.extend_pending_events([
        InputEvent::new(InputEventKind::KeyDown { key_code: 65 }),
        InputEvent::new(InputEventKind::CloseRequested),
        InputEvent::new(InputEventKind::KeyDown { key_code: 66 }),
    ]);

    let mut controls = Vec::new();

    backend
        .run_event_loop(|control| controls.push(control))
        .unwrap();

    assert_eq!(
        controls,
        vec![LoopControl::Continue, LoopControl::ExitRequested]
    );

    assert_eq!(backend.run_loop_calls(), 1);
    assert_eq!(backend.run_loop_ticks(), 2);
    assert_eq!(backend.pending_event_count(), 0);
}

#[test]
fn run_loop_accumulates_counters_across_runs() {
    let mut backend = NativeBackend::new();

    backend.push_pending_event(InputEvent::new(InputEventKind::KeyDown { key_code: 1 }));

    backend.run_event_loop(|_| {}).unwrap();

    backend.push_pending_event(InputEvent::new(InputEventKind::KeyDown { key_code: 2 }));

    backend.run_event_loop(|_| {}).unwrap();

    assert_eq!(backend.run_loop_calls(), 2);
    assert_eq!(backend.run_loop_ticks(), 2);
}

#[test]
fn desktop_session_run_works_with_staged_native_backend() {
    let mut backend = NativeBackend::new();

    backend.extend_pending_events([
        InputEvent::new(InputEventKind::KeyDown { key_code: 65 }),
        InputEvent::new(InputEventKind::CloseRequested),
    ]);

    let config = WindowConfig::new("Staged Native", 640, 480);
    let mut session = DesktopSession::create(backend, config).unwrap();

    let mut frame_ticks = 0usize;

    session
        .run(|_buffer| {
            frame_ticks += 1;
            LoopControl::Continue
        })
        .unwrap();

    assert_eq!(session.state(), SessionState::Running);
    assert_eq!(frame_ticks, 2);
    assert_eq!(session.backend().run_loop_calls(), 1);
    assert_eq!(session.backend().run_loop_ticks(), 2);
    assert_eq!(session.backend().pending_event_count(), 0);
}
