//! Native backend crate skeleton for Semantic UI.
//!
//! This crate is the future home of platform-specific UI backend code.
//! It intentionally does not create native windows yet.
//!
//! Boundary:
//! - `prom-ui-runtime` remains platform-neutral.
//! - Native/window dependencies belong here, not in `prom-ui-runtime`.
//! - The only current seam is `UiBackendAdapter`.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use prom_ui_runtime::{
    DrawFrame, InputEvent, InputEventKind, LoopControl, UiBackendAdapter, UiRuntimeError,
    WindowConfig,
};

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
    cfg!(feature = "winit-backend")
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
        event::{ElementState, WindowEvent},
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
    ) -> Option<prom_ui_runtime::InputEvent> {
        match event {
            WindowEvent::CloseRequested => Some(translate_winit_close_requested()),
            WindowEvent::KeyboardInput { event, .. } => {
                translate_winit_physical_key(event.state, event.physical_key)
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

        /// Future step: adapter integration has not been admitted yet.
        AdapterIntegrationDeferred,
    }

    /// Ownership model for the planned NativeBackend/winit run-loop path.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum NativeBackendWinitRunLoopOwnership {
        /// Manual helper consumes NativeBackend and returns summary only.
        ConsumesBackendReturnsSummary,

        /// Future persistent ownership model is intentionally unresolved.
        PersistentBackendOwnershipDeferred,
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

        pub const fn keeps_runtime_boundary_clean(&self) -> bool {
            !self.integrates_with_backend_run_event_loop
                && !self.mutates_prom_ui_runtime
                && !self.changes_ui_backend_adapter
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

            if let Some(input) = translate_winit_window_event(&event) {
                if matches!(
                    &input.kind,
                    prom_ui_runtime::InputEventKind::CloseRequested
                ) {
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

/// Skeleton native backend.
///
/// This type is intentionally not wired to a platform windowing library yet.
/// It exists to lock the crate boundary before real native integration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeBackend {
    platform_wired: bool,
    window_config: Option<WindowConfig>,
    pending_events: alloc::vec::Vec<InputEvent>,
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
        if let Some(input) = winit_placeholder::translate_winit_window_event(event) {
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
    ) -> Result<
        winit_placeholder::WinitRunAppSmokeResult,
        NativeBackendWinitSmokeError,
    > {
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

    fn run_event_loop<F: FnMut(LoopControl)>(
        &mut self,
        mut on_event: F,
    ) -> Result<(), UiRuntimeError> {
        self.run_loop_calls = self.run_loop_calls.saturating_add(1);

        let events = self.drain_pending_events();

        for event in events {
            self.run_loop_ticks = self.run_loop_ticks.saturating_add(1);

            match event.kind {
                InputEventKind::CloseRequested => {
                    on_event(LoopControl::ExitRequested);
                    break;
                }
                _ => on_event(LoopControl::Continue),
            }
        }

        Ok(())
    }

    fn draw_frame(&mut self, _frame: &DrawFrame) -> Result<(), UiRuntimeError> {
        self.submitted_frames = self.submitted_frames.saturating_add(1);
        Ok(())
    }
}
