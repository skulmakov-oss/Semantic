#![cfg(feature = "winit-backend")]

use prom_ui_backend_native::{
    winit_placeholder::{
        native_backend_winit_app_facade_draw_run_transcript_available,
        NativeBackendWinitApp,
        NativeBackendWinitAppDrawTranscript,
        NativeBackendWinitAppEventTranscript,
        NativeBackendWinitAppEventTranscriptStatus,
        NativeBackendWinitAppFacadeTranscript,
        NativeBackendWinitAppFacadeTranscriptStatus,
        NativeBackendWinitAppRunTranscript,
        NativeBackendWinitAppStateSummary,
    },
    NativeBackend,
};
use prom_ui_runtime::{DrawFrame, UiBackendAdapter, WindowConfig};

#[test]
fn native_backend_winit_app_facade_draw_run_transcript_is_available() {
    assert!(prom_ui_backend_native::winit_backend_feature_enabled());
    assert!(native_backend_winit_app_facade_draw_run_transcript_available());
}

#[test]
fn planned_facade_transcript_records_no_renderer_or_presentation() {
    let mut backend = NativeBackend::new();
    let config = WindowConfig::new("Facade Transcript", 800, 600);

    backend.create_window(&config).unwrap();

    let facade = NativeBackendWinitApp::new(backend).unwrap();
    let transcript = facade.planned_facade_transcript();

    assert_eq!(
        transcript.status,
        NativeBackendWinitAppFacadeTranscriptStatus::Planned
    );
    assert!(!transcript.renderer_used);
    assert!(!transcript.frame_presented);
    assert!(!transcript.rendered_or_presented());
}

#[test]
fn planned_facade_transcript_records_existing_draw_staging_count() {
    let mut backend = NativeBackend::new();
    let config = WindowConfig::new("Draw Count", 800, 600);

    backend.create_window(&config).unwrap();

    let mut facade = NativeBackendWinitApp::new(backend).unwrap();
    let frame = DrawFrame::new();

    facade.stage_draw_frame(&frame).unwrap();
    facade.stage_draw_frame(&frame).unwrap();

    let transcript = facade.planned_facade_transcript();

    assert_eq!(transcript.submitted_frames_before_run, 2);
    assert_eq!(transcript.draw.submitted_before, 2);
    assert_eq!(transcript.draw.submitted_after, 2);
    assert_eq!(transcript.draw.submitted_delta, 0);
    assert!(transcript.has_draw_staging());
}

#[test]
fn completed_facade_transcript_combines_run_event_and_draw_facts() {
    let summary = NativeBackendWinitAppStateSummary {
        resumed_calls: 1,
        window_event_calls: 2,
        create_attempts: 1,
        create_failures: 0,
        window_created: true,
        close_requested: true,
        staged_event_count: 1,
    };

    let run = NativeBackendWinitAppRunTranscript::completed_from_summary(summary);
    let events = NativeBackendWinitAppEventTranscript::from_summary(summary);
    let draw = NativeBackendWinitAppDrawTranscript::submitted(0, 1);

    let transcript = NativeBackendWinitAppFacadeTranscript::completed(run, events, draw);

    assert_eq!(
        transcript.status,
        NativeBackendWinitAppFacadeTranscriptStatus::Completed
    );
    assert_eq!(transcript.run.resumed_calls, 1);
    assert_eq!(
        transcript.events.status,
        NativeBackendWinitAppEventTranscriptStatus::CloseObserved
    );
    assert!(transcript.draw.accepted_one_frame());
    assert!(transcript.completed_without_renderer());
}

#[test]
fn failed_facade_transcript_remains_renderer_free() {
    let run = NativeBackendWinitAppRunTranscript::failed_after_request(true);
    let draw = NativeBackendWinitAppDrawTranscript::not_submitted(0);

    let transcript = NativeBackendWinitAppFacadeTranscript::failed(run, draw);

    assert_eq!(
        transcript.status,
        NativeBackendWinitAppFacadeTranscriptStatus::Failed
    );
    assert!(!transcript.renderer_used);
    assert!(!transcript.frame_presented);
    assert!(!transcript.rendered_or_presented());
}

#[test]
fn facade_helper_combines_transcript_parts() {
    let summary = NativeBackendWinitAppStateSummary {
        resumed_calls: 2,
        window_event_calls: 3,
        create_attempts: 1,
        create_failures: 0,
        window_created: true,
        close_requested: true,
        staged_event_count: 2,
    };

    let run = NativeBackendWinitApp::transcript_from_summary(summary);
    let events = NativeBackendWinitApp::event_transcript_from_summary(summary);
    let draw = NativeBackendWinitAppDrawTranscript::submitted(1, 2);

    let transcript = NativeBackendWinitApp::facade_transcript_from_parts(run, events, draw);

    assert_eq!(
        transcript.status,
        NativeBackendWinitAppFacadeTranscriptStatus::Completed
    );
    assert_eq!(transcript.run.window_event_calls, 3);
    assert_eq!(transcript.events.staged_event_count, 2);
    assert_eq!(transcript.draw.submitted_delta, 1);
}

#[test]
fn facade_run_until_close_facade_transcript_has_expected_shape() {
    let _f: fn(
        NativeBackendWinitApp,
    ) -> Result<
        NativeBackendWinitAppFacadeTranscript,
        prom_ui_backend_native::winit_placeholder::NativeBackendWinitAppError,
    > = NativeBackendWinitApp::run_until_close_facade_transcript;
}

#[test]
#[ignore = "manual NativeBackendWinitApp facade draw/run transcript smoke; opens a native window"]
fn manual_native_backend_winit_app_facade_returns_draw_run_transcript() {
    let mut backend = NativeBackend::new();
    let config = WindowConfig::new("Semantic Native Facade Transcript", 800, 600);

    backend.create_window(&config).unwrap();

    let mut facade = NativeBackendWinitApp::new(backend).unwrap();
    let frame = DrawFrame::new();

    facade.stage_draw_frame(&frame).unwrap();

    let transcript = facade
        .run_until_close_facade_transcript()
        .expect("manual NativeBackendWinitApp facade transcript run should complete");

    assert_eq!(
        transcript.status,
        NativeBackendWinitAppFacadeTranscriptStatus::Completed
    );
    assert!(transcript.completed_without_renderer());
    assert!(transcript.has_draw_staging());
    assert!(transcript.events.observed_close());
    assert_eq!(transcript.submitted_frames_before_run, 1);
}
