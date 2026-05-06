use prom_ui_runtime::{
    Color, DesktopSession, DrawCommand, InMemoryBackend, InputEvent, InputEventKind,
    LoopControl, SessionState, UiRuntimeError, WindowConfig,
};

#[test]
fn tick_in_memory_frame_requires_running_and_preserves_queue() {
    let backend = InMemoryBackend::new();
    let cfg = WindowConfig::new("TickBeforeRun", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session
        .backend_mut()
        .push_event(InputEvent::new(InputEventKind::KeyDown { key_code: 65 }));

    let mut callback_called = false;

    let err = session
        .tick_in_memory_frame(|_events, _frame| {
            callback_called = true;
            LoopControl::Continue
        })
        .expect_err("tick before run must fail");

    assert!(matches!(
        err,
        UiRuntimeError::LifecycleViolation { .. }
    ));

    assert!(!callback_called);
    assert_eq!(session.backend().queued_event_count(), 1);
    assert_eq!(
        session.backend().queued_events()[0].kind,
        InputEventKind::KeyDown { key_code: 65 }
    );
    assert_eq!(session.backend().captured_frame_count(), 0);
}

#[test]
fn tick_in_memory_frame_delivers_queued_events_to_callback() {
    let backend = InMemoryBackend::new();
    let cfg = WindowConfig::new("TickQueuedEvents", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.run(|_| LoopControl::ExitRequested).unwrap();

    session.backend_mut().extend_events([
        InputEvent::new(InputEventKind::KeyDown { key_code: 11 }),
        InputEvent::new(InputEventKind::KeyUp { key_code: 11 }),
        InputEvent::new(InputEventKind::CloseRequested),
    ]);

    let control = session
        .tick_in_memory_frame(|events, frame| {
            assert_eq!(events.len(), 3);
            assert_eq!(events[0].kind, InputEventKind::KeyDown { key_code: 11 });
            assert_eq!(events[1].kind, InputEventKind::KeyUp { key_code: 11 });
            assert_eq!(events[2].kind, InputEventKind::CloseRequested);

            frame.clear(Color::BLACK);
            LoopControl::Continue
        })
        .unwrap();

    assert_eq!(control, LoopControl::Continue);
    assert_eq!(session.backend().queued_event_count(), 0);
    assert_eq!(session.backend().captured_frame_count(), 1);
}

#[test]
fn tick_in_memory_frame_captures_callback_draw_payload() {
    let backend = InMemoryBackend::new();
    let cfg = WindowConfig::new("TickDrawPayload", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.run(|_| LoopControl::ExitRequested).unwrap();

    let control = session
        .tick_in_memory_frame(|_events, frame| {
            frame.clear(Color::WHITE);
            frame.draw_text("in-memory tick", 10, 20, Color::BLUE);
            LoopControl::ExitRequested
        })
        .unwrap();

    assert_eq!(control, LoopControl::ExitRequested);

    let frames = session.backend().captured_frames();
    assert_eq!(frames.len(), 1);

    assert_eq!(
        frames[0],
        vec![
            DrawCommand::Clear {
                color: Color::WHITE,
            },
            DrawCommand::DrawText {
                text: "in-memory tick".into(),
                x: 10,
                y: 20,
                color: Color::BLUE,
            },
        ]
    );
}

#[test]
fn tick_in_memory_frame_empty_queue_passes_empty_events() {
    let backend = InMemoryBackend::new();
    let cfg = WindowConfig::new("TickEmptyQueue", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.run(|_| LoopControl::ExitRequested).unwrap();

    let control = session
        .tick_in_memory_frame(|events, frame| {
            assert!(events.is_empty());
            frame.clear(Color::BLACK);
            LoopControl::Continue
        })
        .unwrap();

    assert_eq!(control, LoopControl::Continue);
    assert_eq!(session.backend().queued_event_count(), 0);
    assert_eq!(session.backend().captured_frame_count(), 1);
}

#[test]
fn tick_in_memory_frame_after_close_preserves_queue() {
    let backend = InMemoryBackend::new();
    let cfg = WindowConfig::new("TickAfterClose", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.run(|_| LoopControl::ExitRequested).unwrap();
    session.close().unwrap();

    session
        .backend_mut()
        .push_event(InputEvent::new(InputEventKind::CloseRequested));

    let mut callback_called = false;

    let err = session
        .tick_in_memory_frame(|_events, _frame| {
            callback_called = true;
            LoopControl::Continue
        })
        .expect_err("tick after close must fail");

    assert!(matches!(
        err,
        UiRuntimeError::LifecycleViolation { .. }
    ));

    assert!(!callback_called);
    assert_eq!(session.backend().queued_event_count(), 1);
    assert_eq!(
        session.backend().queued_events()[0].kind,
        InputEventKind::CloseRequested
    );
    assert_eq!(session.backend().captured_frame_count(), 0);
    assert_eq!(session.state(), SessionState::Closed);
}

#[test]
fn multiple_in_memory_ticks_are_independent() {
    let backend = InMemoryBackend::new();
    let cfg = WindowConfig::new("MultiTick", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.run(|_| LoopControl::ExitRequested).unwrap();

    session
        .backend_mut()
        .push_event(InputEvent::new(InputEventKind::KeyDown { key_code: 1 }));

    session
        .tick_in_memory_frame(|events, frame| {
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].kind, InputEventKind::KeyDown { key_code: 1 });
            frame.clear(Color::BLACK);
            LoopControl::Continue
        })
        .unwrap();

    session
        .backend_mut()
        .push_event(InputEvent::new(InputEventKind::KeyDown { key_code: 2 }));

    session
        .tick_in_memory_frame(|events, frame| {
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].kind, InputEventKind::KeyDown { key_code: 2 });
            frame.clear(Color::WHITE);
            LoopControl::ExitRequested
        })
        .unwrap();

    assert_eq!(session.backend().queued_event_count(), 0);

    let frames = session.backend().captured_frames();
    assert_eq!(frames.len(), 2);
    assert_eq!(
        frames[0],
        vec![DrawCommand::Clear {
            color: Color::BLACK,
        }]
    );
    assert_eq!(
        frames[1],
        vec![DrawCommand::Clear {
            color: Color::WHITE,
        }]
    );
}
