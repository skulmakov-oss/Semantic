//! Native backend crate skeleton for Semantic UI.
//!
//! This crate is the future home of platform-specific UI backend code.
//! It intentionally does not create native windows yet.
//!
//! Boundary:
//! - `prom-ui-runtime` remains platform-neutral.
//! - Native/window dependencies belong here, not in `prom-ui-runtime`.
//! - The only current seam is `UiBackendAdapter`.

#![allow(clippy::mem_replace_with_default)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[allow(unused_imports)]
use prom_ui_runtime::{
    DrawFrame, InputEvent, InputEventKind, LoopControl, UiBackendAdapter, UiRuntimeError,
    WindowConfig,
};

pub mod action_translation;
pub mod draw_generation;
pub mod frame_sink;
pub mod interaction;
pub mod session_hook;

pub use action_translation::{DefaultNativeActionTranslator, NativeActionTranslator};
pub use frame_sink::{UiBackendFrame, UiBackendFrameEntry, UiFrameSink};
pub use interaction::RoutedInteraction;

/// Raw button state representing physical input evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawButtonState {
    Pressed,
    Released,
}

/// Raw keyboard key code representing physical input evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RawKeyCode {
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyW,
    KeyS,
    Digit0,
    Digit1,
    Digit2,
    Enter,
    Escape,
    Space,
    Unknown(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RawMouseButton {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u32),
}

/// Inert physical event evidence captured from the host.
///
/// This does not infer semantic intent.
#[derive(Debug, Clone, PartialEq)]
pub enum RawBackendEvent {
    WindowResized {
        width: u32,
        height: u32,
    },
    KeyboardInput {
        key: RawKeyCode,
        state: RawButtonState,
    },
    PointerMoved {
        x: f64,
        y: f64,
    },
    MouseInput {
        button: RawMouseButton,
        state: RawButtonState,
    },
    CloseRequested,
}

#[cfg(feature = "winit-backend")]
#[derive(Debug)]
pub enum NativeBackendWinitSmokeError {
    MissingWindowConfig,
    EventLoop(winit::error::EventLoopError),
}

#[cfg(feature = "winit-backend")]
impl From<winit::error::EventLoopError> for NativeBackendWinitSmokeError {
    fn from(err: winit::error::EventLoopError) -> Self {
        Self::EventLoop(err)
    }
}

/// Returns whether this crate was compiled with the `winit-backend` feature.
pub const fn winit_backend_feature_enabled() -> bool {
    #[cfg(feature = "winit-backend")]
    {
        true
    }
    #[cfg(not(feature = "winit-backend"))]
    {
        false
    }
}

/// Returns whether the `wgpu-backend` Cargo feature is enabled at compile time.
pub const fn wgpu_backend_feature_enabled() -> bool {
    #[cfg(feature = "wgpu-backend")]
    {
        true
    }
    #[cfg(not(feature = "wgpu-backend"))]
    {
        false
    }
}

pub const fn native_run_loop_safety_lock_available() -> bool {
    true
}

#[cfg(feature = "winit-backend")]
pub mod winit_placeholder {
    //! Feature-gated placeholder for future winit integration.
    //!
    //! This module intentionally does not create windows or run an event loop yet.

    use core::marker::PhantomData;

    use super::NativeBackend;
    use prom_ui_runtime::WindowConfig;
    use winit::error::OsError;
    use winit::{
        application::ApplicationHandler,
        dpi::LogicalSize,
        event::{ElementState, MouseButton, WindowEvent},
        event_loop::{ActiveEventLoop, EventLoop},
        keyboard::{KeyCode, PhysicalKey},
        window::{Window, WindowAttributes, WindowId},
    };

    /// Compile-time marker proving the `winit` crate is available behind the feature.
    pub const WINIT_BACKEND_PLACEHOLDER: bool = true;

    /// Returns whether the winit placeholder feature marker is available.
    pub const fn winit_backend_placeholder_enabled() -> bool {
        true
    }

    /// Returns a marker that anchors the current winit ApplicationHandler scaffold.
    pub const fn winit_event_loop_scaffold_available() -> bool {
        true
    }

    /// Returns whether the winit event-loop creation scaffold is available.
    pub const fn winit_event_loop_creation_available() -> bool {
        true
    }

    /// Create a real winit `EventLoop`.
    ///
    /// This does not run the loop and does not create a window.
    /// It only verifies that the native backend crate can construct the platform event loop
    /// behind the `winit-backend` feature.
    pub fn create_winit_event_loop() -> Result<EventLoop<()>, winit::error::EventLoopError> {
        let mut builder = EventLoop::builder();

        #[cfg(target_os = "windows")]
        {
            use winit::platform::windows::EventLoopBuilderExtWindows;

            builder.with_any_thread(true);
        }

        builder.build()
    }

    /// Returns whether the winit native window creation scaffold is available.
    pub const fn winit_window_creation_scaffold_available() -> bool {
        true
    }

    /// Translate Semantic UI `WindowConfig` into winit `WindowAttributes`.
    ///
    /// This function does not create a native window.
    /// It only prepares attributes for a future `ActiveEventLoop::create_window(...)` call.
    pub fn window_config_to_winit_attributes(config: &WindowConfig) -> WindowAttributes {
        Window::default_attributes()
            .with_title(config.title.clone())
            .with_inner_size(LogicalSize::new(config.width as f64, config.height as f64))
            .with_visible(true)
            .with_resizable(true)
    }

    /// Returns whether the winit window config translation scaffold is available.
    pub const fn winit_window_config_translation_available() -> bool {
        true
    }

    /// Create a native winit `Window` from Semantic `WindowConfig`.
    ///
    /// This must only be called from a valid winit event-loop lifecycle callback,
    /// such as `ApplicationHandler::resumed(...)`.
    ///
    /// This function does not run the event loop and does not render.
    pub fn create_winit_window_from_config(
        event_loop: &ActiveEventLoop,
        config: &WindowConfig,
    ) -> Result<Window, OsError> {
        let attributes = window_config_to_winit_attributes(config);
        event_loop.create_window(attributes)
    }

    /// Returns whether the winit event translation scaffold is available.
    pub const fn winit_event_translation_available() -> bool {
        true
    }

    /// Translate a winit close request into the Semantic UI input event surface.
    pub fn translate_winit_close_requested() -> prom_ui_runtime::InputEvent {
        prom_ui_runtime::InputEvent::new(prom_ui_runtime::InputEventKind::CloseRequested)
    }

    /// Translate a selected winit physical key into the current Semantic key code space.
    ///
    /// This is intentionally small. It is not a full keyboard layout or text-input model.
    pub const fn translate_winit_key_code(key_code: KeyCode) -> Option<u32> {
        match key_code {
            KeyCode::KeyA => Some(65),
            KeyCode::KeyB => Some(66),
            KeyCode::KeyC => Some(67),
            KeyCode::KeyD => Some(68),
            KeyCode::KeyW => Some(87),
            KeyCode::KeyS => Some(83),
            KeyCode::Digit0 => Some(48),
            KeyCode::Digit1 => Some(49),
            KeyCode::Digit2 => Some(50),
            KeyCode::Enter => Some(13),
            KeyCode::Escape => Some(27),
            KeyCode::Space => Some(32),
            _ => None,
        }
    }

    /// Translate a winit physical key state into a Semantic UI input event.
    ///
    /// Unsupported keys return `None`.
    pub fn translate_winit_physical_key(
        state: ElementState,
        physical_key: PhysicalKey,
    ) -> Option<prom_ui_runtime::InputEvent> {
        let key_code = match physical_key {
            PhysicalKey::Code(code) => translate_winit_key_code(code)?,
            PhysicalKey::Unidentified(_) => return None,
        };

        let kind = match state {
            ElementState::Pressed => prom_ui_runtime::InputEventKind::KeyDown { key_code },
            ElementState::Released => prom_ui_runtime::InputEventKind::KeyUp { key_code },
        };

        Some(prom_ui_runtime::InputEvent::new(kind))
    }

    /// Translate selected winit `WindowEvent` variants into the Semantic UI input surface.
    ///
    /// This function does not run an event loop and does not mutate backend state.
    pub fn translate_winit_window_event(
        event: &WindowEvent,
        scale_factor: f64,
    ) -> Option<prom_ui_runtime::InputEvent> {
        match event {
            WindowEvent::CloseRequested => Some(translate_winit_close_requested()),
            WindowEvent::KeyboardInput { event, .. } => {
                translate_winit_physical_key(event.state, event.physical_key)
            }
            WindowEvent::CursorMoved { position, .. } => {
                let logical = position.to_logical::<f64>(scale_factor);
                Some(prom_ui_runtime::InputEvent::new(
                    prom_ui_runtime::InputEventKind::PointerMoved {
                        x: logical.x,
                        y: logical.y,
                    },
                ))
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let btn_num = match button {
                    MouseButton::Left => 0,
                    MouseButton::Right => 1,
                    MouseButton::Middle => 2,
                    MouseButton::Back => 3,
                    MouseButton::Forward => 4,
                    MouseButton::Other(n) => *n as u32,
                };
                let kind = match state {
                    ElementState::Pressed => {
                        prom_ui_runtime::InputEventKind::PointerDown { button: btn_num }
                    }
                    ElementState::Released => {
                        prom_ui_runtime::InputEventKind::PointerUp { button: btn_num }
                    }
                };
                Some(prom_ui_runtime::InputEvent::new(kind))
            }
            _ => None,
        }
    }

    /// Translate a selected winit physical key into `RawKeyCode`.
    pub const fn translate_winit_to_raw_key_code(key_code: KeyCode) -> super::RawKeyCode {
        match key_code {
            KeyCode::KeyA => super::RawKeyCode::KeyA,
            KeyCode::KeyB => super::RawKeyCode::KeyB,
            KeyCode::KeyC => super::RawKeyCode::KeyC,
            KeyCode::KeyD => super::RawKeyCode::KeyD,
            KeyCode::KeyW => super::RawKeyCode::KeyW,
            KeyCode::KeyS => super::RawKeyCode::KeyS,
            KeyCode::Digit0 => super::RawKeyCode::Digit0,
            KeyCode::Digit1 => super::RawKeyCode::Digit1,
            KeyCode::Digit2 => super::RawKeyCode::Digit2,
            KeyCode::Enter => super::RawKeyCode::Enter,
            KeyCode::Escape => super::RawKeyCode::Escape,
            KeyCode::Space => super::RawKeyCode::Space,
            _ => super::RawKeyCode::Unknown(key_code as u32),
        }
    }

    /// Translate a winit `ElementState` to `RawButtonState`.
    pub const fn translate_winit_to_raw_button_state(state: ElementState) -> super::RawButtonState {
        match state {
            ElementState::Pressed => super::RawButtonState::Pressed,
            ElementState::Released => super::RawButtonState::Released,
        }
    }

    /// Translate a winit `WindowEvent` into inert `RawBackendEvent` evidence.
    pub fn translate_winit_to_raw_backend_event(
        event: &WindowEvent,
    ) -> Option<super::RawBackendEvent> {
        match event {
            WindowEvent::CloseRequested => Some(super::RawBackendEvent::CloseRequested),
            WindowEvent::Resized(physical_size) => Some(super::RawBackendEvent::WindowResized {
                width: physical_size.width,
                height: physical_size.height,
            }),
            WindowEvent::KeyboardInput { event, .. } => {
                let key_code = match event.physical_key {
                    PhysicalKey::Code(code) => translate_winit_to_raw_key_code(code),
                    PhysicalKey::Unidentified(_) => return None,
                };
                let state = translate_winit_to_raw_button_state(event.state);
                Some(super::RawBackendEvent::KeyboardInput {
                    key: key_code,
                    state,
                })
            }
            WindowEvent::CursorMoved { position, .. } => {
                Some(super::RawBackendEvent::PointerMoved {
                    x: position.x,
                    y: position.y,
                })
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let button = match button {
                    winit::event::MouseButton::Left => super::RawMouseButton::Left,
                    winit::event::MouseButton::Right => super::RawMouseButton::Right,
                    winit::event::MouseButton::Middle => super::RawMouseButton::Middle,
                    winit::event::MouseButton::Back => super::RawMouseButton::Back,
                    winit::event::MouseButton::Forward => super::RawMouseButton::Forward,
                    winit::event::MouseButton::Other(n) => super::RawMouseButton::Other(*n as u32),
                };
                let state = translate_winit_to_raw_button_state(*state);
                Some(super::RawBackendEvent::MouseInput { button, state })
            }
            _ => None,
        }
    }

    /// Type-level scaffold for future winit event-loop integration.
    ///
    /// This struct intentionally owns no `EventLoop`, no `Window`, and no OS handle.
    /// It only proves that the native backend crate can compile against the
    /// `winit` 0.30 ApplicationHandler surface behind the `winit-backend` feature.
    #[derive(Debug, Default)]
    pub struct WinitEventLoopScaffold {
        resumed_calls: usize,
        window_event_calls: usize,
        close_requested: bool,
        _not_send_sync_runtime: PhantomData<*const ()>,
    }

    impl WinitEventLoopScaffold {
        pub const fn new() -> Self {
            Self {
                resumed_calls: 0,
                window_event_calls: 0,
                close_requested: false,
                _not_send_sync_runtime: PhantomData,
            }
        }

        pub const fn resumed_calls(&self) -> usize {
            self.resumed_calls
        }

        pub const fn window_event_calls(&self) -> usize {
            self.window_event_calls
        }

        pub const fn close_requested(&self) -> bool {
            self.close_requested
        }
    }

    impl ApplicationHandler for WinitEventLoopScaffold {
        fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
            self.resumed_calls = self.resumed_calls.saturating_add(1);
        }

        fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            _window_id: WindowId,
            event: WindowEvent,
        ) {
            self.window_event_calls = self.window_event_calls.saturating_add(1);

            if matches!(event, WindowEvent::CloseRequested) {
                self.close_requested = true;
                event_loop.exit();
            }
        }
    }

    /// ApplicationHandler scaffold that creates one native window on `resumed(...)`.
    ///
    /// This is a controlled native-window creation scaffold.
    /// It does not render, does not submit frames, and does not integrate with
    /// `NativeBackend::run_event_loop(...)` yet.
    #[derive(Debug)]
    pub struct WinitWindowCreationScaffold {
        config: WindowConfig,
        window_id: Option<WindowId>,
        window: Option<Window>,
        create_attempts: usize,
        create_failures: usize,
    }

    impl WinitWindowCreationScaffold {
        pub fn new(config: WindowConfig) -> Self {
            Self {
                config,
                window_id: None,
                window: None,
                create_attempts: 0,
                create_failures: 0,
            }
        }

        pub fn config(&self) -> &WindowConfig {
            &self.config
        }

        pub const fn window_id(&self) -> Option<WindowId> {
            self.window_id
        }

        pub const fn has_window(&self) -> bool {
            self.window_id.is_some()
        }

        pub const fn create_attempts(&self) -> usize {
            self.create_attempts
        }

        pub const fn create_failures(&self) -> usize {
            self.create_failures
        }
    }

    impl ApplicationHandler for WinitWindowCreationScaffold {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            if self.window.is_some() {
                return;
            }

            self.create_attempts = self.create_attempts.saturating_add(1);

            match create_winit_window_from_config(event_loop, &self.config) {
                Ok(window) => {
                    self.window_id = Some(window.id());
                    self.window = Some(window);
                }
                Err(_err) => {
                    self.create_failures = self.create_failures.saturating_add(1);
                    event_loop.exit();
                }
            }
        }

        fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            window_id: WindowId,
            event: WindowEvent,
        ) {
            if Some(window_id) != self.window_id {
                return;
            }

            if matches!(event, WindowEvent::CloseRequested) {
                event_loop.exit();
            }
        }
    }

    /// Returns whether the native run_app smoke scaffold is available.
    pub const fn winit_run_app_smoke_available() -> bool {
        true
    }

    /// Returns whether the NativeBackend winit app-state run_app helper is available.
    pub const fn native_backend_winit_app_state_run_app_available() -> bool {
        true
    }

    /// Result summary for the native run_app smoke path.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct WinitRunAppSmokeResult {
        pub resumed_calls: usize,
        pub create_attempts: usize,
        pub create_failures: usize,
        pub window_created: bool,
    }

    /// Error returned by the manual NativeBackend app-state run_app helper.
    #[derive(Debug)]
    pub enum NativeBackendWinitAppStateRunError {
        MissingWindowConfig,
        EventLoop(winit::error::EventLoopError),
    }

    impl From<winit::error::EventLoopError> for NativeBackendWinitAppStateRunError {
        fn from(err: winit::error::EventLoopError) -> Self {
            Self::EventLoop(err)
        }
    }

    /// Manual smoke scaffold for `EventLoop::run_app(...)`.
    ///
    /// This creates one native window on `resumed(...)` and exits immediately.
    /// It is intended for manual smoke tests only.
    #[derive(Debug)]
    pub struct WinitRunAppSmokeScaffold {
        config: WindowConfig,
        resumed_calls: usize,
        create_attempts: usize,
        create_failures: usize,
        window_created: bool,
        window_id: Option<WindowId>,
        window: Option<Window>,
    }

    impl WinitRunAppSmokeScaffold {
        pub fn new(config: WindowConfig) -> Self {
            Self {
                config,
                resumed_calls: 0,
                create_attempts: 0,
                create_failures: 0,
                window_created: false,
                window_id: None,
                window: None,
            }
        }

        pub fn result(&self) -> WinitRunAppSmokeResult {
            WinitRunAppSmokeResult {
                resumed_calls: self.resumed_calls,
                create_attempts: self.create_attempts,
                create_failures: self.create_failures,
                window_created: self.window_created,
            }
        }

        pub const fn has_window(&self) -> bool {
            self.window_id.is_some()
        }
    }

    impl ApplicationHandler for WinitRunAppSmokeScaffold {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            self.resumed_calls = self.resumed_calls.saturating_add(1);

            if self.window.is_some() {
                event_loop.exit();
                return;
            }

            self.create_attempts = self.create_attempts.saturating_add(1);

            match create_winit_window_from_config(event_loop, &self.config) {
                Ok(window) => {
                    self.window_id = Some(window.id());
                    self.window = Some(window);
                    self.window_created = true;
                    event_loop.exit();
                }
                Err(_err) => {
                    self.create_failures = self.create_failures.saturating_add(1);
                    event_loop.exit();
                }
            }
        }

        fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            window_id: WindowId,
            event: WindowEvent,
        ) {
            if Some(window_id) != self.window_id {
                return;
            }

            if matches!(event, WindowEvent::CloseRequested) {
                event_loop.exit();
            }
        }
    }

    fn ensure_native_backend_has_window_config(
        backend: &NativeBackend,
    ) -> Result<(), NativeBackendWinitAppStateRunError> {
        if backend.window_config().is_some() {
            Ok(())
        } else {
            Err(NativeBackendWinitAppStateRunError::MissingWindowConfig)
        }
    }

    /// Run a manual native smoke using winit `EventLoop::run_app(...)`.
    ///
    /// This creates a real event loop and attempts to create one native window,
    /// then exits immediately.
    ///
    /// This function is not used by `NativeBackend::run_event_loop(...)`.
    pub fn run_winit_window_creation_smoke(
        config: WindowConfig,
    ) -> Result<WinitRunAppSmokeResult, winit::error::EventLoopError> {
        let event_loop = create_winit_event_loop()?;
        let mut app = WinitRunAppSmokeScaffold::new(config);

        event_loop.run_app(&mut app)?;

        Ok(app.result())
    }

    /// Run a manual native winit app loop using `NativeBackendWinitAppState`.
    ///
    /// This opens a real native window and returns after the window is closed.
    /// It is intended for ignored/manual smoke tests only.
    ///
    /// This does not modify `NativeBackend::run_event_loop(...)`.
    pub fn run_native_backend_winit_app_state_until_close(
        backend: NativeBackend,
    ) -> Result<NativeBackendWinitAppStateSummary, NativeBackendWinitAppStateRunError> {
        ensure_native_backend_has_window_config(&backend)?;

        let event_loop = create_winit_event_loop()?;
        let mut app = NativeBackendWinitAppState::new(backend);

        event_loop.run_app(&mut app)?;

        Ok(app.summary())
    }

    /// Returns whether the NativeBackend winit app state scaffold is available.
    pub const fn native_backend_winit_app_state_available() -> bool {
        true
    }

    /// Returns whether the controlled NativeBackend winit run-loop integration plan is available.
    pub const fn native_backend_winit_run_loop_plan_available() -> bool {
        true
    }

    /// Current planned stage for NativeBackend/winit run-loop integration.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum NativeBackendWinitRunLoopStage {
        /// Existing manual smoke path.
        ManualSmoke,

        /// Existing NativeBackendWinitAppState manual run_app path.
        ManualAppStateRun,

        /// Adapter is integrated with `NativeBackend::run_event_loop`.
        AdapterIntegrated,
    }

    /// Ownership model for the planned NativeBackend/winit run-loop path.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum NativeBackendWinitRunLoopOwnership {
        /// Manual helper consumes NativeBackend and returns summary only.
        ConsumesBackendReturnsSummary,

        /// NativeBackend temporarily hosts the winit EventLoop in run_event_loop.
        HostedInsideBackendRunEventLoop,
    }

    /// Preflight readiness for running the NativeBackend-backed winit app state.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum NativeBackendWinitRunLoopReadiness {
        MissingWindowConfig,
        ReadyForManualAppStateRun,
    }

    /// Controlled integration plan for NativeBackend-backed winit run-loop support.
    ///
    /// This is a code-level contract, not a runtime integration.
    /// It records the currently admitted boundary before `NativeBackend::run_event_loop(...)`
    /// is allowed to touch winit.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NativeBackendWinitRunLoopIntegrationPlan {
        pub stage: NativeBackendWinitRunLoopStage,
        pub ownership: NativeBackendWinitRunLoopOwnership,

        pub requires_staged_window_config: bool,
        pub uses_native_backend_app_state: bool,
        pub creates_event_loop: bool,
        pub may_create_native_window: bool,
        pub may_call_run_app: bool,

        pub integrates_with_backend_run_event_loop: bool,
        pub mutates_prom_ui_runtime: bool,
        pub changes_ui_backend_adapter: bool,
        pub includes_renderer: bool,
        pub presents_frames: bool,
    }

    impl NativeBackendWinitRunLoopIntegrationPlan {
        /// Current G16-admitted plan.
        ///
        /// This reflects the existing manual `NativeBackendWinitAppState` path:
        /// it can create an EventLoop, create one native window, and call run_app
        /// only through manual/ignored smoke helpers.
        ///
        /// It does not admit normal runtime integration yet.
        pub const fn current_manual_app_state_plan() -> Self {
            Self {
                stage: NativeBackendWinitRunLoopStage::ManualAppStateRun,
                ownership: NativeBackendWinitRunLoopOwnership::ConsumesBackendReturnsSummary,

                requires_staged_window_config: true,
                uses_native_backend_app_state: true,
                creates_event_loop: true,
                may_create_native_window: true,
                may_call_run_app: true,

                integrates_with_backend_run_event_loop: false,
                mutates_prom_ui_runtime: false,
                changes_ui_backend_adapter: false,
                includes_renderer: false,
                presents_frames: false,
            }
        }

        /// Current integrated plan.
        pub const fn current_integrated_plan() -> Self {
            Self {
                stage: NativeBackendWinitRunLoopStage::AdapterIntegrated,
                ownership: NativeBackendWinitRunLoopOwnership::HostedInsideBackendRunEventLoop,

                requires_staged_window_config: true,
                uses_native_backend_app_state: true,
                creates_event_loop: true,
                may_create_native_window: true,
                may_call_run_app: true,

                integrates_with_backend_run_event_loop: true,
                mutates_prom_ui_runtime: false,
                changes_ui_backend_adapter: false,
                includes_renderer: false,
                presents_frames: false,
            }
        }

        pub const fn keeps_runtime_boundary_clean(&self) -> bool {
            !self.mutates_prom_ui_runtime && !self.changes_ui_backend_adapter
        }

        pub const fn keeps_rendering_out_of_scope(&self) -> bool {
            !self.includes_renderer && !self.presents_frames
        }

        pub const fn is_g16_admissible(&self) -> bool {
            self.requires_staged_window_config
                && self.uses_native_backend_app_state
                && self.creates_event_loop
                && self.may_create_native_window
                && self.may_call_run_app
                && self.keeps_runtime_boundary_clean()
                && self.keeps_rendering_out_of_scope()
        }
    }

    /// Return the current controlled integration plan.
    pub const fn current_native_backend_winit_run_loop_plan(
    ) -> NativeBackendWinitRunLoopIntegrationPlan {
        NativeBackendWinitRunLoopIntegrationPlan::current_manual_app_state_plan()
    }

    /// Preflight the staged NativeBackend state for the current manual app-state run path.
    ///
    /// This does not create an EventLoop, does not create a Window, and does not call run_app.
    pub fn native_backend_winit_run_loop_readiness(
        backend: &NativeBackend,
    ) -> NativeBackendWinitRunLoopReadiness {
        if backend.window_config().is_some() {
            NativeBackendWinitRunLoopReadiness::ReadyForManualAppStateRun
        } else {
            NativeBackendWinitRunLoopReadiness::MissingWindowConfig
        }
    }

    /// Returns whether the separate NativeBackend winit app facade is available.
    pub const fn native_backend_winit_app_facade_available() -> bool {
        true
    }

    /// Returns whether the NativeBackend winit app facade transcript contract is available.
    pub const fn native_backend_winit_app_facade_transcript_available() -> bool {
        true
    }

    /// Returns whether the NativeBackend winit app event transcript extraction is available.
    pub const fn native_backend_winit_app_event_transcript_available() -> bool {
        true
    }

    /// Returns whether the NativeBackend winit app draw staging bridge is available.
    pub const fn native_backend_winit_app_draw_staging_available() -> bool {
        true
    }

    /// Returns whether the combined facade run transcript with draw facts is available.
    pub const fn native_backend_winit_app_facade_draw_run_transcript_available() -> bool {
        true
    }

    /// Error returned by the separate NativeBackend winit app facade.
    #[derive(Debug)]
    pub enum NativeBackendWinitAppError {
        MissingWindowConfig,
        EventLoop(winit::error::EventLoopError),
    }

    impl From<winit::error::EventLoopError> for NativeBackendWinitAppError {
        fn from(err: winit::error::EventLoopError) -> Self {
            Self::EventLoop(err)
        }
    }

    /// High-level status of the NativeBackend winit facade run transcript.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum NativeBackendWinitAppRunTranscriptStatus {
        /// Facade exists but native run has not been executed.
        Planned,

        /// Native run completed and returned a summary.
        Completed,

        /// Native run was attempted but failed before a completed summary.
        Failed,
    }

    /// Transcript of the NativeBackend winit app facade run path.
    ///
    /// This is a compact contract object, not a full event log.
    /// It captures the externally relevant milestones of the native facade path.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NativeBackendWinitAppRunTranscript {
        pub status: NativeBackendWinitAppRunTranscriptStatus,

        pub staged_window_config: bool,
        pub event_loop_requested: bool,
        pub app_state_created: bool,
        pub run_app_requested: bool,

        pub resumed_calls: usize,
        pub window_event_calls: usize,
        pub create_attempts: usize,
        pub create_failures: usize,
        pub window_created: bool,
        pub close_requested: bool,
        pub staged_event_count: usize,
    }

    impl NativeBackendWinitAppRunTranscript {
        pub const fn planned(staged_window_config: bool) -> Self {
            Self {
                status: NativeBackendWinitAppRunTranscriptStatus::Planned,

                staged_window_config,
                event_loop_requested: false,
                app_state_created: false,
                run_app_requested: false,

                resumed_calls: 0,
                window_event_calls: 0,
                create_attempts: 0,
                create_failures: 0,
                window_created: false,
                close_requested: false,
                staged_event_count: 0,
            }
        }

        pub const fn completed_from_summary(summary: NativeBackendWinitAppStateSummary) -> Self {
            Self {
                status: NativeBackendWinitAppRunTranscriptStatus::Completed,

                staged_window_config: true,
                event_loop_requested: true,
                app_state_created: true,
                run_app_requested: true,

                resumed_calls: summary.resumed_calls,
                window_event_calls: summary.window_event_calls,
                create_attempts: summary.create_attempts,
                create_failures: summary.create_failures,
                window_created: summary.window_created,
                close_requested: summary.close_requested,
                staged_event_count: summary.staged_event_count,
            }
        }

        pub const fn failed_after_request(staged_window_config: bool) -> Self {
            Self {
                status: NativeBackendWinitAppRunTranscriptStatus::Failed,

                staged_window_config,
                event_loop_requested: staged_window_config,
                app_state_created: false,
                run_app_requested: false,

                resumed_calls: 0,
                window_event_calls: 0,
                create_attempts: 0,
                create_failures: 0,
                window_created: false,
                close_requested: false,
                staged_event_count: 0,
            }
        }

        pub fn completed_cleanly(&self) -> bool {
            matches!(
                self.status,
                NativeBackendWinitAppRunTranscriptStatus::Completed
            ) && self.staged_window_config
                && self.event_loop_requested
                && self.app_state_created
                && self.run_app_requested
                && self.window_created
                && self.create_failures == 0
        }
    }

    /// Derived event status for the NativeBackend winit app facade path.
    ///
    /// This is derived from summary counters, not from a full raw event log.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum NativeBackendWinitAppEventTranscriptStatus {
        NoWindowEventsObserved,
        WindowEventsObserved,
        CloseObserved,
    }

    /// Derived event transcript extracted from facade summary/transcript.
    ///
    /// This is not a raw event log. It records event-level facts available from
    /// `NativeBackendWinitAppStateSummary`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NativeBackendWinitAppEventTranscript {
        pub status: NativeBackendWinitAppEventTranscriptStatus,
        pub window_event_calls: usize,
        pub staged_event_count: usize,
        pub close_observed: bool,
    }

    impl NativeBackendWinitAppEventTranscript {
        pub const fn from_summary(summary: NativeBackendWinitAppStateSummary) -> Self {
            let status = if summary.close_requested {
                NativeBackendWinitAppEventTranscriptStatus::CloseObserved
            } else if summary.window_event_calls > 0 || summary.staged_event_count > 0 {
                NativeBackendWinitAppEventTranscriptStatus::WindowEventsObserved
            } else {
                NativeBackendWinitAppEventTranscriptStatus::NoWindowEventsObserved
            };

            Self {
                status,
                window_event_calls: summary.window_event_calls,
                staged_event_count: summary.staged_event_count,
                close_observed: summary.close_requested,
            }
        }

        pub const fn from_run_transcript(transcript: NativeBackendWinitAppRunTranscript) -> Self {
            let status = if transcript.close_requested {
                NativeBackendWinitAppEventTranscriptStatus::CloseObserved
            } else if transcript.window_event_calls > 0 || transcript.staged_event_count > 0 {
                NativeBackendWinitAppEventTranscriptStatus::WindowEventsObserved
            } else {
                NativeBackendWinitAppEventTranscriptStatus::NoWindowEventsObserved
            };

            Self {
                status,
                window_event_calls: transcript.window_event_calls,
                staged_event_count: transcript.staged_event_count,
                close_observed: transcript.close_requested,
            }
        }

        pub const fn observed_any_event(&self) -> bool {
            self.window_event_calls > 0 || self.staged_event_count > 0 || self.close_observed
        }

        pub const fn observed_close(&self) -> bool {
            self.close_observed
        }
    }

    /// Status of a draw-frame staging operation.
    ///
    /// This is not renderer status. It only describes backend-side staging/accounting.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum NativeBackendWinitAppDrawStagingStatus {
        NotSubmitted,
        Submitted,
    }

    /// Transcript for staged draw-frame submission through the native facade path.
    ///
    /// This does not imply rendering or presentation. It only records that a
    /// `DrawFrame` was accepted into backend-side accounting.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NativeBackendWinitAppDrawTranscript {
        pub status: NativeBackendWinitAppDrawStagingStatus,
        pub submitted_before: usize,
        pub submitted_after: usize,
        pub submitted_delta: usize,
    }

    impl NativeBackendWinitAppDrawTranscript {
        pub const fn not_submitted(current_submitted_frames: usize) -> Self {
            Self {
                status: NativeBackendWinitAppDrawStagingStatus::NotSubmitted,
                submitted_before: current_submitted_frames,
                submitted_after: current_submitted_frames,
                submitted_delta: 0,
            }
        }

        pub const fn submitted(before: usize, after: usize) -> Self {
            Self {
                status: NativeBackendWinitAppDrawStagingStatus::Submitted,
                submitted_before: before,
                submitted_after: after,
                submitted_delta: after.saturating_sub(before),
            }
        }

        pub fn accepted_one_frame(&self) -> bool {
            matches!(
                self.status,
                NativeBackendWinitAppDrawStagingStatus::Submitted
            ) && self.submitted_delta == 1
                && self.submitted_after == self.submitted_before.saturating_add(1)
        }
    }

    /// High-level status for the combined facade transcript.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum NativeBackendWinitAppFacadeTranscriptStatus {
        Planned,
        Completed,
        Failed,
    }

    /// Combined transcript for the NativeBackend winit app facade.
    ///
    /// This combines run, event, and draw-staging facts.
    /// It does not imply rendering or frame presentation.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NativeBackendWinitAppFacadeTranscript {
        pub status: NativeBackendWinitAppFacadeTranscriptStatus,

        pub run: NativeBackendWinitAppRunTranscript,
        pub events: NativeBackendWinitAppEventTranscript,
        pub draw: NativeBackendWinitAppDrawTranscript,

        pub submitted_frames_before_run: usize,
        pub renderer_used: bool,
        pub frame_presented: bool,
    }

    impl NativeBackendWinitAppFacadeTranscript {
        pub const fn planned(
            run: NativeBackendWinitAppRunTranscript,
            draw: NativeBackendWinitAppDrawTranscript,
        ) -> Self {
            Self {
                status: NativeBackendWinitAppFacadeTranscriptStatus::Planned,
                events: NativeBackendWinitAppEventTranscript {
                    status: NativeBackendWinitAppEventTranscriptStatus::NoWindowEventsObserved,
                    window_event_calls: 0,
                    staged_event_count: 0,
                    close_observed: false,
                },
                submitted_frames_before_run: draw.submitted_after,
                run,
                draw,
                renderer_used: false,
                frame_presented: false,
            }
        }

        pub const fn completed(
            run: NativeBackendWinitAppRunTranscript,
            events: NativeBackendWinitAppEventTranscript,
            draw: NativeBackendWinitAppDrawTranscript,
        ) -> Self {
            Self {
                status: NativeBackendWinitAppFacadeTranscriptStatus::Completed,
                submitted_frames_before_run: draw.submitted_after,
                run,
                events,
                draw,
                renderer_used: false,
                frame_presented: false,
            }
        }

        pub const fn failed(
            run: NativeBackendWinitAppRunTranscript,
            draw: NativeBackendWinitAppDrawTranscript,
        ) -> Self {
            Self {
                status: NativeBackendWinitAppFacadeTranscriptStatus::Failed,
                events: NativeBackendWinitAppEventTranscript {
                    status: NativeBackendWinitAppEventTranscriptStatus::NoWindowEventsObserved,
                    window_event_calls: 0,
                    staged_event_count: 0,
                    close_observed: false,
                },
                submitted_frames_before_run: draw.submitted_after,
                run,
                draw,
                renderer_used: false,
                frame_presented: false,
            }
        }

        pub const fn has_draw_staging(&self) -> bool {
            self.draw.submitted_after > 0 || self.draw.submitted_delta > 0
        }

        pub const fn rendered_or_presented(&self) -> bool {
            self.renderer_used || self.frame_presented
        }

        pub fn completed_without_renderer(&self) -> bool {
            matches!(
                self.status,
                NativeBackendWinitAppFacadeTranscriptStatus::Completed
            ) && !self.rendered_or_presented()
        }
    }

    /// Separate native app facade for running NativeBackend through winit.
    ///
    /// This facade intentionally lives outside `UiBackendAdapter`.
    /// It owns the native event-loop path and keeps `prom-ui-runtime` platform-neutral.
    #[derive(Debug)]
    pub struct NativeBackendWinitApp {
        backend: NativeBackend,
    }

    impl NativeBackendWinitApp {
        pub fn new(backend: NativeBackend) -> Result<Self, NativeBackendWinitAppError> {
            if backend.window_config().is_none() {
                return Err(NativeBackendWinitAppError::MissingWindowConfig);
            }

            Ok(Self { backend })
        }

        pub fn backend(&self) -> &NativeBackend {
            &self.backend
        }

        pub fn backend_mut(&mut self) -> &mut NativeBackend {
            &mut self.backend
        }

        pub fn into_backend(self) -> NativeBackend {
            self.backend
        }

        /// Return the planned transcript before native execution.
        pub fn planned_transcript(&self) -> NativeBackendWinitAppRunTranscript {
            NativeBackendWinitAppRunTranscript::planned(self.backend.window_config().is_some())
        }

        /// Build a completed transcript from a run summary.
        pub const fn transcript_from_summary(
            summary: NativeBackendWinitAppStateSummary,
        ) -> NativeBackendWinitAppRunTranscript {
            NativeBackendWinitAppRunTranscript::completed_from_summary(summary)
        }

        /// Build an event transcript from a completed app-state summary.
        pub const fn event_transcript_from_summary(
            summary: NativeBackendWinitAppStateSummary,
        ) -> NativeBackendWinitAppEventTranscript {
            NativeBackendWinitAppEventTranscript::from_summary(summary)
        }

        /// Build an event transcript from a facade run transcript.
        pub const fn event_transcript_from_run_transcript(
            transcript: NativeBackendWinitAppRunTranscript,
        ) -> NativeBackendWinitAppEventTranscript {
            NativeBackendWinitAppEventTranscript::from_run_transcript(transcript)
        }

        /// Return a planned facade transcript including current draw-staging facts.
        pub fn planned_facade_transcript(&self) -> NativeBackendWinitAppFacadeTranscript {
            let run = self.planned_transcript();
            let submitted = self.backend.submitted_frames();
            let draw = NativeBackendWinitAppDrawTranscript::not_submitted(submitted);

            NativeBackendWinitAppFacadeTranscript::planned(run, draw)
        }

        /// Build a completed facade transcript from run summary and pre-run draw facts.
        pub const fn facade_transcript_from_parts(
            run: NativeBackendWinitAppRunTranscript,
            events: NativeBackendWinitAppEventTranscript,
            draw: NativeBackendWinitAppDrawTranscript,
        ) -> NativeBackendWinitAppFacadeTranscript {
            NativeBackendWinitAppFacadeTranscript::completed(run, events, draw)
        }

        /// Run the separate NativeBackend-backed winit app facade until the native window closes.
        ///
        /// This consumes the facade and returns a summary.
        /// It does not modify `NativeBackend::run_event_loop(...)`.
        pub fn run_until_close(
            self,
        ) -> Result<NativeBackendWinitAppStateSummary, NativeBackendWinitAppError> {
            let event_loop = create_winit_event_loop()?;
            let mut app_state = NativeBackendWinitAppState::new(self.backend);

            event_loop.run_app(&mut app_state)?;

            Ok(app_state.summary())
        }

        /// Run until close and return a transcript instead of raw summary.
        pub fn run_until_close_transcript(
            self,
        ) -> Result<NativeBackendWinitAppRunTranscript, NativeBackendWinitAppError> {
            let summary = self.run_until_close()?;
            Ok(NativeBackendWinitAppRunTranscript::completed_from_summary(
                summary,
            ))
        }

        /// Run until close and return a derived event transcript.
        pub fn run_until_close_event_transcript(
            self,
        ) -> Result<NativeBackendWinitAppEventTranscript, NativeBackendWinitAppError> {
            let transcript = self.run_until_close_transcript()?;
            Ok(NativeBackendWinitAppEventTranscript::from_run_transcript(
                transcript,
            ))
        }

        /// Run until close and return a combined facade transcript.
        ///
        /// Draw facts are captured from the inner backend's submitted frame counter
        /// before native run. This does not render or present frames.
        pub fn run_until_close_facade_transcript(
            self,
        ) -> Result<NativeBackendWinitAppFacadeTranscript, NativeBackendWinitAppError> {
            let submitted_before_run = self.backend.submitted_frames();

            let draw = NativeBackendWinitAppDrawTranscript::not_submitted(submitted_before_run);

            let run = self.run_until_close_transcript()?;
            let events = NativeBackendWinitAppEventTranscript::from_run_transcript(run);

            Ok(NativeBackendWinitAppFacadeTranscript::completed(
                run, events, draw,
            ))
        }

        /// Stage a DrawFrame through the facade's inner NativeBackend.
        ///
        /// This only updates backend accounting through the existing `draw_frame(...)`
        /// adapter method. It does not render and does not present anything.
        pub fn stage_draw_frame(
            &mut self,
            frame: &prom_ui_runtime::DrawFrame,
        ) -> Result<NativeBackendWinitAppDrawTranscript, prom_ui_runtime::UiRuntimeError> {
            let before = self.backend.submitted_frames();

            prom_ui_runtime::UiBackendAdapter::draw_frame(&mut self.backend, frame)?;

            let after = self.backend.submitted_frames();

            Ok(NativeBackendWinitAppDrawTranscript::submitted(
                before, after,
            ))
        }
    }

    /// Return a staged draw transcript from frame counters.
    pub const fn draw_transcript_from_counts(
        before: usize,
        after: usize,
    ) -> NativeBackendWinitAppDrawTranscript {
        NativeBackendWinitAppDrawTranscript::submitted(before, after)
    }

    /// Read-only summary of the NativeBackend winit app scaffold.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NativeBackendWinitAppStateSummary {
        pub resumed_calls: usize,
        pub window_event_calls: usize,
        pub create_attempts: usize,
        pub create_failures: usize,
        pub window_created: bool,
        pub close_requested: bool,
        pub staged_event_count: usize,
    }

    /// Winit run loop host that implements `ApplicationHandler`.
    ///
    /// This acts as a bridge between the winit `EventLoop` and `NativeBackend::run_event_loop`.
    /// It hosts the backend temporarily while the event loop runs.
    pub struct WinitRunLoopHost<
        'a,
        F: FnMut(prom_ui_runtime::LoopControl, &mut prom_ui_runtime::DrawFrame),
    > {
        pub backend: &'a mut NativeBackend,
        pub on_event: F,
        pub window: Option<alloc::sync::Arc<Window>>,
        pub window_id: Option<WindowId>,

        #[cfg(feature = "wgpu-backend")]
        pub wgpu_context: Option<super::wgpu_integration::NativeBackendWgpuContext>,
        #[cfg(feature = "wgpu-backend")]
        pub presentation_surface: Option<super::wgpu_integration::NativeBackendPresentationSurface>,
    }

    impl<'a, F: FnMut(prom_ui_runtime::LoopControl, &mut prom_ui_runtime::DrawFrame)>
        WinitRunLoopHost<'a, F>
    {
        pub fn new(backend: &'a mut NativeBackend, on_event: F) -> Self {
            Self {
                backend,
                on_event,
                window: None,
                window_id: None,
                #[cfg(feature = "wgpu-backend")]
                wgpu_context: None,
                #[cfg(feature = "wgpu-backend")]
                presentation_surface: None,
            }
        }
    }

    impl<'a, F: FnMut(prom_ui_runtime::LoopControl, &mut prom_ui_runtime::DrawFrame)>
        ApplicationHandler for WinitRunLoopHost<'a, F>
    {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            if self.window.is_some() {
                return;
            }

            if let Some(config) = &self.backend.window_config {
                let window_attributes = Window::default_attributes()
                    .with_title(config.title.clone())
                    .with_inner_size(winit::dpi::LogicalSize::new(config.width, config.height));

                if let Ok(window) = event_loop.create_window(window_attributes) {
                    self.window_id = Some(window.id());
                    let window_arc = alloc::sync::Arc::new(window);
                    self.window = Some(window_arc.clone());

                    #[cfg(feature = "wgpu-backend")]
                    {
                        if let Some(context) =
                            super::wgpu_integration::NativeBackendWgpuContext::new()
                        {
                            if let Ok(surface) =
                                super::wgpu_integration::NativeBackendPresentationSurface::new(
                                    &context,
                                    window_arc,
                                    config.width,
                                    config.height,
                                )
                            {
                                self.presentation_surface = Some(surface);
                            }
                            self.wgpu_context = Some(context);
                        }
                    }
                }
            }
        }

        fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            window_id: WindowId,
            event: WindowEvent,
        ) {
            if Some(window_id) != self.window_id {
                return;
            }

            #[cfg(feature = "wgpu-backend")]
            {
                match &event {
                    WindowEvent::Resized(physical_size) => {
                        if let (Some(context), Some(surface)) =
                            (&self.wgpu_context, &mut self.presentation_surface)
                        {
                            surface.resize(context, physical_size.width, physical_size.height);
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        if let (Some(context), Some(surface)) =
                            (&self.wgpu_context, &mut self.presentation_surface)
                        {
                            let _ = surface.present_semantic_commands(
                                context,
                                &self.backend.last_frame_commands,
                            );
                        }
                    }
                    _ => {}
                }
            }

            if let Some(backend_event) = translate_winit_to_raw_backend_event(&event) {
                if matches!(backend_event, super::RawBackendEvent::CloseRequested) {
                    let mut frame = prom_ui_runtime::DrawFrame::new();
                    (self.on_event)(prom_ui_runtime::LoopControl::ExitRequested, &mut frame);
                    use prom_ui_runtime::UiBackendAdapter;
                    let _ = self.backend.draw_frame(&frame);
                    event_loop.exit();
                } else {
                    let scale_factor = self
                        .window
                        .as_ref()
                        .map(|w| w.scale_factor())
                        .unwrap_or(1.0);
                    if let Some(input_event) = translate_winit_window_event(&event, scale_factor) {
                        self.backend.pending_events.push(input_event);
                    }
                    let mut frame = prom_ui_runtime::DrawFrame::new();
                    (self.on_event)(prom_ui_runtime::LoopControl::Continue, &mut frame);
                    use prom_ui_runtime::UiBackendAdapter;
                    let _ = self.backend.draw_frame(&frame);
                }
            }
        }

        fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
            // Unconditionally advance the app loop on every iteration
            let mut frame = prom_ui_runtime::DrawFrame::new();
            (self.on_event)(prom_ui_runtime::LoopControl::Continue, &mut frame);
            use prom_ui_runtime::UiBackendAdapter;
            let _ = self.backend.draw_frame(&frame);
            // Redraw continuously
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }

    /// Winit ApplicationHandler scaffold backed by `NativeBackend`.
    ///
    /// This is a state scaffold for future native runtime integration.
    /// It can create one native window and stage translated winit events into
    /// the inner `NativeBackend`, but it is not wired into
    /// `NativeBackend::run_event_loop(...)` yet.
    #[derive(Debug)]
    pub struct NativeBackendWinitAppState {
        backend: NativeBackend,
        window: Option<Window>,
        window_id: Option<WindowId>,
        resumed_calls: usize,
        window_event_calls: usize,
        create_attempts: usize,
        create_failures: usize,
        window_created: bool,
        close_requested: bool,
    }

    impl NativeBackendWinitAppState {
        pub fn new(backend: NativeBackend) -> Self {
            Self {
                backend,
                window: None,
                window_id: None,
                resumed_calls: 0,
                window_event_calls: 0,
                create_attempts: 0,
                create_failures: 0,
                window_created: false,
                close_requested: false,
            }
        }

        pub fn backend(&self) -> &NativeBackend {
            &self.backend
        }

        pub fn backend_mut(&mut self) -> &mut NativeBackend {
            &mut self.backend
        }

        pub const fn window_id(&self) -> Option<WindowId> {
            self.window_id
        }

        pub const fn has_window(&self) -> bool {
            self.window_id.is_some()
        }

        pub const fn resumed_calls(&self) -> usize {
            self.resumed_calls
        }

        pub const fn window_event_calls(&self) -> usize {
            self.window_event_calls
        }

        pub const fn create_attempts(&self) -> usize {
            self.create_attempts
        }

        pub const fn create_failures(&self) -> usize {
            self.create_failures
        }

        pub const fn window_created(&self) -> bool {
            self.window_created
        }

        pub const fn close_requested(&self) -> bool {
            self.close_requested
        }

        pub fn summary(&self) -> NativeBackendWinitAppStateSummary {
            NativeBackendWinitAppStateSummary {
                resumed_calls: self.resumed_calls,
                window_event_calls: self.window_event_calls,
                create_attempts: self.create_attempts,
                create_failures: self.create_failures,
                window_created: self.window_created,
                close_requested: self.close_requested,
                staged_event_count: self.backend.pending_event_count(),
            }
        }

        fn staged_config(&self) -> Option<&WindowConfig> {
            self.backend.window_config()
        }
    }

    impl ApplicationHandler for NativeBackendWinitAppState {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            self.resumed_calls = self.resumed_calls.saturating_add(1);

            if self.window.is_some() {
                return;
            }

            self.create_attempts = self.create_attempts.saturating_add(1);

            let Some(config) = self.staged_config().cloned() else {
                self.create_failures = self.create_failures.saturating_add(1);
                event_loop.exit();
                return;
            };

            match create_winit_window_from_config(event_loop, &config) {
                Ok(window) => {
                    self.window_id = Some(window.id());
                    self.window = Some(window);
                    self.window_created = true;
                }
                Err(_err) => {
                    self.create_failures = self.create_failures.saturating_add(1);
                    event_loop.exit();
                }
            }
        }

        fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            window_id: WindowId,
            event: WindowEvent,
        ) {
            if Some(window_id) != self.window_id {
                return;
            }

            self.window_event_calls = self.window_event_calls.saturating_add(1);

            let scale_factor = self
                .window
                .as_ref()
                .map(|w| w.scale_factor())
                .unwrap_or(1.0);
            if let Some(input) = translate_winit_window_event(&event, scale_factor) {
                if matches!(&input.kind, prom_ui_runtime::InputEventKind::CloseRequested) {
                    self.close_requested = true;
                    self.backend.push_pending_event(input);
                    event_loop.exit();
                    return;
                }

                self.backend.push_pending_event(input);
            }
        }
    }
}

/// Native backend transport boundary.
///
/// Handles event staging and loop lifecycle without owning
/// semantic admission or dispatch.
#[derive(Debug, Clone, PartialEq)]
pub struct NativeBackend {
    platform_wired: bool,
    window_config: Option<WindowConfig>,
    pending_events: alloc::vec::Vec<InputEvent>,
    last_frame_commands: alloc::vec::Vec<prom_ui_runtime::DrawCommand>,
    submitted_frames: usize,
    closed: bool,
    run_loop_calls: usize,
    run_loop_ticks: usize,
}

impl NativeBackend {
    /// Create an unwired native backend skeleton.
    pub fn new() -> Self {
        Self {
            platform_wired: false,
            window_config: None,
            pending_events: alloc::vec::Vec::new(),
            last_frame_commands: alloc::vec::Vec::new(),
            submitted_frames: 0,
            closed: false,
            run_loop_calls: 0,
            run_loop_ticks: 0,
        }
    }

    /// Returns whether this backend is connected to a real platform implementation.
    ///
    /// For G2 this must remain `false`.
    pub const fn is_platform_wired(&self) -> bool {
        self.platform_wired
    }

    pub fn window_config(&self) -> Option<&WindowConfig> {
        self.window_config.as_ref()
    }

    pub fn pending_events(&self) -> &[InputEvent] {
        &self.pending_events
    }

    pub fn pending_event_count(&self) -> usize {
        self.pending_events.len()
    }

    pub fn submitted_frames(&self) -> usize {
        self.submitted_frames
    }

    pub const fn is_closed(&self) -> bool {
        self.closed
    }

    pub const fn run_loop_calls(&self) -> usize {
        self.run_loop_calls
    }

    pub const fn run_loop_ticks(&self) -> usize {
        self.run_loop_ticks
    }

    pub fn push_pending_event(&mut self, event: InputEvent) {
        self.pending_events.push(event);
    }

    pub fn extend_pending_events<I>(&mut self, events: I)
    where
        I: IntoIterator<Item = InputEvent>,
    {
        self.pending_events.extend(events);
    }

    pub fn drain_pending_events(&mut self) -> alloc::vec::Vec<InputEvent> {
        core::mem::replace(&mut self.pending_events, alloc::vec::Vec::new())
    }
}

impl Default for NativeBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "winit-backend")]
impl NativeBackend {
    pub const fn winit_smoke_adapter_available() -> bool {
        true
    }

    /// Stage a translated winit `WindowEvent` into the backend pending-event queue.
    ///
    /// Returns `true` if the event was translated and staged.
    /// Returns `false` for unsupported events.
    ///
    /// This does not run a native event loop and does not mutate platform state.
    pub fn stage_winit_window_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        if let Some(input) = winit_placeholder::translate_winit_window_event(event, 1.0) {
            self.push_pending_event(input);
            true
        } else {
            false
        }
    }

    /// Stage a translated winit physical key event into the backend pending-event queue.
    ///
    /// Returns `true` if the key was supported and staged.
    /// Returns `false` for unsupported or unidentified keys.
    pub fn stage_winit_physical_key(
        &mut self,
        state: winit::event::ElementState,
        physical_key: winit::keyboard::PhysicalKey,
    ) -> bool {
        if let Some(input) = winit_placeholder::translate_winit_physical_key(state, physical_key) {
            self.push_pending_event(input);
            true
        } else {
            false
        }
    }

    /// Stage a close request into the backend pending-event queue.
    pub fn stage_winit_close_requested(&mut self) {
        self.push_pending_event(winit_placeholder::translate_winit_close_requested());
    }

    /// Run the manual winit window-creation smoke using the staged `WindowConfig`.
    ///
    /// This does not integrate winit with `NativeBackend::run_event_loop(...)`.
    /// It creates a temporary native event loop/window through the G12 manual smoke path
    /// and returns the smoke result.
    ///
    /// The created native window is not retained by `NativeBackend`.
    pub fn run_winit_smoke_from_staged_config(
        &self,
    ) -> Result<winit_placeholder::WinitRunAppSmokeResult, NativeBackendWinitSmokeError> {
        let config = self
            .window_config
            .clone()
            .ok_or(NativeBackendWinitSmokeError::MissingWindowConfig)?;

        let result = winit_placeholder::run_winit_window_creation_smoke(config)?;

        Ok(result)
    }
}

impl UiBackendAdapter for NativeBackend {
    fn create_window(&mut self, config: &WindowConfig) -> Result<(), UiRuntimeError> {
        self.window_config = Some(config.clone());
        self.closed = false;
        Ok(())
    }

    fn close_window(&mut self) {
        self.closed = true;
    }

    fn run_event_loop<F: FnMut(prom_ui_runtime::LoopControl, &mut prom_ui_runtime::DrawFrame)>(
        &mut self,
        #[allow(unused_mut)] mut on_event: F,
    ) -> Result<(), UiRuntimeError> {
        self.run_loop_calls = self.run_loop_calls.saturating_add(1);

        #[cfg(feature = "winit-backend")]
        {
            let event_loop = match winit_placeholder::create_winit_event_loop() {
                Ok(el) => el,
                Err(_) => return Err(UiRuntimeError::EventLoopFailed),
            };

            let mut host = winit_placeholder::WinitRunLoopHost::new(self, on_event);

            if event_loop.run_app(&mut host).is_err() {
                return Err(UiRuntimeError::EventLoopFailed);
            }
        }

        #[cfg(not(feature = "winit-backend"))]
        {
            let events = self.drain_pending_events();

            for event in events {
                self.run_loop_ticks = self.run_loop_ticks.saturating_add(1);

                match event.kind {
                    InputEventKind::CloseRequested => {
                        let mut frame = prom_ui_runtime::DrawFrame::new();
                        on_event(LoopControl::ExitRequested, &mut frame);
                        break;
                    }
                    _ => {
                        let mut frame = prom_ui_runtime::DrawFrame::new();
                        on_event(LoopControl::Continue, &mut frame);
                    }
                }
            }
        }

        Ok(())
    }

    fn draw_frame(&mut self, frame: &DrawFrame) -> Result<(), UiRuntimeError> {
        self.last_frame_commands = frame.commands().to_vec();
        self.submitted_frames = self.submitted_frames.saturating_add(1);
        Ok(())
    }
}

#[cfg(feature = "wgpu-backend")]
pub fn selected_draw_backend_name() -> &'static str {
    "wgpu"
}

#[cfg(feature = "wgpu-backend")]
pub mod wgpu_integration {
    //! Feature-gated wgpu integration.
    //!
    //! Provides `NativeBackendWgpuContext` and `NativeBackendPresentationSurface`
    //! which are wired into the `winit` run loop when both `wgpu-backend` and
    //! `winit-backend` features are enabled.

    use wgpu::{Adapter, Device, Instance, Queue};

    /// Encapsulates the core wgpu primitives.
    #[derive(Debug)]
    pub struct NativeBackendWgpuContext {
        pub instance: Instance,
        pub adapter: Adapter,
        pub device: Device,
        pub queue: Queue,
        pub pipeline: wgpu::RenderPipeline,
    }

    #[repr(C)]
    #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
    struct Vertex {
        position: [f32; 2],
        color: [f32; 4],
    }

    impl Vertex {
        fn desc() -> wgpu::VertexBufferLayout<'static> {
            wgpu::VertexBufferLayout {
                array_stride: core::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x2,
                    },
                    wgpu::VertexAttribute {
                        offset: core::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                        shader_location: 1,
                        format: wgpu::VertexFormat::Float32x4,
                    },
                ],
            }
        }
    }

    impl NativeBackendWgpuContext {
        /// Initializes the wgpu context asynchronously and blocks using `pollster`.
        ///
        /// This creates the instance, requests the default adapter, and requests the device.
        pub fn new() -> Option<Self> {
            let instance = Instance::default();

            // Block on adapter request
            let adapter =
                pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    force_fallback_adapter: false,
                    compatible_surface: None, // No surface yet
                }))?;

            // Block on device request
            let (device, queue) = pollster::block_on(adapter.request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("NativeBackend Minimal Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                },
                None,
            ))
            .ok()?;

            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Rect Shader"),
                source: wgpu::ShaderSource::Wgsl(alloc::borrow::Cow::Borrowed(
                    "
                    struct VertexInput {
                        @location(0) position: vec2<f32>,
                        @location(1) color: vec4<f32>,
                    };

                    struct VertexOutput {
                        @builtin(position) clip_position: vec4<f32>,
                        @location(0) color: vec4<f32>,
                    };

                    @vertex
                    fn vs_main(model: VertexInput) -> VertexOutput {
                        var out: VertexOutput;
                        out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
                        out.color = model.color;
                        return out;
                    }

                    @fragment
                    fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
                        return in.color;
                    }
                ",
                )),
            });

            let render_pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc()],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8UnormSrgb,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

            Some(Self {
                instance,
                adapter,
                device,
                queue,
                pipeline,
            })
        }

        /// Proves minimal draw execution by clearing a dummy texture offscreen.
        ///
        /// This records a minimal `RenderPass` without swapchain presentation.
        pub fn execute_minimal_draw_pass(&self) {
            let texture_size = wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            };

            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Minimal Draw Dummy Texture"),
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Minimal Draw Encoder"),
                });

            {
                let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Minimal Draw Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
            }

            self.queue.submit(core::iter::once(encoder.finish()));
        }
    }

    /// Feature-gated surface presentation logic.
    ///
    /// Requires both `wgpu-backend` and `winit-backend` to link physical pixels to a native window.
    #[cfg(feature = "winit-backend")]
    #[derive(Debug)]
    pub struct NativeBackendPresentationSurface {
        pub surface: wgpu::Surface<'static>,
        pub config: wgpu::SurfaceConfiguration,
    }

    #[cfg(feature = "winit-backend")]
    impl NativeBackendPresentationSurface {
        /// Creates a new surface from an `Arc<winit::window::Window>`.
        pub fn new(
            context: &NativeBackendWgpuContext,
            window: alloc::sync::Arc<winit::window::Window>,
            width: u32,
            height: u32,
        ) -> Result<Self, wgpu::CreateSurfaceError> {
            let surface = context.instance.create_surface(window)?;

            let max_dim = context.device.limits().max_texture_dimension_2d;
            let w = width.clamp(1, max_dim);
            let h = height.clamp(1, max_dim);

            let mut config = surface
                .get_default_config(&context.adapter, w, h)
                .unwrap_or_else(|| wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    width: w,
                    height: h,
                    present_mode: wgpu::PresentMode::AutoVsync,
                    alpha_mode: wgpu::CompositeAlphaMode::Auto,
                    view_formats: alloc::vec![],
                    desired_maximum_frame_latency: 2,
                });
            config.width = w;
            config.height = h;
            config.format = wgpu::TextureFormat::Rgba8UnormSrgb;
            surface.configure(&context.device, &config);

            Ok(Self { surface, config })
        }

        /// Reconfigures the swapchain upon window resize.
        pub fn resize(&mut self, context: &NativeBackendWgpuContext, width: u32, height: u32) {
            let max_dim = context.device.limits().max_texture_dimension_2d;
            let w = width.clamp(1, max_dim);
            let h = height.clamp(1, max_dim);
            if self.config.width != w || self.config.height != h {
                self.config.width = w;
                self.config.height = h;
                self.surface.configure(&context.device, &self.config);
            }
        }

        /// Presents a minimal clear color to the surface to prove connectivity.
        ///
        /// Returns `false` if the surface was lost or outdated, requiring reconfiguration.
        pub fn present_minimal_clear(&self, context: &NativeBackendWgpuContext) -> bool {
            self.present_semantic_commands(context, &[])
        }

        /// Presents a semantic frame to the surface by mapping `DrawCommand`s to physical wgpu instructions.
        pub fn present_semantic_commands(
            &self,
            context: &NativeBackendWgpuContext,
            commands: &[prom_ui_runtime::DrawCommand],
        ) -> bool {
            let frame = match self.surface.get_current_texture() {
                Ok(frame) => frame,
                Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => return false,
                Err(_e) => return false,
            };

            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder =
                context
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Semantic Presentation Encoder"),
                    });

            // Extract the background color from a semantic Clear command, defaulting to static color if not found.
            let mut bg_color = wgpu::Color {
                r: 0.15,
                g: 0.15,
                b: 0.2,
                a: 1.0,
            };

            let mut vertices = alloc::vec::Vec::new();
            let surface_width = self.config.width as f32;
            let surface_height = self.config.height as f32;

            for cmd in commands {
                match cmd {
                    prom_ui_runtime::DrawCommand::Clear { color } => {
                        bg_color = wgpu::Color {
                            r: color.r as f64 / 255.0,
                            g: color.g as f64 / 255.0,
                            b: color.b as f64 / 255.0,
                            a: color.a as f64 / 255.0,
                        };
                    }
                    prom_ui_runtime::DrawCommand::FillRect { rect, color } => {
                        let r = color.r as f32 / 255.0;
                        let g = color.g as f32 / 255.0;
                        let b = color.b as f32 / 255.0;
                        let a = color.a as f32 / 255.0;
                        let c = [r, g, b, a];

                        let x0 = (rect.x as f32 / surface_width) * 2.0 - 1.0;
                        let y0 = 1.0 - (rect.y as f32 / surface_height) * 2.0;
                        let x1 = ((rect.x + rect.width as i32) as f32 / surface_width) * 2.0 - 1.0;
                        let y1 =
                            1.0 - ((rect.y + rect.height as i32) as f32 / surface_height) * 2.0;

                        // Triangle 1
                        vertices.push(Vertex {
                            position: [x0, y0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x0, y1],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x1, y0],
                            color: c,
                        });
                        // Triangle 2
                        vertices.push(Vertex {
                            position: [x1, y0],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x0, y1],
                            color: c,
                        });
                        vertices.push(Vertex {
                            position: [x1, y1],
                            color: c,
                        });
                    }
                    _ => {}
                }
            }

            let vertex_buffer = if !vertices.is_empty() {
                use wgpu::util::DeviceExt;
                Some(
                    context
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Rect Vertex Buffer"),
                            contents: bytemuck::cast_slice(&vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        }),
                )
            } else {
                None
            };

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Semantic Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(bg_color),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                if let Some(vb) = &vertex_buffer {
                    render_pass.set_pipeline(&context.pipeline);
                    render_pass.set_vertex_buffer(0, vb.slice(..));
                    render_pass.draw(0..vertices.len() as u32, 0..1);
                }
            }

            context.queue.submit(Some(encoder.finish()));
            frame.present();
            true
        }
    }
}
