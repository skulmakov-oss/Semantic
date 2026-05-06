use prom_ui_runtime::{
    Color, DesktopSession, DrawFrame, EventBuffer, InputEvent, InputEventKind, LoopControl,
    UiBackendAdapter, UiRuntimeError, WindowConfig,
};

struct NoopBackend;

impl UiBackendAdapter for NoopBackend {
    fn create_window(&mut self, _config: &WindowConfig) -> Result<(), UiRuntimeError> {
        Ok(())
    }

    fn close_window(&mut self) {}

    fn run_event_loop<F: FnMut(LoopControl)>(
        &mut self,
        mut on_event: F,
    ) -> Result<(), UiRuntimeError> {
        on_event(LoopControl::ExitRequested);
        Ok(())
    }

    fn draw_frame(&mut self, _frame: &DrawFrame) -> Result<(), UiRuntimeError> {
        Ok(())
    }
}

#[test]
fn poll_events_preserves_event_order_and_payload() {
    let backend = NoopBackend;
    let cfg = WindowConfig::new("EventPayload", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.run(|_| LoopControl::ExitRequested).unwrap();

    let mut buffer = EventBuffer::new();
    buffer.push(InputEvent::new(InputEventKind::KeyDown { key_code: 65 }));
    buffer.push(InputEvent::new(InputEventKind::KeyUp { key_code: 65 }));
    buffer.push(InputEvent::new(InputEventKind::CloseRequested));

    let events = session.poll_events(&mut buffer).unwrap();

    assert_eq!(events.len(), 3);
    assert_eq!(events[0].kind, InputEventKind::KeyDown { key_code: 65 });
    assert_eq!(events[1].kind, InputEventKind::KeyUp { key_code: 65 });
    assert_eq!(events[2].kind, InputEventKind::CloseRequested);
    assert!(buffer.is_empty());
}

#[test]
fn tick_frame_passes_event_payload_to_callback() {
    let backend = NoopBackend;
    let cfg = WindowConfig::new("TickEventPayload", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.run(|_| LoopControl::ExitRequested).unwrap();

    let mut buffer = EventBuffer::new();
    buffer.push(InputEvent::new(InputEventKind::KeyDown { key_code: 13 }));
    buffer.push(InputEvent::new(InputEventKind::KeyUp { key_code: 13 }));

    let control = session
        .tick_frame(&mut buffer, |events, frame| {
            assert_eq!(events.len(), 2);
            assert_eq!(events[0].kind, InputEventKind::KeyDown { key_code: 13 });
            assert_eq!(events[1].kind, InputEventKind::KeyUp { key_code: 13 });

            frame.clear(Color::BLACK);
            LoopControl::Continue
        })
        .unwrap();

    assert_eq!(control, LoopControl::Continue);
    assert!(buffer.is_empty());
}

#[test]
fn empty_event_buffer_polls_as_empty_event_list() {
    let backend = NoopBackend;
    let cfg = WindowConfig::new("EmptyEvents", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.run(|_| LoopControl::ExitRequested).unwrap();

    let mut buffer = EventBuffer::new();

    let events = session.poll_events(&mut buffer).unwrap();

    assert!(events.is_empty());
    assert!(buffer.is_empty());
}

#[test]
fn multiple_event_drains_are_independent() {
    let backend = NoopBackend;
    let cfg = WindowConfig::new("MultiDrain", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.run(|_| LoopControl::ExitRequested).unwrap();

    let mut buffer = EventBuffer::new();

    buffer.push(InputEvent::new(InputEventKind::KeyDown { key_code: 1 }));
    let first = session.poll_events(&mut buffer).unwrap();

    assert_eq!(first.len(), 1);
    assert_eq!(first[0].kind, InputEventKind::KeyDown { key_code: 1 });
    assert!(buffer.is_empty());

    buffer.push(InputEvent::new(InputEventKind::KeyDown { key_code: 2 }));
    buffer.push(InputEvent::new(InputEventKind::CloseRequested));
    let second = session.poll_events(&mut buffer).unwrap();

    assert_eq!(second.len(), 2);
    assert_eq!(second[0].kind, InputEventKind::KeyDown { key_code: 2 });
    assert_eq!(second[1].kind, InputEventKind::CloseRequested);
    assert!(buffer.is_empty());
}

#[test]
fn failed_poll_preserves_event_payload() {
    let backend = NoopBackend;
    let cfg = WindowConfig::new("FailedPoll", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    let mut buffer = EventBuffer::new();
    buffer.push(InputEvent::new(InputEventKind::KeyDown { key_code: 77 }));
    buffer.push(InputEvent::new(InputEventKind::KeyUp { key_code: 77 }));

    let err = session
        .poll_events(&mut buffer)
        .expect_err("poll before run must fail");

    assert!(matches!(
        err,
        UiRuntimeError::LifecycleViolation { .. }
    ));

    let raw_events = buffer.drain();
    assert_eq!(raw_events.len(), 2);
    assert_eq!(raw_events[0].kind, InputEventKind::KeyDown { key_code: 77 });
    assert_eq!(raw_events[1].kind, InputEventKind::KeyUp { key_code: 77 });
}

#[test]
fn failed_tick_preserves_event_payload() {
    let backend = NoopBackend;
    let cfg = WindowConfig::new("FailedTick", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    let mut buffer = EventBuffer::new();
    buffer.push(InputEvent::new(InputEventKind::CloseRequested));

    let mut callback_called = false;

    let err = session
        .tick_frame(&mut buffer, |_events, _frame| {
            callback_called = true;
            LoopControl::Continue
        })
        .expect_err("tick before run must fail");

    assert!(matches!(
        err,
        UiRuntimeError::LifecycleViolation { .. }
    ));

    assert!(!callback_called);

    let raw_events = buffer.drain();
    assert_eq!(raw_events.len(), 1);
    assert_eq!(raw_events[0].kind, InputEventKind::CloseRequested);
}
