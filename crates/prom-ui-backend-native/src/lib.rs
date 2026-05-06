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

use prom_ui::UiOperationId;
use prom_ui_runtime::{DrawFrame, LoopControl, UiBackendAdapter, UiRuntimeError, WindowConfig};

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

    use prom_ui_runtime::WindowConfig;
    use winit::{
        application::ApplicationHandler,
        dpi::LogicalSize,
        event::WindowEvent,
        event_loop::ActiveEventLoop,
        window::{Window, WindowAttributes, WindowId},
    };

    /// Compile-time marker proving the `winit` crate is available behind the feature.
    pub const WINIT_BACKEND_PLACEHOLDER: bool = true;

    /// Return the winit crate version selected by Cargo.
    ///
    /// Kept as a simple static marker to avoid touching native APIs in G3.
    pub const fn winit_backend_placeholder_enabled() -> bool {
        true
    }

    /// Returns a marker that anchors the current winit ApplicationHandler scaffold.
    pub const fn winit_event_loop_scaffold_available() -> bool {
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
}

/// Skeleton native backend.
///
/// This type is intentionally not wired to a platform windowing library yet.
/// It exists to lock the crate boundary before real native integration.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NativeBackend {
    platform_wired: bool,
}

impl NativeBackend {
    /// Create an unwired native backend skeleton.
    pub const fn new() -> Self {
        Self {
            platform_wired: false,
        }
    }

    /// Returns whether this backend is connected to a real platform implementation.
    ///
    /// For G2 this must remain `false`.
    pub const fn is_platform_wired(&self) -> bool {
        self.platform_wired
    }
}

impl UiBackendAdapter for NativeBackend {
    fn create_window(&mut self, _config: &WindowConfig) -> Result<(), UiRuntimeError> {
        Err(UiRuntimeError::OperationNotAdmitted(
            UiOperationId::WindowCreate,
        ))
    }

    fn close_window(&mut self) {
        // No-op while the backend is not platform-wired.
    }

    fn run_event_loop<F: FnMut(LoopControl)>(
        &mut self,
        _on_event: F,
    ) -> Result<(), UiRuntimeError> {
        Err(UiRuntimeError::OperationNotAdmitted(UiOperationId::WindowRun))
    }

    fn draw_frame(&mut self, _frame: &DrawFrame) -> Result<(), UiRuntimeError> {
        Err(UiRuntimeError::OperationNotAdmitted(
            UiOperationId::FrameSubmit,
        ))
    }
}
