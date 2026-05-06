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
