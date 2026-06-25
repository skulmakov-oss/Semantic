use prom_ui_runtime::{
    Color, DesktopSession, DrawFrame, EventBuffer, LoopControl, Rect, SessionState,
    UiBackendAdapter, UiRuntimeError, WindowConfig,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FailPoint {
    CreateWindow,
    RunEventLoop,
    DrawFrame,
}

struct FailingBackend {
    fail_at: FailPoint,
}

impl FailingBackend {
    fn new(fail_at: FailPoint) -> Self {
        Self { fail_at }
    }
}

impl UiBackendAdapter for FailingBackend {
    fn create_window(&mut self, _config: &WindowConfig) -> Result<(), UiRuntimeError> {
        if self.fail_at == FailPoint::CreateWindow {
            return Err(UiRuntimeError::WindowCreationFailed);
        }
        Ok(())
    }

    fn close_window(&mut self) {}

    fn run_event_loop<F: FnMut(LoopControl, &mut prom_ui_runtime::DrawFrame)>(
        &mut self,
        _on_event: F,
    ) -> Result<(), UiRuntimeError> {
        if self.fail_at == FailPoint::RunEventLoop {
            return Err(UiRuntimeError::EventLoopFailed);
        }
        Ok(())
    }

    fn draw_frame(&mut self, _frame: &DrawFrame) -> Result<(), UiRuntimeError> {
        if self.fail_at == FailPoint::DrawFrame {
            return Err(UiRuntimeError::EventLoopFailed);
        }
        Ok(())
    }
}

#[test]
fn create_window_error_is_propagated_from_session_create() {
    let backend = FailingBackend::new(FailPoint::CreateWindow);
    let cfg = WindowConfig::new("FailCreate", 640, 480);

    let err = match DesktopSession::create(backend, cfg) {
        Ok(_) => panic!("create must fail"),
        Err(err) => err,
    };

    assert_eq!(err, UiRuntimeError::WindowCreationFailed);
}

#[test]
fn run_event_loop_error_is_propagated_from_session_run() {
    let backend = FailingBackend::new(FailPoint::RunEventLoop);
    let cfg = WindowConfig::new("FailRun", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    let err = session.run(|_, _| LoopControl::Continue).unwrap_err();

    assert_eq!(err, UiRuntimeError::EventLoopFailed);
    assert_eq!(session.state(), SessionState::Running);
}

#[test]
fn draw_frame_error_is_propagated_from_submit_frame() {
    let backend = FailingBackend::new(FailPoint::DrawFrame);
    let cfg = WindowConfig::new("FailDraw", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.run(|_, _| LoopControl::ExitRequested).unwrap();

    let mut frame = DrawFrame::new();
    frame.clear(Color::BLACK);
    frame.fill_rect(Rect::new(0, 0, 10, 10), Color::GREEN);

    let err = session.submit_frame(&frame).unwrap_err();

    assert_eq!(err, UiRuntimeError::EventLoopFailed);
    assert_eq!(session.state(), SessionState::Running);
}

#[test]
fn draw_frame_error_is_propagated_from_tick_frame() {
    let backend = FailingBackend::new(FailPoint::DrawFrame);
    let cfg = WindowConfig::new("FailTickDraw", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.run(|_, _| LoopControl::ExitRequested).unwrap();

    let mut buffer = EventBuffer::new();

    let err = session
        .tick_frame(&mut buffer, |_events, frame| {
            frame.clear(Color::BLUE);
            LoopControl::Continue
        })
        .unwrap_err();

    assert_eq!(err, UiRuntimeError::EventLoopFailed);
    assert_eq!(session.state(), SessionState::Running);
}

#[test]
fn lifecycle_error_takes_precedence_before_backend_draw_error() {
    let backend = FailingBackend::new(FailPoint::DrawFrame);
    let cfg = WindowConfig::new("LifecycleFirst", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    let frame = DrawFrame::new();
    let err = session.submit_frame(&frame).unwrap_err();

    assert!(matches!(err, UiRuntimeError::LifecycleViolation { .. }));
}
