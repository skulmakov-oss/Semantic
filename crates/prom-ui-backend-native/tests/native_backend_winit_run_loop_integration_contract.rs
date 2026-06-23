#![cfg(feature = "winit-backend")]

use prom_ui_backend_native::{
    winit_placeholder::{NativeBackendWinitAppError, NativeBackendWinitAppFacadeTranscript},
    NativeBackend,
};

#[test]
fn native_backend_run_winit_run_loop_from_staged_config_transcript_exists() {
    let _f: fn(
        NativeBackend,
    ) -> Result<NativeBackendWinitAppFacadeTranscript, NativeBackendWinitAppError> =
        NativeBackend::run_winit_run_loop_from_staged_config_transcript;
}

#[test]
fn run_winit_run_loop_from_staged_config_transcript_rejects_missing_config() {
    let backend = NativeBackend::new();

    let err = backend
        .run_winit_run_loop_from_staged_config_transcript()
        .expect_err("Must require staged WindowConfig without spawning event loop");

    assert!(matches!(
        err,
        NativeBackendWinitAppError::MissingWindowConfig
    ));
}

#[test]
fn run_winit_run_loop_from_staged_config_transcript_preserves_no_renderer_constraint() {
    let plan =
        prom_ui_backend_native::winit_placeholder::current_native_backend_winit_run_loop_plan();
    assert!(!plan.includes_renderer);
    assert!(!plan.presents_frames);
}
