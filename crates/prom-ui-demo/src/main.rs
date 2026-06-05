//! Canonical demo application for the Semantic UI application boundary.
//!
//! This binary demonstrates the first-wave UI contract:
//! - single-window session creation via `DesktopSession`
//! - per-frame event polling via `EventBuffer`
//! - draw-command submission via `DrawFrame`
//!
//! The demo uses `NullBackend` — a no-op backend that records submitted frames
//! for verification. A real platform backend (winit + wgpu or equivalent) is
//! wired in a later wave. The demo is kept as a consumer of the owner-layer
//! contract, not an owner of it.
//!
//! # Non-Commitments
//!
//! This binary does not claim:
//! - that a visible window actually opens on the desktop
//! - that draw commands produce pixels (NullBackend is a stub)
//! - that UI support is already part of the published `v1.1.1` line

use prom_ui_runtime::{
    Color, DesktopSession, DrawCommand, DrawFrame, EventBuffer, FrameToken, FrameTokenIssuer,
    InputEventKind, LoopControl, Rect, SessionState, UiBackendAdapter, UiRuntimeError,
    WindowConfig,
};

// ── NullBackend ───────────────────────────────────────────────────────────────

/// A no-op backend that records submitted draw frames for demo verification.
///
/// Used as the canonical demo backend until a real platform backend is wired
/// in Wave 3 final / Wave 4. No window is actually created.
struct NullBackend {
    frames_received: usize,
    last_clear_color: Option<Color>,
}

impl NullBackend {
    fn new() -> Self {
        Self {
            frames_received: 0,
            last_clear_color: None,
        }
    }
}

impl UiBackendAdapter for NullBackend {
    fn create_window(&mut self, config: &WindowConfig) -> Result<(), UiRuntimeError> {
        println!(
            "[NullBackend] create_window: title='{}', {}x{}",
            config.title, config.width, config.height
        );
        Ok(())
    }

    fn close_window(&mut self) {
        println!("[NullBackend] close_window");
    }

    fn run_event_loop<F: FnMut(LoopControl)>(
        &mut self,
        mut on_event: F,
    ) -> Result<(), UiRuntimeError> {
        // Simulate 3 frame ticks then stop.
        for tick in 0..3 {
            println!("[NullBackend] event loop tick {}", tick);
            on_event(LoopControl::Continue);
        }
        Ok(())
    }

    fn draw_frame(&mut self, frame: &DrawFrame) -> Result<(), UiRuntimeError> {
        self.frames_received += 1;
        println!(
            "[NullBackend] draw_frame #{}: {} command(s)",
            self.frames_received,
            frame.len()
        );
        for cmd in frame.commands() {
            match cmd {
                DrawCommand::Clear { color } => {
                    println!(
                        "  Clear {{ r:{} g:{} b:{} a:{} }}",
                        color.r, color.g, color.b, color.a
                    );
                    self.last_clear_color = Some(*color);
                }
                DrawCommand::FillRect { rect, color } => {
                    println!(
                        "  FillRect {{ x:{} y:{} w:{} h:{} }} color {{ r:{} g:{} b:{} }}",
                        rect.x, rect.y, rect.width, rect.height, color.r, color.g, color.b
                    );
                }
                DrawCommand::DrawText { text, x, y, color } => {
                    println!(
                        "  DrawText '{}' at ({},{}) color {{ r:{} g:{} b:{} }}",
                        text, x, y, color.r, color.g, color.b
                    );
                }
            }
        }
        Ok(())
    }
}

// ── Demo entry point ──────────────────────────────────────────────────────────

fn main() {
    println!("=== Semantic UI Application Boundary — Canonical Demo (M7 Wave 3) ===");
    println!("Backend: NullBackend (no-op stub; real backend wired in later wave)");
    println!();

    let config = WindowConfig::new("Semantic UI Demo", 800, 600);
    let backend = NullBackend::new();

    let mut session =
        DesktopSession::create(backend, config).expect("NullBackend::create_window must succeed");
    assert_eq!(session.state(), SessionState::Created);
    println!("Session state: {:?}", session.state());

    let mut issuer = FrameTokenIssuer::new();
    let mut frame_tokens: Vec<FrameToken> = Vec::new();

    session
        .run(|buf: &mut EventBuffer| {
            // Drain any pending events (none in the null backend, but the
            // contract is exercised).
            let events = buf.drain();
            for evt in &events {
                match &evt.kind {
                    InputEventKind::KeyDown { key_code } => {
                        println!("  Event: KeyDown({})", key_code);
                    }
                    InputEventKind::KeyUp { key_code } => {
                        println!("  Event: KeyUp({})", key_code);
                    }
                    InputEventKind::CloseRequested => {
                        println!("  Event: CloseRequested");
                        return LoopControl::ExitRequested;
                    }
                }
            }

            // Build the frame.
            let token = issuer.next();
            frame_tokens.push(token);

            let mut frame = DrawFrame::new();
            frame.clear(Color::rgb(30, 30, 30)); // dark background
            frame.fill_rect(
                Rect::new(50, 50, 200, 100),
                Color::rgb(70, 130, 180), // steel-blue panel
            );
            frame.draw_text(format!("Frame #{}", token.frame_id), 60, 90, Color::WHITE);

            // Submit the frame to the backend.
            // In a real backend this would block until vsync.
            println!("Submitting frame token #{}", token.frame_id);

            LoopControl::Continue
        })
        .expect("event loop must succeed");

    let _ = session.close();

    println!();
    println!("Session state after close: {:?}", session.state());
    println!("Total frames issued:       {}", frame_tokens.len());
    println!();

    // Verification assertions — this is a demo, not a test binary, but we
    // keep explicit checks so CI can run `prom-ui-demo` as a smoke test.
    assert_eq!(
        session.state(),
        SessionState::Closed,
        "session must be Closed after close()"
    );
    assert_eq!(frame_tokens.len(), 3, "NullBackend runs 3 ticks");
    assert!(
        frame_tokens[0].frame_id < frame_tokens[1].frame_id,
        "frame tokens must be monotone"
    );

    println!("All assertions passed. Demo complete.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_backend_demo_runs_without_panic() {
        let config = WindowConfig::new("Test Window", 640, 480);
        let backend = NullBackend::new();
        let mut session = DesktopSession::create(backend, config).expect("create must succeed");

        let mut frame_count = 0u32;
        session
            .run(|_buf| {
                frame_count += 1;
                LoopControl::Continue
            })
            .expect("run must succeed");

        let _ = session.close();
        assert_eq!(session.state(), SessionState::Closed);
        assert_eq!(frame_count, 3);
    }

    #[test]
    fn draw_frame_commands_are_submitted_to_null_backend() {
        let mut backend = NullBackend::new();
        let mut frame = DrawFrame::new();
        frame.clear(Color::BLACK);
        frame.fill_rect(Rect::new(0, 0, 100, 50), Color::BLUE);
        frame.draw_text("hello", 5, 5, Color::WHITE);

        backend.draw_frame(&frame).expect("draw_frame must succeed");
        assert_eq!(backend.frames_received, 1);
        assert_eq!(backend.last_clear_color, Some(Color::BLACK));
    }
}
