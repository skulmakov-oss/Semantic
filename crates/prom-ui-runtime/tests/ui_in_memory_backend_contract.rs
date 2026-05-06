use prom_ui_runtime::{
    Color, DesktopSession, DrawCommand, DrawFrame, EventBuffer, InMemoryBackend, InputEvent,
    InputEventKind, LoopControl, Rect, SessionState, WindowConfig,
};

#[test]
fn in_memory_backend_records_window_config() {
    let backend = InMemoryBackend::new();
    let cfg = WindowConfig::new("Memory", 640, 480);

    let session = DesktopSession::create(backend, cfg).unwrap();

    let stored = session.backend().config().expect("config must be recorded");
    assert_eq!(stored.title, "Memory");
    assert_eq!(stored.width, 640);
    assert_eq!(stored.height, 480);
    assert_eq!(session.state(), SessionState::Created);
}

#[test]
fn in_memory_backend_counts_run_event_loop_calls() {
    let backend = InMemoryBackend::new();
    let cfg = WindowConfig::new("RunCounter", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.run(|_| LoopControl::ExitRequested).unwrap();

    assert_eq!(session.backend().run_event_loop_calls(), 1);
    assert_eq!(session.state(), SessionState::Running);
}

#[test]
fn in_memory_backend_captures_submitted_frame_payload() {
    let backend = InMemoryBackend::new();
    let cfg = WindowConfig::new("FrameCapture", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.run(|_| LoopControl::ExitRequested).unwrap();

    let mut frame = DrawFrame::new();
    frame.clear(Color::BLACK);
    frame.fill_rect(Rect::new(1, 2, 3, 4), Color::GREEN);
    frame.draw_text("hello", 5, 6, Color::BLUE);

    session.submit_frame(&frame).unwrap();

    let frames = session.backend().captured_frames();
    assert_eq!(frames.len(), 1);

    assert_eq!(
        frames[0],
        vec![
            DrawCommand::Clear {
                color: Color::BLACK,
            },
            DrawCommand::FillRect {
                rect: Rect::new(1, 2, 3, 4),
                color: Color::GREEN,
            },
            DrawCommand::DrawText {
                text: "hello".into(),
                x: 5,
                y: 6,
                color: Color::BLUE,
            },
        ]
    );
}

#[test]
fn in_memory_backend_captures_tick_frame_payload() {
    let backend = InMemoryBackend::new();
    let cfg = WindowConfig::new("TickCapture", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.run(|_| LoopControl::ExitRequested).unwrap();

    let mut buffer = EventBuffer::new();
    buffer.push(InputEvent::new(InputEventKind::KeyDown { key_code: 65 }));

    let control = session
        .tick_frame(&mut buffer, |events, frame| {
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].kind, InputEventKind::KeyDown { key_code: 65 });

            frame.clear(Color::WHITE);
            LoopControl::Continue
        })
        .unwrap();

    assert_eq!(control, LoopControl::Continue);

    let frames = session.backend().captured_frames();
    assert_eq!(frames.len(), 1);
    assert_eq!(
        frames[0],
        vec![DrawCommand::Clear {
            color: Color::WHITE,
        }]
    );
}

#[test]
fn in_memory_backend_counts_close_calls() {
    let backend = InMemoryBackend::new();
    let cfg = WindowConfig::new("CloseCounter", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.close().unwrap();

    assert_eq!(session.backend().close_window_calls(), 1);
    assert_eq!(session.state(), SessionState::Closed);
}

#[test]
fn in_memory_backend_does_not_capture_rejected_frame() {
    let backend = InMemoryBackend::new();
    let cfg = WindowConfig::new("RejectedFrame", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    let mut frame = DrawFrame::new();
    frame.clear(Color::RED);

    let err = session.submit_frame(&frame).unwrap_err();

    assert!(matches!(
        err,
        prom_ui_runtime::UiRuntimeError::LifecycleViolation { .. }
    ));

    assert_eq!(session.backend().captured_frame_count(), 0);
}
