//! Desktop session ownership, event polling, and frame lifecycle for the
//! Semantic UI application boundary.
//!
//! This crate owns the runtime side of the first-wave UI boundary: desktop
//! session lifecycle, input event polling, frame token ownership, and the
//! backend adapter contract.
//!
//! # Current Wave Status
//!
//! Wave 3: minimal draw-command family (clear, filled rect, text label) and
//! backend adapter wiring. `DrawCommand`, `Color`, `Rect`, `DrawFrame`, and
//! the `draw_frame` method on `UiBackendAdapter` are the Wave 3 additions.
//! One canonical demo binary lives in `crates/prom-ui-demo`.
//!
//! # Backend Policy
//!
//! Backend selection is an internal implementation detail of this crate.
//! No backend library becomes a language-level promise in the first wave.
//! `UiBackendAdapter` is the only seam — no platform crate name crosses it.
//!
//! # Non-Commitments
//!
//! This crate does not claim:
//! - a specific graphics backend or wgpu fork
//! - multi-window, browser, or mobile support
//! - a widget/layout framework
//! - that UI runtime support is already part of the published `v1.1.1` line
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod boundary_registry;
pub mod visual_tokens;

use prom_ui::{UiCapabilityKind, UiOperationId};

// ── Error type ───────────────────────────────────────────────────────────────

/// Error type for UI runtime operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiRuntimeError {
    /// The required UI capability was not admitted for this session.
    CapabilityDenied(UiCapabilityKind),
    /// The requested UI operation is not yet admitted in this wave.
    OperationNotAdmitted(UiOperationId),
    /// The requested UI operation is invalid for the current lifecycle state.
    LifecycleViolation {
        operation: UiOperationId,
        state: SessionState,
    },
    /// The backend failed to create the window.
    WindowCreationFailed,
    /// The event loop terminated with a backend-level error.
    EventLoopFailed,
}

impl core::fmt::Display for UiRuntimeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            UiRuntimeError::CapabilityDenied(k) => {
                write!(f, "UI capability denied: {:?}", k)
            }
            UiRuntimeError::OperationNotAdmitted(op) => {
                write!(f, "UI operation not yet admitted: {:?}", op)
            }
            UiRuntimeError::LifecycleViolation { operation, state } => {
                write!(
                    f,
                    "UI lifecycle violation: {:?} is not valid in {:?}",
                    operation, state
                )
            }
            UiRuntimeError::WindowCreationFailed => {
                write!(f, "backend failed to create window")
            }
            UiRuntimeError::EventLoopFailed => {
                write!(f, "event loop terminated with backend error")
            }
        }
    }
}

// ── PR 5: Single-window session ownership and lifecycle API ───────────────────

/// Configuration for creating a single desktop window.
///
/// Passed to `UiBackendAdapter::create_window` at session creation.
/// Backend selection is an internal detail; this struct is the public
/// contract between the owner layer and whatever backend is wired in Wave 3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowConfig {
    pub title: alloc::string::String,
    pub width: u32,
    pub height: u32,
}

impl WindowConfig {
    pub fn new(title: impl Into<alloc::string::String>, width: u32, height: u32) -> Self {
        Self {
            title: title.into(),
            width,
            height,
        }
    }
}

/// Continuation signal returned by the per-frame callback.
///
/// `Continue` keeps the loop alive; `ExitRequested` drains the loop cleanly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopControl {
    Continue,
    ExitRequested,
}

/// Lifecycle state of a `DesktopSession`.
///
/// Transitions: `Created` → `Running` → `Closed`.
/// No backward transitions are permitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Created,
    Running,
    Closed,
}

/// Fail-closed lifecycle gate for admitted UI operations.
///
/// This gate is purely runtime-local: it checks whether a UI operation is
/// valid for the current session state and updates the state when admitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiLifecycleGate {
    state: SessionState,
}

impl UiLifecycleGate {
    /// Create a new lifecycle gate in the logical `Created` state.
    pub fn new() -> Self {
        Self {
            state: SessionState::Created,
        }
    }

    /// Current lifecycle state tracked by this gate.
    pub fn state(&self) -> SessionState {
        self.state
    }

    /// Check whether the given UI operation is admissible in the current state.
    pub fn admit(&self, operation: UiOperationId) -> Result<(), UiRuntimeError> {
        lifecycle_transition(self.state, operation).map(|_| ())
    }

    /// Check and apply the given UI operation, updating the lifecycle state.
    pub fn apply(&mut self, operation: UiOperationId) -> Result<(), UiRuntimeError> {
        let next = lifecycle_transition(self.state, operation)?;
        self.state = next;
        Ok(())
    }
}

fn lifecycle_transition(
    state: SessionState,
    operation: UiOperationId,
) -> Result<SessionState, UiRuntimeError> {
    match (state, operation) {
        (SessionState::Created, UiOperationId::WindowRun) => Ok(SessionState::Running),
        (SessionState::Created, UiOperationId::WindowClose) => Ok(SessionState::Closed),
        (SessionState::Running, UiOperationId::EventPoll) => Ok(SessionState::Running),
        (SessionState::Running, UiOperationId::FrameSubmit) => Ok(SessionState::Running),
        (SessionState::Running, UiOperationId::WindowClose) => Ok(SessionState::Closed),
        _ => Err(UiRuntimeError::LifecycleViolation { operation, state }),
    }
}

/// Internal contract a backend must implement to be driven by `DesktopSession`.
///
/// This trait is the backend seam — no platform crate name appears in the
/// public API. Actual backend implementations are wired in Wave 3.
pub trait UiBackendAdapter {
    fn create_window(&mut self, config: &WindowConfig) -> Result<(), UiRuntimeError>;
    fn close_window(&mut self);
    /// Drive the event/frame loop, calling `on_event` once per iteration.
    ///
    /// The backend controls iteration timing; the closure signals whether to
    /// continue. In Wave 3 the backend reconciles `LoopControl::ExitRequested`
    /// with platform-native close events.
    fn run_event_loop<F: FnMut(LoopControl)>(
        &mut self,
        on_event: F,
    ) -> Result<(), UiRuntimeError>;
    /// Submit a completed `DrawFrame` to the backend for rendering.
    ///
    /// Called at the end of each frame after the application callback has
    /// pushed draw commands. The backend is responsible for interpreting
    /// `DrawCommand` values and producing visible output.
    ///
    /// Default implementation is a no-op so Wave 2 backends remain valid.
    fn draw_frame(&mut self, _frame: &DrawFrame) -> Result<(), UiRuntimeError> {
        Ok(())
    }
}

/// Owner of a single desktop window session.
///
/// Wraps a `UiBackendAdapter`. Lifecycle: `create` → `run` → `close`.
///
/// `B` is the backend type. The concrete backend is supplied by the caller
/// through dependency injection, keeping platform crates out of the
/// public API surface.
pub struct DesktopSession<B: UiBackendAdapter> {
    backend: B,
    lifecycle: UiLifecycleGate,
}

impl<B: UiBackendAdapter> DesktopSession<B> {
    /// Create a new session, initialising the backend window.
    ///
    /// Returns the session in `SessionState::Created` on success.
    pub fn create(mut backend: B, config: WindowConfig) -> Result<Self, UiRuntimeError> {
        backend.create_window(&config)?;
        Ok(Self {
            backend,
            lifecycle: UiLifecycleGate::new(),
        })
    }

    /// Run the event/frame loop until the callback returns `LoopControl::ExitRequested`.
    ///
    /// Transitions the session to `SessionState::Running` on entry.
    /// The per-frame callback receives a mutable `EventBuffer`; events pushed
    /// into it during the frame are drained before the next iteration.
    ///
    /// NOTE: In Wave 2 the caller's `LoopControl` return governs termination
    /// via a captured flag. Wave 3 reconciles this with backend-native close
    /// events.
    pub fn run<F>(&mut self, mut on_frame: F) -> Result<(), UiRuntimeError>
    where
        F: FnMut(&mut EventBuffer) -> LoopControl,
    {
        self.lifecycle.apply(UiOperationId::WindowRun)?;
        let mut buffer = EventBuffer::new();
        let exit_requested = core::cell::Cell::new(false);
        let result = self.backend.run_event_loop(|_backend_control| {
            if exit_requested.get() {
                return;
            }
            let signal = on_frame(&mut buffer);
            let _ = buffer.drain();
            if signal == LoopControl::ExitRequested {
                exit_requested.set(true);
            }
        });
        result
    }

    /// Close the window and mark the session as closed.
    pub fn close(&mut self) -> Result<(), UiRuntimeError> {
        self.lifecycle.apply(UiOperationId::WindowClose)?;
        self.backend.close_window();
        Ok(())
    }

    /// Submit a completed draw frame through the lifecycle-gated session.
    pub fn submit_frame(&mut self, frame: &DrawFrame) -> Result<(), UiRuntimeError> {
        self.lifecycle.admit(UiOperationId::FrameSubmit)?;
        self.backend.draw_frame(frame)
    }

    /// Poll and drain input events through the lifecycle-gated session.
    pub fn poll_events(
        &mut self,
        buffer: &mut EventBuffer,
    ) -> Result<alloc::vec::Vec<InputEvent>, UiRuntimeError> {
        self.lifecycle.admit(UiOperationId::EventPoll)?;
        Ok(buffer.drain())
    }

    /// Run one lifecycle-gated UI frame tick.
    pub fn tick_frame<F>(
        &mut self,
        buffer: &mut EventBuffer,
        mut on_frame: F,
    ) -> Result<LoopControl, UiRuntimeError>
    where
        F: FnMut(&[InputEvent], &mut DrawFrame) -> LoopControl,
    {
        let events = self.poll_events(buffer)?;
        let mut frame = DrawFrame::new();
        let control = on_frame(&events, &mut frame);
        self.submit_frame(&frame)?;
        Ok(control)
    }

    /// Current lifecycle state of this session.
    pub fn state(&self) -> SessionState {
        self.lifecycle.state()
    }

    /// Read-only access to the backend for tests and reference inspection.
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// Mutable access to the backend for reference and test configuration.
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }
}

impl DesktopSession<InMemoryBackend> {
    /// Run one lifecycle-gated in-memory frame tick using queued backend events.
    pub fn tick_in_memory_frame<F>(&mut self, on_frame: F) -> Result<LoopControl, UiRuntimeError>
    where
        F: FnMut(&[InputEvent], &mut DrawFrame) -> LoopControl,
    {
        self.lifecycle.admit(UiOperationId::EventPoll)?;

        let queued = self.backend_mut().drain_events();
        let mut buffer = EventBuffer::new();

        for event in queued {
            buffer.push(event);
        }

        self.tick_frame(&mut buffer, on_frame)
    }
}

// ── PR 6: Deterministic event polling and frame-token ownership ───────────────

/// Taxonomy of first-wave input events admitted for polling.
///
/// Mouse, touch, and gamepad events are deferred to a future wave.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputEventKind {
    KeyDown { key_code: u32 },
    KeyUp { key_code: u32 },
    CloseRequested,
}

/// A single input event polled from the desktop session.
///
/// Replaces the Wave 0 inert marker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputEvent {
    pub kind: InputEventKind,
}

impl InputEvent {
    pub fn new(kind: InputEventKind) -> Self {
        Self { kind }
    }
}

/// Vec-backed accumulator for input events within a single frame.
///
/// The session's per-frame callback receives a `&mut EventBuffer`.
/// Events pushed by the backend (Wave 3) are drained each frame.
#[derive(Debug)]
pub struct EventBuffer {
    events: alloc::vec::Vec<InputEvent>,
}

impl EventBuffer {
    pub fn new() -> Self {
        Self {
            events: alloc::vec::Vec::new(),
        }
    }

    /// Push a single event into the buffer.
    pub fn push(&mut self, event: InputEvent) {
        self.events.push(event);
    }

    /// Drain all accumulated events, returning them as a `Vec`.
    ///
    /// After draining the buffer is empty.
    pub fn drain(&mut self) -> alloc::vec::Vec<InputEvent> {
        core::mem::replace(&mut self.events, alloc::vec::Vec::new())
    }

    /// Returns `true` if no events have been pushed since the last drain.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl Default for EventBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Token representing a single submitted draw frame.
///
/// Replaces the Wave 0 inert marker. Draw command submission semantics are
/// deferred to Wave 3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FrameToken {
    pub frame_id: u64,
}

/// Issues monotonically increasing `FrameToken` values.
///
/// A single issuer should be held per session. Frame IDs start at 0.
pub struct FrameTokenIssuer {
    next_id: u64,
}

impl FrameTokenIssuer {
    pub fn new() -> Self {
        Self { next_id: 0 }
    }

    /// Issue the next sequential `FrameToken`.
    pub fn next(&mut self) -> FrameToken {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        FrameToken { frame_id: id }
    }
}

impl Default for FrameTokenIssuer {
    fn default() -> Self {
        Self::new()
    }
}

// ── PR 7: Minimal draw-command family ─────────────────────────────────────────

/// An RGBA color value for use in draw commands.
///
/// All channels are in the range 0–255.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 255)
    }

    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const RED: Self = Self::rgb(255, 0, 0);
    pub const GREEN: Self = Self::rgb(0, 255, 0);
    pub const BLUE: Self = Self::rgb(0, 0, 255);
}

/// An axis-aligned integer rectangle in screen coordinates.
///
/// `x` and `y` are the top-left corner; `width` and `height` are in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }
}

/// A single draw command in the first-wave minimal drawing surface.
///
/// Three admitted forms: clear the surface, fill a rect, or draw a text label.
/// Additional draw command families (images, paths, etc.) are deferred to
/// future waves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrawCommand {
    /// Fill the entire surface with `color`.
    Clear { color: Color },
    /// Fill the axis-aligned `rect` with `color`.
    FillRect { rect: Rect, color: Color },
    /// Draw `text` at position (`x`, `y`) in `color`.
    ///
    /// Font selection, size, and layout are backend-determined in Wave 3.
    DrawText {
        text: alloc::string::String,
        x: i32,
        y: i32,
        color: Color,
    },
}

/// An ordered sequence of `DrawCommand` values for one frame.
///
/// Built by the application callback, then submitted to the backend via
/// `UiBackendAdapter::draw_frame`. Commands are applied in order.
#[derive(Debug, Default)]
pub struct DrawFrame {
    commands: alloc::vec::Vec<DrawCommand>,
}

impl DrawFrame {
    pub fn new() -> Self {
        Self {
            commands: alloc::vec::Vec::new(),
        }
    }

    /// Append a `DrawCommand` to this frame.
    pub fn push(&mut self, cmd: DrawCommand) {
        self.commands.push(cmd);
    }

    /// Convenience: clear the surface with `color`.
    pub fn clear(&mut self, color: Color) {
        self.push(DrawCommand::Clear { color });
    }

    /// Convenience: fill `rect` with `color`.
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.push(DrawCommand::FillRect { rect, color });
    }

    /// Convenience: draw `text` at (`x`, `y`) in `color`.
    pub fn draw_text(
        &mut self,
        text: impl Into<alloc::string::String>,
        x: i32,
        y: i32,
        color: Color,
    ) {
        self.push(DrawCommand::DrawText {
            text: text.into(),
            x,
            y,
            color,
        });
    }

    /// Read access to the accumulated commands (for backend consumption).
    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }

    /// Number of commands in this frame.
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Returns `true` if no commands have been pushed.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

/// Deterministic in-memory reference backend.
///
/// This backend does not create native windows and does not render pixels.
/// It records lifecycle calls and submitted draw frames for tests/reference use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InMemoryBackend {
    config: Option<WindowConfig>,
    run_event_loop_calls: usize,
    close_window_calls: usize,
    loop_iterations: usize,
    captured_frames: alloc::vec::Vec<alloc::vec::Vec<DrawCommand>>,
    queued_events: alloc::vec::Vec<InputEvent>,
}

impl InMemoryBackend {
    pub fn new() -> Self {
        Self::with_loop_iterations(1)
    }

    pub fn with_loop_iterations(loop_iterations: usize) -> Self {
        Self {
            config: None,
            run_event_loop_calls: 0,
            close_window_calls: 0,
            loop_iterations,
            captured_frames: alloc::vec::Vec::new(),
            queued_events: alloc::vec::Vec::new(),
        }
    }

    pub fn config(&self) -> Option<&WindowConfig> {
        self.config.as_ref()
    }

    pub fn run_event_loop_calls(&self) -> usize {
        self.run_event_loop_calls
    }

    pub fn close_window_calls(&self) -> usize {
        self.close_window_calls
    }

    pub fn captured_frames(&self) -> &[alloc::vec::Vec<DrawCommand>] {
        &self.captured_frames
    }

    pub fn captured_frame_count(&self) -> usize {
        self.captured_frames.len()
    }

    pub fn push_event(&mut self, event: InputEvent) {
        self.queued_events.push(event);
    }

    pub fn extend_events<I>(&mut self, events: I)
    where
        I: IntoIterator<Item = InputEvent>,
    {
        self.queued_events.extend(events);
    }

    pub fn queued_events(&self) -> &[InputEvent] {
        &self.queued_events
    }

    pub fn queued_event_count(&self) -> usize {
        self.queued_events.len()
    }

    pub fn drain_events(&mut self) -> alloc::vec::Vec<InputEvent> {
        core::mem::replace(&mut self.queued_events, alloc::vec::Vec::new())
    }
}

impl Default for InMemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl UiBackendAdapter for InMemoryBackend {
    fn create_window(&mut self, config: &WindowConfig) -> Result<(), UiRuntimeError> {
        self.config = Some(config.clone());
        Ok(())
    }

    fn close_window(&mut self) {
        self.close_window_calls = self.close_window_calls.saturating_add(1);
    }

    fn run_event_loop<F: FnMut(LoopControl)>(
        &mut self,
        mut on_event: F,
    ) -> Result<(), UiRuntimeError> {
        self.run_event_loop_calls = self.run_event_loop_calls.saturating_add(1);

        for _ in 0..self.loop_iterations {
            on_event(LoopControl::Continue);
        }

        Ok(())
    }

    fn draw_frame(&mut self, frame: &DrawFrame) -> Result<(), UiRuntimeError> {
        self.captured_frames.push(frame.commands().to_vec());
        Ok(())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::rc::Rc;
    use core::cell::Cell;

    struct CountingDrawBackend {
        submitted: Rc<Cell<usize>>,
        draw_error: Option<UiRuntimeError>,
    }

    impl CountingDrawBackend {
        fn new() -> Self {
            Self {
                submitted: Rc::new(Cell::new(0)),
                draw_error: None,
            }
        }

        fn with_error(draw_error: UiRuntimeError) -> Self {
            Self {
                submitted: Rc::new(Cell::new(0)),
                draw_error: Some(draw_error),
            }
        }
    }

    impl UiBackendAdapter for CountingDrawBackend {
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
            self.submitted.set(self.submitted.get() + 1);
            if let Some(err) = &self.draw_error {
                return Err(err.clone());
            }
            Ok(())
        }
    }

    fn all_ui_ops() -> [UiOperationId; 5] {
        [
            UiOperationId::WindowCreate,
            UiOperationId::WindowRun,
            UiOperationId::WindowClose,
            UiOperationId::EventPoll,
            UiOperationId::FrameSubmit,
        ]
    }

    #[test]
    fn window_config_holds_title_and_dimensions() {
        let cfg = WindowConfig::new("Hello Wave 2", 1280, 720);
        assert_eq!(cfg.title, "Hello Wave 2");
        assert_eq!(cfg.width, 1280);
        assert_eq!(cfg.height, 720);
    }

    #[test]
    fn loop_control_continue_and_exit_are_distinct() {
        assert_ne!(LoopControl::Continue, LoopControl::ExitRequested);
        let a = LoopControl::Continue;
        let _b = a; // Copy — no move
        assert_eq!(a, LoopControl::Continue);
    }

    #[test]
    fn event_buffer_push_and_drain_roundtrip() {
        let mut buf = EventBuffer::new();
        assert!(buf.is_empty());
        buf.push(InputEvent::new(InputEventKind::KeyDown { key_code: 65 }));
        buf.push(InputEvent::new(InputEventKind::KeyUp { key_code: 65 }));
        buf.push(InputEvent::new(InputEventKind::CloseRequested));
        assert!(!buf.is_empty());
        let drained = buf.drain();
        assert_eq!(drained.len(), 3);
        assert_eq!(drained[0].kind, InputEventKind::KeyDown { key_code: 65 });
        assert_eq!(drained[2].kind, InputEventKind::CloseRequested);
        assert!(buf.is_empty());
    }

    #[test]
    fn frame_token_issuer_issues_sequential_ids() {
        let mut issuer = FrameTokenIssuer::new();
        let t0 = issuer.next();
        let t1 = issuer.next();
        let t2 = issuer.next();
        assert_eq!(t0.frame_id, 0);
        assert_eq!(t1.frame_id, 1);
        assert_eq!(t2.frame_id, 2);
        assert!(t0 < t1);
        assert!(t1 < t2);
    }

    #[test]
    fn session_state_transitions_are_explicit() {
        let s = SessionState::Created;
        assert_ne!(s, SessionState::Running);
        assert_ne!(s, SessionState::Closed);
        assert_ne!(SessionState::Running, SessionState::Closed);
        let a = SessionState::Running;
        let _b = a; // Copy
        assert_eq!(a, SessionState::Running);
    }

    #[test]
    fn lifecycle_gate_starts_created() {
        let gate = UiLifecycleGate::new();
        assert_eq!(gate.state(), SessionState::Created);
        assert!(gate.admit(UiOperationId::WindowRun).is_ok());
    }

    #[test]
    fn window_run_transitions_created_to_running() {
        let mut gate = UiLifecycleGate::new();
        gate.apply(UiOperationId::WindowRun).unwrap();
        assert_eq!(gate.state(), SessionState::Running);
    }

    #[test]
    fn close_from_created_transitions_to_closed() {
        let mut gate = UiLifecycleGate::new();
        gate.apply(UiOperationId::WindowClose).unwrap();
        assert_eq!(gate.state(), SessionState::Closed);
    }

    #[test]
    fn event_poll_requires_running() {
        let mut gate = UiLifecycleGate::new();
        assert!(gate.apply(UiOperationId::EventPoll).is_err());
        gate.apply(UiOperationId::WindowRun).unwrap();
        assert!(gate.apply(UiOperationId::EventPoll).is_ok());
        assert_eq!(gate.state(), SessionState::Running);
    }

    #[test]
    fn frame_submit_requires_running() {
        let mut gate = UiLifecycleGate::new();
        assert!(gate.apply(UiOperationId::FrameSubmit).is_err());
        gate.apply(UiOperationId::WindowRun).unwrap();
        assert!(gate.apply(UiOperationId::FrameSubmit).is_ok());
        assert_eq!(gate.state(), SessionState::Running);
    }

    #[test]
    fn window_create_is_rejected_after_gate_exists() {
        let gate = UiLifecycleGate::new();
        assert!(gate.admit(UiOperationId::WindowCreate).is_err());

        let mut gate = UiLifecycleGate::new();
        assert!(gate.apply(UiOperationId::WindowCreate).is_err());
        assert_eq!(gate.state(), SessionState::Created);

        let mut running_gate = UiLifecycleGate::new();
        running_gate.apply(UiOperationId::WindowRun).unwrap();
        assert!(running_gate.apply(UiOperationId::WindowCreate).is_err());

        let mut closed_gate = UiLifecycleGate::new();
        closed_gate.apply(UiOperationId::WindowClose).unwrap();
        assert!(closed_gate.apply(UiOperationId::WindowCreate).is_err());
    }

    #[test]
    fn closed_state_rejects_all_ui_operations() {
        let mut gate = UiLifecycleGate::new();
        gate.apply(UiOperationId::WindowClose).unwrap();
        assert_eq!(gate.state(), SessionState::Closed);

        for op in all_ui_ops() {
            assert!(gate.apply(op).is_err(), "{op:?} should be rejected in Closed");
        }
    }

    #[test]
    fn lifecycle_violation_reports_operation_and_state() {
        let mut gate = UiLifecycleGate::new();
        let err = gate.apply(UiOperationId::EventPoll).unwrap_err();
        assert_eq!(
            err,
            UiRuntimeError::LifecycleViolation {
                operation: UiOperationId::EventPoll,
                state: SessionState::Created,
            }
        );
        assert_eq!(
            err.to_string(),
            "UI lifecycle violation: EventPoll is not valid in Created"
        );
    }

    #[test]
    fn desktop_session_lifecycle_via_mock_backend() {
        struct MockBackend {
            created: bool,
            closed: bool,
            loop_iters: usize,
        }
        impl UiBackendAdapter for MockBackend {
            fn create_window(&mut self, _config: &WindowConfig) -> Result<(), UiRuntimeError> {
                self.created = true;
                Ok(())
            }
            fn close_window(&mut self) {
                self.closed = true;
            }
            fn run_event_loop<F: FnMut(LoopControl)>(
                &mut self,
                mut on_event: F,
            ) -> Result<(), UiRuntimeError> {
                for _ in 0..self.loop_iters {
                    on_event(LoopControl::Continue);
                }
                Ok(())
            }
        }

        let backend = MockBackend {
            created: false,
            closed: false,
            loop_iters: 3,
        };
        let cfg = WindowConfig::new("Mock", 800, 600);
        let mut session =
            DesktopSession::create(backend, cfg).expect("mock backend create must succeed");
        assert_eq!(session.state(), SessionState::Created);

        let mut frame_count = 0u32;
        session
            .run(|_buf| {
                frame_count += 1;
                LoopControl::Continue
            })
            .expect("mock run must succeed");
        assert_eq!(session.state(), SessionState::Running);
        assert_eq!(frame_count, 3);

        session.close().expect("mock close must succeed");
        assert_eq!(session.state(), SessionState::Closed);
    }

    #[test]
    fn desktop_session_rejects_run_after_close() {
        struct MockBackend {
            closed: bool,
        }
        impl UiBackendAdapter for MockBackend {
            fn create_window(&mut self, _config: &WindowConfig) -> Result<(), UiRuntimeError> {
                Ok(())
            }
            fn close_window(&mut self) {
                self.closed = true;
            }
            fn run_event_loop<F: FnMut(LoopControl)>(
                &mut self,
                _on_event: F,
            ) -> Result<(), UiRuntimeError> {
                Ok(())
            }
        }

        let backend = MockBackend { closed: false };
        let cfg = WindowConfig::new("Mock", 800, 600);
        let mut session = DesktopSession::create(backend, cfg).unwrap();
        session.close().unwrap();

        let err = session
            .run(|_| LoopControl::Continue)
            .expect_err("run after close must fail");
        assert_eq!(
            err,
            UiRuntimeError::LifecycleViolation {
                operation: UiOperationId::WindowRun,
                state: SessionState::Closed,
            }
        );
    }

    #[test]
    fn desktop_session_rejects_double_run() {
        struct MockBackend;
        impl UiBackendAdapter for MockBackend {
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
        }

        let backend = MockBackend;
        let cfg = WindowConfig::new("Mock", 800, 600);
        let mut session = DesktopSession::create(backend, cfg).unwrap();

        session
            .run(|_| LoopControl::ExitRequested)
            .expect("first run must succeed");

        let err = session
            .run(|_| LoopControl::ExitRequested)
            .expect_err("second run must fail");
        assert_eq!(
            err,
            UiRuntimeError::LifecycleViolation {
                operation: UiOperationId::WindowRun,
                state: SessionState::Running,
            }
        );
    }

    #[test]
    fn desktop_session_close_is_fail_closed_after_closed() {
        struct MockBackend {
            closed: usize,
        }
        impl UiBackendAdapter for MockBackend {
            fn create_window(&mut self, _config: &WindowConfig) -> Result<(), UiRuntimeError> {
                Ok(())
            }
            fn close_window(&mut self) {
                self.closed = self.closed.saturating_add(1);
            }
            fn run_event_loop<F: FnMut(LoopControl)>(
                &mut self,
                _on_event: F,
            ) -> Result<(), UiRuntimeError> {
                Ok(())
            }
        }

        let backend = MockBackend { closed: 0 };
        let cfg = WindowConfig::new("Mock", 800, 600);
        let mut session = DesktopSession::create(backend, cfg).unwrap();

        session.close().unwrap();
        let err = session.close().expect_err("second close must fail");
        assert_eq!(
            err,
            UiRuntimeError::LifecycleViolation {
                operation: UiOperationId::WindowClose,
                state: SessionState::Closed,
            }
        );
    }

    #[test]
    fn desktop_session_submit_frame_requires_running() {
        let backend = CountingDrawBackend::new();
        let submitted = backend.submitted.clone();
        let cfg = WindowConfig::new("Mock", 800, 600);
        let mut session = DesktopSession::create(backend, cfg).unwrap();
        let frame = DrawFrame::new();

        let err = session
            .submit_frame(&frame)
            .expect_err("submit before run must fail");
        assert_eq!(
            err,
            UiRuntimeError::LifecycleViolation {
                operation: UiOperationId::FrameSubmit,
                state: SessionState::Created,
            }
        );
        assert_eq!(submitted.get(), 0);
    }

    #[test]
    fn desktop_session_submit_frame_succeeds_while_running() {
        let backend = CountingDrawBackend::new();
        let submitted = backend.submitted.clone();
        let cfg = WindowConfig::new("Mock", 800, 600);
        let mut session = DesktopSession::create(backend, cfg).unwrap();

        session
            .run(|_| LoopControl::ExitRequested)
            .expect("run must succeed");

        let mut frame = DrawFrame::new();
        frame.clear(Color::BLUE);

        session.submit_frame(&frame).unwrap();

        assert_eq!(submitted.get(), 1);
        assert_eq!(session.state(), SessionState::Running);
    }

    #[test]
    fn desktop_session_submit_frame_rejects_after_close() {
        let backend = CountingDrawBackend::new();
        let submitted = backend.submitted.clone();
        let cfg = WindowConfig::new("Mock", 800, 600);
        let mut session = DesktopSession::create(backend, cfg).unwrap();

        session
            .run(|_| LoopControl::ExitRequested)
            .expect("run must succeed");
        session.close().expect("close must succeed");

        let frame = DrawFrame::new();
        let err = session
            .submit_frame(&frame)
            .expect_err("submit after close must fail");
        assert_eq!(
            err,
            UiRuntimeError::LifecycleViolation {
                operation: UiOperationId::FrameSubmit,
                state: SessionState::Closed,
            }
        );
        assert_eq!(submitted.get(), 0);
    }

    #[test]
    fn desktop_session_submit_frame_propagates_backend_error() {
        let backend = CountingDrawBackend::with_error(UiRuntimeError::EventLoopFailed);
        let submitted = backend.submitted.clone();
        let cfg = WindowConfig::new("Mock", 800, 600);
        let mut session = DesktopSession::create(backend, cfg).unwrap();

        session
            .run(|_| LoopControl::ExitRequested)
            .expect("run must succeed");

        let frame = DrawFrame::new();
        let err = session
            .submit_frame(&frame)
            .expect_err("backend error must propagate");
        assert_eq!(err, UiRuntimeError::EventLoopFailed);
        assert_eq!(submitted.get(), 1);
        assert_eq!(session.state(), SessionState::Running);
    }

    #[test]
    fn desktop_session_poll_events_requires_running() {
        let backend = CountingDrawBackend::new();
        let cfg = WindowConfig::new("Mock", 800, 600);
        let mut session = DesktopSession::create(backend, cfg).unwrap();

        let mut buffer = EventBuffer::new();
        buffer.push(InputEvent::new(InputEventKind::KeyDown { key_code: 65 }));

        let err = session
            .poll_events(&mut buffer)
            .expect_err("poll before run must fail");
        assert_eq!(
            err,
            UiRuntimeError::LifecycleViolation {
                operation: UiOperationId::EventPoll,
                state: SessionState::Created,
            }
        );
        assert!(!buffer.is_empty());
    }

    #[test]
    fn desktop_session_poll_events_succeeds_while_running() {
        let backend = CountingDrawBackend::new();
        let cfg = WindowConfig::new("Mock", 800, 600);
        let mut session = DesktopSession::create(backend, cfg).unwrap();

        session
            .run(|_| LoopControl::ExitRequested)
            .expect("run must succeed");

        let mut buffer = EventBuffer::new();
        buffer.push(InputEvent::new(InputEventKind::KeyDown { key_code: 65 }));
        buffer.push(InputEvent::new(InputEventKind::KeyUp { key_code: 65 }));

        let events = session.poll_events(&mut buffer).unwrap();

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].kind, InputEventKind::KeyDown { key_code: 65 });
        assert_eq!(events[1].kind, InputEventKind::KeyUp { key_code: 65 });
        assert!(buffer.is_empty());
        assert_eq!(session.state(), SessionState::Running);
    }

    #[test]
    fn desktop_session_poll_events_rejects_after_close() {
        let backend = CountingDrawBackend::new();
        let cfg = WindowConfig::new("Mock", 800, 600);
        let mut session = DesktopSession::create(backend, cfg).unwrap();

        session
            .run(|_| LoopControl::ExitRequested)
            .expect("run must succeed");
        session.close().expect("close must succeed");

        let mut buffer = EventBuffer::new();
        buffer.push(InputEvent::new(InputEventKind::CloseRequested));

        let err = session
            .poll_events(&mut buffer)
            .expect_err("poll after close must fail");
        assert_eq!(
            err,
            UiRuntimeError::LifecycleViolation {
                operation: UiOperationId::EventPoll,
                state: SessionState::Closed,
            }
        );
        assert!(!buffer.is_empty());
    }

    #[test]
    fn desktop_session_poll_events_does_not_change_running_state() {
        let backend = CountingDrawBackend::new();
        let cfg = WindowConfig::new("Mock", 800, 600);
        let mut session = DesktopSession::create(backend, cfg).unwrap();

        session
            .run(|_| LoopControl::ExitRequested)
            .expect("run must succeed");

        let mut buffer = EventBuffer::new();
        let _ = session.poll_events(&mut buffer).unwrap();

        assert_eq!(session.state(), SessionState::Running);
    }

    #[test]
    fn desktop_session_tick_frame_requires_running() {
        let backend = CountingDrawBackend::new();
        let submitted = backend.submitted.clone();
        let cfg = WindowConfig::new("Mock", 800, 600);
        let mut session = DesktopSession::create(backend, cfg).unwrap();

        let mut buffer = EventBuffer::new();
        buffer.push(InputEvent::new(InputEventKind::KeyDown { key_code: 65 }));

        let mut callback_called = false;

        let err = session
            .tick_frame(&mut buffer, |_events, _frame| {
                callback_called = true;
                LoopControl::Continue
            })
            .expect_err("tick before run must fail");

        assert_eq!(
            err,
            UiRuntimeError::LifecycleViolation {
                operation: UiOperationId::EventPoll,
                state: SessionState::Created,
            }
        );
        assert!(!callback_called);
        assert!(!buffer.is_empty());
        assert_eq!(submitted.get(), 0);
    }

    #[test]
    fn desktop_session_tick_frame_drains_events_and_submits_frame() {
        let backend = CountingDrawBackend::new();
        let submitted = backend.submitted.clone();
        let cfg = WindowConfig::new("Mock", 800, 600);
        let mut session = DesktopSession::create(backend, cfg).unwrap();

        session
            .run(|_| LoopControl::ExitRequested)
            .expect("run must succeed");

        let mut buffer = EventBuffer::new();
        buffer.push(InputEvent::new(InputEventKind::KeyDown { key_code: 65 }));
        buffer.push(InputEvent::new(InputEventKind::KeyUp { key_code: 65 }));

        let control = session
            .tick_frame(&mut buffer, |events, frame| {
                assert_eq!(events.len(), 2);
                assert_eq!(events[0].kind, InputEventKind::KeyDown { key_code: 65 });
                assert_eq!(events[1].kind, InputEventKind::KeyUp { key_code: 65 });

                frame.clear(Color::BLACK);
                frame.fill_rect(Rect::new(0, 0, 10, 10), Color::GREEN);

                LoopControl::Continue
            })
            .expect("tick must succeed");

        assert_eq!(control, LoopControl::Continue);
        assert!(buffer.is_empty());
        assert_eq!(submitted.get(), 1);
        assert_eq!(session.state(), SessionState::Running);
    }

    #[test]
    fn desktop_session_tick_frame_rejects_after_close() {
        let backend = CountingDrawBackend::new();
        let submitted = backend.submitted.clone();
        let cfg = WindowConfig::new("Mock", 800, 600);
        let mut session = DesktopSession::create(backend, cfg).unwrap();

        session.run(|_| LoopControl::ExitRequested).unwrap();
        session.close().unwrap();

        let mut buffer = EventBuffer::new();
        buffer.push(InputEvent::new(InputEventKind::CloseRequested));

        let mut callback_called = false;

        let err = session
            .tick_frame(&mut buffer, |_events, _frame| {
                callback_called = true;
                LoopControl::ExitRequested
            })
            .expect_err("tick after close must fail");

        assert_eq!(
            err,
            UiRuntimeError::LifecycleViolation {
                operation: UiOperationId::EventPoll,
                state: SessionState::Closed,
            }
        );
        assert!(!callback_called);
        assert!(!buffer.is_empty());
        assert_eq!(submitted.get(), 0);
    }

    #[test]
    fn desktop_session_tick_frame_propagates_submit_error() {
        let backend = CountingDrawBackend::with_error(UiRuntimeError::EventLoopFailed);
        let submitted = backend.submitted.clone();
        let cfg = WindowConfig::new("Mock", 800, 600);
        let mut session = DesktopSession::create(backend, cfg).unwrap();

        session.run(|_| LoopControl::ExitRequested).unwrap();

        let mut buffer = EventBuffer::new();
        buffer.push(InputEvent::new(InputEventKind::KeyDown { key_code: 65 }));

        let err = session
            .tick_frame(&mut buffer, |_events, frame| {
                frame.clear(Color::BLUE);
                LoopControl::Continue
            })
            .expect_err("backend submit error must propagate");

        assert_eq!(err, UiRuntimeError::EventLoopFailed);
        assert_eq!(submitted.get(), 1);
        assert!(buffer.is_empty());
        assert_eq!(session.state(), SessionState::Running);
    }

    #[test]
    fn desktop_session_tick_frame_returns_callback_control() {
        let backend = CountingDrawBackend::new();
        let cfg = WindowConfig::new("Mock", 800, 600);
        let mut session = DesktopSession::create(backend, cfg).unwrap();

        session.run(|_| LoopControl::ExitRequested).unwrap();

        let mut buffer = EventBuffer::new();

        let control = session
            .tick_frame(&mut buffer, |_events, _frame| LoopControl::ExitRequested)
            .unwrap();

        assert_eq!(control, LoopControl::ExitRequested);
    }

    // ── Wave 3: draw-command family ───────────────────────────────────────────

    #[test]
    fn color_constants_and_constructors_are_correct() {
        assert_eq!(Color::BLACK, Color::rgb(0, 0, 0));
        assert_eq!(Color::WHITE, Color::rgb(255, 255, 255));
        let c = Color::rgba(10, 20, 30, 128);
        assert_eq!(c.r, 10);
        assert_eq!(c.g, 20);
        assert_eq!(c.b, 30);
        assert_eq!(c.a, 128);
        // rgb() fills alpha to 255
        assert_eq!(Color::RED.a, 255);
    }

    #[test]
    fn rect_constructor_holds_fields() {
        let r = Rect::new(10, 20, 100, 50);
        assert_eq!(r.x, 10);
        assert_eq!(r.y, 20);
        assert_eq!(r.width, 100);
        assert_eq!(r.height, 50);
    }

    #[test]
    fn draw_frame_push_and_commands_roundtrip() {
        let mut frame = DrawFrame::new();
        assert!(frame.is_empty());

        frame.clear(Color::BLACK);
        frame.fill_rect(Rect::new(0, 0, 100, 100), Color::RED);
        frame.draw_text("hello", 10, 10, Color::WHITE);

        assert_eq!(frame.len(), 3);
        assert!(!frame.is_empty());

        let cmds = frame.commands();
        assert_eq!(cmds[0], DrawCommand::Clear { color: Color::BLACK });
        assert_eq!(
            cmds[1],
            DrawCommand::FillRect {
                rect: Rect::new(0, 0, 100, 100),
                color: Color::RED,
            }
        );
        assert_eq!(
            cmds[2],
            DrawCommand::DrawText {
                text: "hello".into(),
                x: 10,
                y: 10,
                color: Color::WHITE,
            }
        );
    }

    #[test]
    fn draw_frame_submitted_to_mock_backend() {
        struct DrawCapturingBackend {
            captured: alloc::vec::Vec<DrawCommand>,
        }
        impl UiBackendAdapter for DrawCapturingBackend {
            fn create_window(&mut self, _: &WindowConfig) -> Result<(), UiRuntimeError> {
                Ok(())
            }
            fn close_window(&mut self) {}
            fn run_event_loop<F: FnMut(LoopControl)>(
                &mut self,
                mut on_event: F,
            ) -> Result<(), UiRuntimeError> {
                on_event(LoopControl::Continue);
                Ok(())
            }
            fn draw_frame(&mut self, frame: &DrawFrame) -> Result<(), UiRuntimeError> {
                self.captured.extend(frame.commands().iter().cloned());
                Ok(())
            }
        }

        let mut backend = DrawCapturingBackend { captured: alloc::vec::Vec::new() };

        let mut frame = DrawFrame::new();
        frame.clear(Color::BLUE);
        frame.fill_rect(Rect::new(5, 5, 10, 10), Color::GREEN);
        backend.draw_frame(&frame).expect("draw_frame must succeed");

        assert_eq!(backend.captured.len(), 2);
        assert_eq!(backend.captured[0], DrawCommand::Clear { color: Color::BLUE });
        assert_eq!(
            backend.captured[1],
            DrawCommand::FillRect {
                rect: Rect::new(5, 5, 10, 10),
                color: Color::GREEN,
            }
        );
    }
}
