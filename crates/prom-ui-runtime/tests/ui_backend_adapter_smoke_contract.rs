use std::cell::RefCell;
use std::rc::Rc;

use prom_ui::UiOperationId;
use prom_ui_runtime::{
    Color, DesktopSession, DrawFrame, LoopControl, Rect, UiBackendAdapter, UiRuntimeError,
    WindowConfig,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum BackendCall {
    CreateWindow {
        title: String,
        width: u32,
        height: u32,
    },
    RunEventLoop,
    DrawFrame {
        command_count: usize,
    },
    CloseWindow,
}

struct SmokeBackend {
    calls: Rc<RefCell<Vec<BackendCall>>>,
}

impl SmokeBackend {
    fn new(calls: Rc<RefCell<Vec<BackendCall>>>) -> Self {
        Self { calls }
    }
}

impl UiBackendAdapter for SmokeBackend {
    fn create_window(&mut self, config: &WindowConfig) -> Result<(), UiRuntimeError> {
        self.calls.borrow_mut().push(BackendCall::CreateWindow {
            title: config.title.clone(),
            width: config.width,
            height: config.height,
        });
        Ok(())
    }

    fn close_window(&mut self) {
        self.calls.borrow_mut().push(BackendCall::CloseWindow);
    }

    fn run_event_loop<F: FnMut(LoopControl, &mut prom_ui_runtime::DrawFrame)>(
        &mut self,
        mut on_event: F,
    ) -> Result<(), UiRuntimeError> {
        self.calls.borrow_mut().push(BackendCall::RunEventLoop);
        {
            let mut f = prom_ui_runtime::DrawFrame::new();
            on_event(LoopControl::ExitRequested, &mut f);
        };
        Ok(())
    }

    fn draw_frame(&mut self, frame: &DrawFrame) -> Result<(), UiRuntimeError> {
        self.calls.borrow_mut().push(BackendCall::DrawFrame {
            command_count: frame.len(),
        });
        Ok(())
    }
}

#[test]
fn backend_adapter_receives_lifecycle_calls_in_order() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let backend = SmokeBackend::new(calls.clone());

    let cfg = WindowConfig::new("Smoke", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.run(|_, _| LoopControl::ExitRequested).unwrap();

    let mut frame = DrawFrame::new();
    frame.clear(Color::BLACK);
    frame.fill_rect(Rect::new(0, 0, 10, 10), Color::GREEN);

    session.submit_frame(&frame).unwrap();
    session.close().unwrap();

    assert_eq!(
        calls.borrow().as_slice(),
        &[
            BackendCall::CreateWindow {
                title: "Smoke".into(),
                width: 640,
                height: 480,
            },
            BackendCall::RunEventLoop,
            BackendCall::DrawFrame { command_count: 2 },
            BackendCall::CloseWindow,
        ]
    );
}

#[test]
fn backend_adapter_draw_is_not_called_when_frame_submit_is_rejected() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let backend = SmokeBackend::new(calls.clone());

    let cfg = WindowConfig::new("Smoke", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    let frame = DrawFrame::new();

    let err = session.submit_frame(&frame).unwrap_err();

    assert!(matches!(
        err,
        UiRuntimeError::LifecycleViolation {
            operation: UiOperationId::FrameSubmit,
            ..
        }
    ));

    assert_eq!(
        calls.borrow().as_slice(),
        &[BackendCall::CreateWindow {
            title: "Smoke".into(),
            width: 640,
            height: 480,
        }]
    );
}

#[test]
fn backend_adapter_close_is_not_called_when_close_is_rejected() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let backend = SmokeBackend::new(calls.clone());

    let cfg = WindowConfig::new("Smoke", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.close().unwrap();

    let err = session.close().unwrap_err();

    assert!(matches!(
        err,
        UiRuntimeError::LifecycleViolation {
            operation: UiOperationId::WindowClose,
            ..
        }
    ));

    assert_eq!(
        calls.borrow().as_slice(),
        &[
            BackendCall::CreateWindow {
                title: "Smoke".into(),
                width: 640,
                height: 480,
            },
            BackendCall::CloseWindow,
        ]
    );
}

#[test]
fn backend_adapter_run_is_not_called_when_run_is_rejected() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let backend = SmokeBackend::new(calls.clone());

    let cfg = WindowConfig::new("Smoke", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.run(|_, _| LoopControl::ExitRequested).unwrap();

    let err = session.run(|_, _| LoopControl::ExitRequested).unwrap_err();

    assert!(matches!(
        err,
        UiRuntimeError::LifecycleViolation {
            operation: UiOperationId::WindowRun,
            ..
        }
    ));

    assert_eq!(
        calls.borrow().as_slice(),
        &[
            BackendCall::CreateWindow {
                title: "Smoke".into(),
                width: 640,
                height: 480,
            },
            BackendCall::RunEventLoop,
        ]
    );
}
