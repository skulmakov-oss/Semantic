#![cfg(feature = "winit-backend")]

use prom_ui_backend_native::{
    winit_placeholder::{
        native_backend_winit_app_draw_staging_available, NativeBackendWinitApp,
        NativeBackendWinitAppDrawStagingStatus, NativeBackendWinitAppDrawTranscript,
    },
    NativeBackend,
};
use prom_ui_runtime::{DrawFrame, UiBackendAdapter, WindowConfig};

#[test]
fn native_backend_winit_app_draw_staging_is_available() {
    assert!(prom_ui_backend_native::winit_backend_feature_enabled());
    assert!(native_backend_winit_app_draw_staging_available());
}

#[test]
fn draw_transcript_can_represent_not_submitted() {
    let transcript = NativeBackendWinitAppDrawTranscript::not_submitted(3);

    assert_eq!(
        transcript.status,
        NativeBackendWinitAppDrawStagingStatus::NotSubmitted
    );
    assert_eq!(transcript.submitted_before, 3);
    assert_eq!(transcript.submitted_after, 3);
    assert_eq!(transcript.submitted_delta, 0);
    assert!(!transcript.accepted_one_frame());
}

#[test]
fn draw_transcript_can_represent_one_submitted_frame() {
    let transcript = NativeBackendWinitAppDrawTranscript::submitted(2, 3);

    assert_eq!(
        transcript.status,
        NativeBackendWinitAppDrawStagingStatus::Submitted
    );
    assert_eq!(transcript.submitted_before, 2);
    assert_eq!(transcript.submitted_after, 3);
    assert_eq!(transcript.submitted_delta, 1);
    assert!(transcript.accepted_one_frame());
}

#[test]
fn native_backend_winit_app_facade_exposes_mutable_backend_for_staging() {
    let mut backend = NativeBackend::new();
    let config = WindowConfig::new("Draw Staging", 640, 480);

    backend.create_window(&config).unwrap();

    let mut facade = NativeBackendWinitApp::new(backend).unwrap();

    facade.backend_mut().stage_winit_close_requested();

    assert_eq!(facade.backend().pending_event_count(), 1);
}

#[test]
fn stage_draw_frame_increments_backend_submitted_frame_count() {
    let mut backend = NativeBackend::new();
    let config = WindowConfig::new("Draw Staging", 640, 480);

    backend.create_window(&config).unwrap();

    let mut facade = NativeBackendWinitApp::new(backend).unwrap();
    let frame = DrawFrame::new();

    let transcript = facade
        .stage_draw_frame(&frame)
        .expect("draw staging should succeed");

    assert!(transcript.accepted_one_frame());
    assert_eq!(transcript.submitted_before, 0);
    assert_eq!(transcript.submitted_after, 1);
    assert_eq!(facade.backend().submitted_frames(), 1);
}

#[test]
fn multiple_staged_draw_frames_accumulate() {
    let mut backend = NativeBackend::new();
    let config = WindowConfig::new("Draw Accumulate", 640, 480);

    backend.create_window(&config).unwrap();

    let mut facade = NativeBackendWinitApp::new(backend).unwrap();
    let frame = DrawFrame::new();

    let first = facade.stage_draw_frame(&frame).unwrap();
    let second = facade.stage_draw_frame(&frame).unwrap();

    assert_eq!(first.submitted_before, 0);
    assert_eq!(first.submitted_after, 1);
    assert_eq!(second.submitted_before, 1);
    assert_eq!(second.submitted_after, 2);
    assert_eq!(facade.backend().submitted_frames(), 2);
}

#[test]
fn staging_draw_frame_does_not_mark_backend_platform_wired() {
    let mut backend = NativeBackend::new();
    let config = WindowConfig::new("No Platform Wiring", 640, 480);

    backend.create_window(&config).unwrap();

    let mut facade = NativeBackendWinitApp::new(backend).unwrap();
    let frame = DrawFrame::new();

    facade.stage_draw_frame(&frame).unwrap();

    assert!(!facade.backend().is_platform_wired());
}

#[test]
fn facade_stage_draw_frame_has_expected_shape() {
    let _f: fn(
        &mut NativeBackendWinitApp,
        &DrawFrame,
    ) -> Result<NativeBackendWinitAppDrawTranscript, prom_ui_runtime::UiRuntimeError> =
        NativeBackendWinitApp::stage_draw_frame;
}
