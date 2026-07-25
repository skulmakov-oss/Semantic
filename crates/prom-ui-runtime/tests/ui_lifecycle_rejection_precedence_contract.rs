use std::cell::RefCell;
use std::rc::Rc;

use prom_ui::UiOperationId;
use prom_ui_runtime::{
    DesktopSession, DrawFrame, EventBuffer, InputEvent, InputEventKind, LoopControl,
    UiBackendAdapter, UiRuntimeError, WindowConfig,
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct BackendCounters {
    create_window: usize,
    run_event_loop: usize,
    draw_frame: usize,
    close_window: usize,
}

struct FailingCountingBackend {
    counters: Rc<RefCell<BackendCounters>>,
}

impl FailingCountingBackend {
    fn new(counters: Rc<RefCell<BackendCounters>>) -> Self {
        Self { counters }
    }
}

impl UiBackendAdapter for FailingCountingBackend {
    fn create_window(&mut self, _config: &WindowConfig) -> Result<(), UiRuntimeError> {
        self.counters.borrow_mut().create_window += 1;
        Ok(())
    }

    fn close_window(&mut self) {
        self.counters.borrow_mut().close_window += 1;
    }

    fn run_event_loop<F: FnMut(&[InputEvent], LoopControl, &mut prom_ui_runtime::DrawFrame)>(
        &mut self,
        _on_event: F,
    ) -> Result<(), UiRuntimeError> {
        self.counters.borrow_mut().run_event_loop += 1;
        Err(UiRuntimeError::EventLoopFailed)
    }

    fn draw_frame(&mut self, _frame: &DrawFrame) -> Result<(), UiRuntimeError> {
        self.counters.borrow_mut().draw_frame += 1;
        Err(UiRuntimeError::EventLoopFailed)
    }
}

#[test]
fn run_after_close_returns_lifecycle_error_before_backend_run_error() {
    let counters = Rc::new(RefCell::new(BackendCounters::default()));
    let backend = FailingCountingBackend::new(counters.clone());

    let cfg = WindowConfig::new("PrecedenceRun", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.close().unwrap();

    let err = session
        .run(|_, _| LoopControl::Continue)
        .expect_err("run after close must fail at lifecycle gate");

    assert!(matches!(
        err,
        UiRuntimeError::LifecycleViolation {
            operation: UiOperationId::WindowRun,
            ..
        }
    ));

    let counters = counters.borrow();
    assert_eq!(counters.create_window, 1);
    assert_eq!(counters.close_window, 1);
    assert_eq!(counters.run_event_loop, 0);
    assert_eq!(counters.draw_frame, 0);
}

#[test]
fn submit_before_run_returns_lifecycle_error_before_backend_draw_error() {
    let counters = Rc::new(RefCell::new(BackendCounters::default()));
    let backend = FailingCountingBackend::new(counters.clone());

    let cfg = WindowConfig::new("PrecedenceSubmitBeforeRun", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    let frame = DrawFrame::new();

    let err = session
        .submit_frame(&frame)
        .expect_err("submit before run must fail at lifecycle gate");

    assert!(matches!(
        err,
        UiRuntimeError::LifecycleViolation {
            operation: UiOperationId::FrameSubmit,
            ..
        }
    ));

    let counters = counters.borrow();
    assert_eq!(counters.create_window, 1);
    assert_eq!(counters.run_event_loop, 0);
    assert_eq!(counters.draw_frame, 0);
    assert_eq!(counters.close_window, 0);
}

#[test]
fn submit_after_close_returns_lifecycle_error_before_backend_draw_error() {
    let counters = Rc::new(RefCell::new(BackendCounters::default()));
    let backend = FailingCountingBackend::new(counters.clone());

    let cfg = WindowConfig::new("PrecedenceSubmitAfterClose", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.close().unwrap();

    let frame = DrawFrame::new();

    let err = session
        .submit_frame(&frame)
        .expect_err("submit after close must fail at lifecycle gate");

    assert!(matches!(
        err,
        UiRuntimeError::LifecycleViolation {
            operation: UiOperationId::FrameSubmit,
            ..
        }
    ));

    let counters = counters.borrow();
    assert_eq!(counters.create_window, 1);
    assert_eq!(counters.close_window, 1);
    assert_eq!(counters.draw_frame, 0);
}

#[test]
fn poll_before_run_returns_lifecycle_error_and_preserves_buffer() {
    let counters = Rc::new(RefCell::new(BackendCounters::default()));
    let backend = FailingCountingBackend::new(counters.clone());

    let cfg = WindowConfig::new("PrecedencePollBeforeRun", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    let mut buffer = EventBuffer::new();
    buffer.push(InputEvent::new(InputEventKind::KeyDown { key_code: 42 }));

    let err = session
        .poll_events(&mut buffer)
        .expect_err("poll before run must fail at lifecycle gate");

    assert!(matches!(
        err,
        UiRuntimeError::LifecycleViolation {
            operation: UiOperationId::EventPoll,
            ..
        }
    ));

    let remaining = buffer.drain();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].kind, InputEventKind::KeyDown { key_code: 42 });

    let counters = counters.borrow();
    assert_eq!(counters.create_window, 1);
    assert_eq!(counters.run_event_loop, 0);
    assert_eq!(counters.draw_frame, 0);
}

#[test]
fn tick_before_run_returns_lifecycle_error_before_callback_or_backend_draw() {
    let counters = Rc::new(RefCell::new(BackendCounters::default()));
    let backend = FailingCountingBackend::new(counters.clone());

    let cfg = WindowConfig::new("PrecedenceTickBeforeRun", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    let mut buffer = EventBuffer::new();
    buffer.push(InputEvent::new(InputEventKind::CloseRequested));

    let mut callback_called = false;

    let err = session
        .tick_frame(&mut buffer, |_events, _frame| {
            callback_called = true;
            LoopControl::Continue
        })
        .expect_err("tick before run must fail at EventPoll lifecycle gate");

    assert!(matches!(
        err,
        UiRuntimeError::LifecycleViolation {
            operation: UiOperationId::EventPoll,
            ..
        }
    ));

    assert!(!callback_called);

    let remaining = buffer.drain();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].kind, InputEventKind::CloseRequested);

    let counters = counters.borrow();
    assert_eq!(counters.create_window, 1);
    assert_eq!(counters.draw_frame, 0);
}

#[test]
fn second_close_returns_lifecycle_error_before_backend_close() {
    let counters = Rc::new(RefCell::new(BackendCounters::default()));
    let backend = FailingCountingBackend::new(counters.clone());

    let cfg = WindowConfig::new("PrecedenceSecondClose", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.close().unwrap();

    let err = session
        .close()
        .expect_err("second close must fail at lifecycle gate");

    assert!(matches!(
        err,
        UiRuntimeError::LifecycleViolation {
            operation: UiOperationId::WindowClose,
            ..
        }
    ));

    let counters = counters.borrow();
    assert_eq!(counters.create_window, 1);
    assert_eq!(counters.close_window, 1);
}
