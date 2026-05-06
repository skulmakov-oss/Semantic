use prom_ui_runtime::{
    Color, DesktopSession, DrawFrame, EventBuffer, InputEvent, InputEventKind, LoopControl,
    SessionState, UiBackendAdapter, UiRuntimeError, WindowConfig,
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
fn desktop_session_lifecycle_gated_api_surface_roundtrips() {
    let backend = NoopBackend;
    let cfg = WindowConfig::new("Mock", 800, 600);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    let _: SessionState = session.state();

    let run_result: Result<(), UiRuntimeError> = session.run(|_| LoopControl::ExitRequested);
    run_result.unwrap();
    assert_eq!(session.state(), SessionState::Running);

    let mut buffer = EventBuffer::new();
    buffer.push(InputEvent::new(InputEventKind::KeyDown { key_code: 65 }));

    let events: Vec<InputEvent> = session.poll_events(&mut buffer).unwrap();
    assert_eq!(events.len(), 1);
    assert!(buffer.is_empty());

    let mut frame = DrawFrame::new();
    frame.clear(Color::BLACK);
    frame.fill_rect(prom_ui_runtime::Rect::new(0, 0, 10, 10), Color::GREEN);

    let submit_result: Result<(), UiRuntimeError> = session.submit_frame(&frame);
    submit_result.unwrap();
    assert_eq!(session.state(), SessionState::Running);

    let control: Result<LoopControl, UiRuntimeError> = session.tick_frame(
        &mut buffer,
        |_events, frame| {
            frame.clear(Color::BLUE);
            LoopControl::Continue
        },
    );
    assert_eq!(control.unwrap(), LoopControl::Continue);
    assert_eq!(session.state(), SessionState::Running);

    let close_result: Result<(), UiRuntimeError> = session.close();
    close_result.unwrap();
    assert_eq!(session.state(), SessionState::Closed);
}
