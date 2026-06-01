use prom_ui_runtime::{
    Color, DesktopSession, DrawCommand, DrawFrame, InMemoryBackend, InputEvent, InputEventKind,
    LoopControl, Rect, WindowConfig,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReplayResult {
    final_score: i32,
    close_seen: bool,
    controls: Vec<LoopControl>,
    frames: Vec<Vec<DrawCommand>>,
}

#[derive(Debug, Default)]
struct ReplayApp {
    score: i32,
    close_seen: bool,
}

impl ReplayApp {
    fn on_frame(&mut self, events: &[InputEvent], frame: &mut DrawFrame) -> LoopControl {
        for event in events {
            match event.kind {
                InputEventKind::KeyDown { key_code: 1 } => {
                    self.score += 1;
                }
                InputEventKind::KeyDown { key_code: 2 } => {
                    self.score += 10;
                }
                InputEventKind::KeyUp { .. } => {}
                InputEventKind::CloseRequested => {
                    self.close_seen = true;
                    frame.clear(Color::BLACK);
                    frame.draw_text(format!("score={} close", self.score), 0, 0, Color::WHITE);
                    return LoopControl::ExitRequested;
                }
                _ => {}
            }
        }

        frame.clear(Color::BLACK);
        frame.fill_rect(Rect::new(0, 0, self.score as u32 + 1, 8), Color::GREEN);
        frame.draw_text(format!("score={}", self.score), 0, 0, Color::WHITE);

        LoopControl::Continue
    }
}

fn event_script() -> Vec<Vec<InputEvent>> {
    vec![
        vec![
            InputEvent::new(InputEventKind::KeyDown { key_code: 1 }),
            InputEvent::new(InputEventKind::KeyDown { key_code: 1 }),
        ],
        vec![InputEvent::new(InputEventKind::KeyDown { key_code: 2 })],
        vec![
            InputEvent::new(InputEventKind::KeyUp { key_code: 1 }),
            InputEvent::new(InputEventKind::CloseRequested),
        ],
    ]
}

fn event_script_with_trailing_events() -> Vec<Vec<InputEvent>> {
    vec![
        vec![InputEvent::new(InputEventKind::CloseRequested)],
        vec![InputEvent::new(InputEventKind::KeyDown { key_code: 2 })],
    ]
}

fn run_replay(script: Vec<Vec<InputEvent>>) -> ReplayResult {
    let backend = InMemoryBackend::new();
    let cfg = WindowConfig::new("Replay", 640, 480);
    let mut session = DesktopSession::create(backend, cfg).unwrap();

    session.run(|_| LoopControl::ExitRequested).unwrap();

    let mut app = ReplayApp::default();
    let mut controls = Vec::new();

    for tick_events in script {
        session.backend_mut().extend_events(tick_events);

        let control = session
            .tick_in_memory_frame(|events, frame| app.on_frame(events, frame))
            .unwrap();

        controls.push(control);

        if control == LoopControl::ExitRequested {
            break;
        }
    }

    ReplayResult {
        final_score: app.score,
        close_seen: app.close_seen,
        controls,
        frames: session.backend().captured_frames().to_vec(),
    }
}

#[test]
fn same_event_script_replays_to_same_result() {
    let first = run_replay(event_script());
    let second = run_replay(event_script());

    assert_eq!(first, second);
}

#[test]
fn replay_result_matches_expected_transcript() {
    let result = run_replay(event_script());

    assert_eq!(result.final_score, 12);
    assert!(result.close_seen);

    assert_eq!(
        result.controls,
        vec![
            LoopControl::Continue,
            LoopControl::Continue,
            LoopControl::ExitRequested,
        ]
    );

    assert_eq!(result.frames.len(), 3);

    assert_eq!(
        result.frames[0],
        vec![
            DrawCommand::Clear {
                color: Color::BLACK,
            },
            DrawCommand::FillRect {
                rect: Rect::new(0, 0, 3, 8),
                color: Color::GREEN,
            },
            DrawCommand::DrawText {
                text: "score=2".into(),
                x: 0,
                y: 0,
                color: Color::WHITE,
            },
        ]
    );

    assert_eq!(
        result.frames[1],
        vec![
            DrawCommand::Clear {
                color: Color::BLACK,
            },
            DrawCommand::FillRect {
                rect: Rect::new(0, 0, 13, 8),
                color: Color::GREEN,
            },
            DrawCommand::DrawText {
                text: "score=12".into(),
                x: 0,
                y: 0,
                color: Color::WHITE,
            },
        ]
    );

    assert_eq!(
        result.frames[2],
        vec![
            DrawCommand::Clear {
                color: Color::BLACK,
            },
            DrawCommand::DrawText {
                text: "score=12 close".into(),
                x: 0,
                y: 0,
                color: Color::WHITE,
            },
        ]
    );
}

#[test]
fn replay_stops_after_exit_requested() {
    let result = run_replay(event_script_with_trailing_events());

    assert_eq!(result.controls, vec![LoopControl::ExitRequested]);
    assert_eq!(result.frames.len(), 1);
    assert_eq!(result.final_score, 0);
    assert!(result.close_seen);
}
