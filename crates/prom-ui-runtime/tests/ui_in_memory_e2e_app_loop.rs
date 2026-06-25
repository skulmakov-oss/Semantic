use prom_ui_runtime::{
    Color, DesktopSession, DrawCommand, DrawFrame, InMemoryBackend, InputEvent, InputEventKind,
    LoopControl, Rect, SessionState, UiRuntimeError, WindowConfig,
};

#[derive(Debug, Default)]
struct CounterApp {
    counter: i32,
    close_seen: bool,
}

impl CounterApp {
    fn on_frame(&mut self, events: &[InputEvent], frame: &mut DrawFrame) -> LoopControl {
        for event in events {
            match event.kind {
                InputEventKind::KeyDown { key_code: 1 } => {
                    self.counter += 1;
                }
                InputEventKind::KeyDown { key_code: 2 } => {
                    self.counter += 10;
                }
                InputEventKind::KeyUp { .. } => {}
                InputEventKind::CloseRequested => {
                    self.close_seen = true;
                    frame.clear(Color::BLACK);
                    frame.draw_text(
                        format!("counter={} closing", self.counter),
                        10,
                        20,
                        Color::WHITE,
                    );
                    return LoopControl::ExitRequested;
                }
                _ => {}
            }
        }

        frame.clear(Color::BLACK);
        frame.fill_rect(Rect::new(0, 0, self.counter as u32 + 1, 10), Color::GREEN);
        frame.draw_text(format!("counter={}", self.counter), 10, 20, Color::WHITE);

        LoopControl::Continue
    }
}

#[test]
fn in_memory_e2e_app_loop_updates_state_and_captures_frame() {
    let backend = InMemoryBackend::new();
    let cfg = WindowConfig::new("E2E", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.run(|_, _| LoopControl::ExitRequested).unwrap();
    assert_eq!(session.state(), SessionState::Running);

    session
        .backend_mut()
        .push_event(InputEvent::new(InputEventKind::KeyDown { key_code: 1 }));

    let mut app = CounterApp::default();

    let control = session
        .tick_in_memory_frame(|events, frame| app.on_frame(events, frame))
        .unwrap();

    assert_eq!(control, LoopControl::Continue);
    assert_eq!(app.counter, 1);
    assert!(!app.close_seen);
    assert_eq!(session.backend().queued_event_count(), 0);

    let frames = session.backend().captured_frames();
    assert_eq!(frames.len(), 1);

    assert_eq!(
        frames[0],
        vec![
            DrawCommand::Clear {
                color: Color::BLACK,
            },
            DrawCommand::FillRect {
                rect: Rect::new(0, 0, 2, 10),
                color: Color::GREEN,
            },
            DrawCommand::DrawText {
                text: "counter=1".into(),
                x: 10,
                y: 20,
                color: Color::WHITE,
            },
        ]
    );
}

#[test]
fn in_memory_e2e_app_loop_accumulates_state_across_ticks() {
    let backend = InMemoryBackend::new();
    let cfg = WindowConfig::new("E2EMultiTick", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.run(|_, _| LoopControl::ExitRequested).unwrap();

    let mut app = CounterApp::default();

    session
        .backend_mut()
        .push_event(InputEvent::new(InputEventKind::KeyDown { key_code: 1 }));

    let first = session
        .tick_in_memory_frame(|events, frame| app.on_frame(events, frame))
        .unwrap();

    assert_eq!(first, LoopControl::Continue);
    assert_eq!(app.counter, 1);

    session
        .backend_mut()
        .push_event(InputEvent::new(InputEventKind::KeyDown { key_code: 2 }));

    let second = session
        .tick_in_memory_frame(|events, frame| app.on_frame(events, frame))
        .unwrap();

    assert_eq!(second, LoopControl::Continue);
    assert_eq!(app.counter, 11);

    let frames = session.backend().captured_frames();
    assert_eq!(frames.len(), 2);

    assert_eq!(
        frames[0],
        vec![
            DrawCommand::Clear {
                color: Color::BLACK,
            },
            DrawCommand::FillRect {
                rect: Rect::new(0, 0, 2, 10),
                color: Color::GREEN,
            },
            DrawCommand::DrawText {
                text: "counter=1".into(),
                x: 10,
                y: 20,
                color: Color::WHITE,
            },
        ]
    );

    assert_eq!(
        frames[1],
        vec![
            DrawCommand::Clear {
                color: Color::BLACK,
            },
            DrawCommand::FillRect {
                rect: Rect::new(0, 0, 12, 10),
                color: Color::GREEN,
            },
            DrawCommand::DrawText {
                text: "counter=11".into(),
                x: 10,
                y: 20,
                color: Color::WHITE,
            },
        ]
    );
}

#[test]
fn in_memory_e2e_close_event_returns_exit_requested_and_can_close_session() {
    let backend = InMemoryBackend::new();
    let cfg = WindowConfig::new("E2EClose", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.run(|_, _| LoopControl::ExitRequested).unwrap();

    let mut app = CounterApp::default();

    session
        .backend_mut()
        .push_event(InputEvent::new(InputEventKind::KeyDown { key_code: 1 }));
    session
        .backend_mut()
        .push_event(InputEvent::new(InputEventKind::CloseRequested));

    let control = session
        .tick_in_memory_frame(|events, frame| app.on_frame(events, frame))
        .unwrap();

    assert_eq!(control, LoopControl::ExitRequested);
    assert_eq!(app.counter, 1);
    assert!(app.close_seen);

    let frames = session.backend().captured_frames();
    assert_eq!(frames.len(), 1);

    assert_eq!(
        frames[0],
        vec![
            DrawCommand::Clear {
                color: Color::BLACK,
            },
            DrawCommand::DrawText {
                text: "counter=1 closing".into(),
                x: 10,
                y: 20,
                color: Color::WHITE,
            },
        ]
    );

    session.close().unwrap();
    assert_eq!(session.state(), SessionState::Closed);
    assert_eq!(session.backend().close_window_calls(), 1);
}

#[test]
fn in_memory_e2e_tick_after_close_is_rejected_and_preserves_events() {
    let backend = InMemoryBackend::new();
    let cfg = WindowConfig::new("E2EAfterClose", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.run(|_, _| LoopControl::ExitRequested).unwrap();
    session.close().unwrap();

    session
        .backend_mut()
        .push_event(InputEvent::new(InputEventKind::KeyDown { key_code: 1 }));

    let mut app = CounterApp::default();
    let mut callback_called = false;

    let err = session
        .tick_in_memory_frame(|events, frame| {
            callback_called = true;
            app.on_frame(events, frame)
        })
        .expect_err("tick after close must fail");

    assert!(matches!(err, UiRuntimeError::LifecycleViolation { .. }));

    assert!(!callback_called);
    assert_eq!(app.counter, 0);
    assert!(!app.close_seen);
    assert_eq!(session.backend().queued_event_count(), 1);
    assert_eq!(
        session.backend().queued_events()[0].kind,
        InputEventKind::KeyDown { key_code: 1 }
    );
    assert_eq!(session.backend().captured_frame_count(), 0);
}
