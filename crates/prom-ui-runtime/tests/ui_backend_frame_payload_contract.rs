use std::cell::RefCell;
use std::rc::Rc;

use prom_ui_runtime::{
    Color, DesktopSession, DrawCommand, DrawFrame, LoopControl, Rect, UiBackendAdapter,
    UiRuntimeError, WindowConfig,
};

struct CapturingBackend {
    captured_frames: Rc<RefCell<Vec<Vec<DrawCommand>>>>,
}

impl CapturingBackend {
    fn new(captured_frames: Rc<RefCell<Vec<Vec<DrawCommand>>>>) -> Self {
        Self { captured_frames }
    }
}

impl UiBackendAdapter for CapturingBackend {
    fn create_window(&mut self, _config: &WindowConfig) -> Result<(), UiRuntimeError> {
        Ok(())
    }

    fn close_window(&mut self) {}

    fn run_event_loop<F: FnMut(LoopControl, &mut prom_ui_runtime::DrawFrame)>(
        &mut self,
        mut on_event: F,
    ) -> Result<(), UiRuntimeError> {
        {
            let mut f = prom_ui_runtime::DrawFrame::new();
            on_event(LoopControl::ExitRequested, &mut f);
        };
        Ok(())
    }

    fn draw_frame(&mut self, frame: &DrawFrame) -> Result<(), UiRuntimeError> {
        self.captured_frames
            .borrow_mut()
            .push(frame.commands().to_vec());
        Ok(())
    }
}

#[test]
fn submit_frame_preserves_draw_command_order_and_payload() {
    let captured_frames = Rc::new(RefCell::new(Vec::new()));
    let backend = CapturingBackend::new(captured_frames.clone());

    let cfg = WindowConfig::new("Payload", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.run(|_, _| LoopControl::ExitRequested).unwrap();

    let mut frame = DrawFrame::new();
    frame.clear(Color::rgba(1, 2, 3, 4));
    frame.fill_rect(Rect::new(10, 20, 30, 40), Color::rgb(50, 60, 70));
    frame.draw_text("hello Semantic UI", 100, 200, Color::WHITE);

    session.submit_frame(&frame).unwrap();

    let frames = captured_frames.borrow();
    assert_eq!(frames.len(), 1);

    assert_eq!(
        frames[0],
        vec![
            DrawCommand::Clear {
                color: Color::rgba(1, 2, 3, 4),
            },
            DrawCommand::FillRect {
                rect: Rect::new(10, 20, 30, 40),
                color: Color::rgb(50, 60, 70),
            },
            DrawCommand::DrawText {
                text: "hello Semantic UI".into(),
                x: 100,
                y: 200,
                color: Color::WHITE,
            },
        ]
    );
}

#[test]
fn tick_frame_preserves_callback_built_draw_payload() {
    let captured_frames = Rc::new(RefCell::new(Vec::new()));
    let backend = CapturingBackend::new(captured_frames.clone());

    let cfg = WindowConfig::new("TickPayload", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.run(|_, _| LoopControl::ExitRequested).unwrap();

    let mut buffer = prom_ui_runtime::EventBuffer::new();

    let control = session
        .tick_frame(&mut buffer, |_events, frame| {
            frame.clear(Color::BLACK);
            frame.fill_rect(Rect::new(1, 2, 3, 4), Color::GREEN);
            frame.draw_text("tick", 5, 6, Color::BLUE);
            LoopControl::Continue
        })
        .unwrap();

    assert_eq!(control, LoopControl::Continue);

    let frames = captured_frames.borrow();
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
                text: "tick".into(),
                x: 5,
                y: 6,
                color: Color::BLUE,
            },
        ]
    );
}

#[test]
fn empty_frame_payload_reaches_backend_as_empty_command_list() {
    let captured_frames = Rc::new(RefCell::new(Vec::new()));
    let backend = CapturingBackend::new(captured_frames.clone());

    let cfg = WindowConfig::new("EmptyFrame", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.run(|_, _| LoopControl::ExitRequested).unwrap();

    let frame = DrawFrame::new();
    session.submit_frame(&frame).unwrap();

    let frames = captured_frames.borrow();
    assert_eq!(frames.len(), 1);
    assert!(frames[0].is_empty());
}

#[test]
fn multiple_frame_payloads_are_captured_independently() {
    let captured_frames = Rc::new(RefCell::new(Vec::new()));
    let backend = CapturingBackend::new(captured_frames.clone());

    let cfg = WindowConfig::new("MultiFrame", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.run(|_, _| LoopControl::ExitRequested).unwrap();

    let mut frame_a = DrawFrame::new();
    frame_a.clear(Color::RED);

    let mut frame_b = DrawFrame::new();
    frame_b.clear(Color::BLUE);
    frame_b.fill_rect(Rect::new(9, 8, 7, 6), Color::WHITE);

    session.submit_frame(&frame_a).unwrap();
    session.submit_frame(&frame_b).unwrap();

    let frames = captured_frames.borrow();
    assert_eq!(frames.len(), 2);

    assert_eq!(frames[0], vec![DrawCommand::Clear { color: Color::RED }]);

    assert_eq!(
        frames[1],
        vec![
            DrawCommand::Clear { color: Color::BLUE },
            DrawCommand::FillRect {
                rect: Rect::new(9, 8, 7, 6),
                color: Color::WHITE,
            },
        ]
    );
}
