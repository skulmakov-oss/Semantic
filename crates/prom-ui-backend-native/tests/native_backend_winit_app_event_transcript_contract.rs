#![cfg(feature = "winit-backend")]

use prom_ui_backend_native::{
    winit_placeholder::{
        native_backend_winit_app_event_transcript_available, NativeBackendWinitApp,
        NativeBackendWinitAppEventTranscript, NativeBackendWinitAppEventTranscriptStatus,
        NativeBackendWinitAppRunTranscript, NativeBackendWinitAppRunTranscriptStatus,
        NativeBackendWinitAppStateSummary,
    },
    NativeBackend,
};
use prom_ui_runtime::{UiBackendAdapter, WindowConfig};

#[test]
fn native_backend_winit_app_event_transcript_is_available() {
    assert!(prom_ui_backend_native::winit_backend_feature_enabled());
    assert!(native_backend_winit_app_event_transcript_available());
}

#[test]
fn event_transcript_from_empty_summary_records_no_events() {
    let summary = NativeBackendWinitAppStateSummary {
        resumed_calls: 0,
        window_event_calls: 0,
        create_attempts: 0,
        create_failures: 0,
        window_created: false,
        close_requested: false,
        staged_event_count: 0,
    };

    let transcript = NativeBackendWinitAppEventTranscript::from_summary(summary);

    assert_eq!(
        transcript.status,
        NativeBackendWinitAppEventTranscriptStatus::NoWindowEventsObserved
    );
    assert_eq!(transcript.window_event_calls, 0);
    assert_eq!(transcript.staged_event_count, 0);
    assert!(!transcript.close_observed);
    assert!(!transcript.observed_any_event());
}

#[test]
fn event_transcript_records_window_events_without_close() {
    let summary = NativeBackendWinitAppStateSummary {
        resumed_calls: 1,
        window_event_calls: 2,
        create_attempts: 1,
        create_failures: 0,
        window_created: true,
        close_requested: false,
        staged_event_count: 1,
    };

    let transcript = NativeBackendWinitAppEventTranscript::from_summary(summary);

    assert_eq!(
        transcript.status,
        NativeBackendWinitAppEventTranscriptStatus::WindowEventsObserved
    );
    assert_eq!(transcript.window_event_calls, 2);
    assert_eq!(transcript.staged_event_count, 1);
    assert!(!transcript.close_observed);
    assert!(transcript.observed_any_event());
}

#[test]
fn event_transcript_records_close_observed() {
    let summary = NativeBackendWinitAppStateSummary {
        resumed_calls: 1,
        window_event_calls: 1,
        create_attempts: 1,
        create_failures: 0,
        window_created: true,
        close_requested: true,
        staged_event_count: 1,
    };

    let transcript = NativeBackendWinitAppEventTranscript::from_summary(summary);

    assert_eq!(
        transcript.status,
        NativeBackendWinitAppEventTranscriptStatus::CloseObserved
    );
    assert_eq!(transcript.window_event_calls, 1);
    assert_eq!(transcript.staged_event_count, 1);
    assert!(transcript.close_observed);
    assert!(transcript.observed_any_event());
    assert!(transcript.observed_close());
}

#[test]
fn event_transcript_can_be_extracted_from_run_transcript() {
    let run_transcript = NativeBackendWinitAppRunTranscript {
        status: NativeBackendWinitAppRunTranscriptStatus::Completed,

        staged_window_config: true,
        event_loop_requested: true,
        app_state_created: true,
        run_app_requested: true,

        resumed_calls: 1,
        window_event_calls: 3,
        create_attempts: 1,
        create_failures: 0,
        window_created: true,
        close_requested: true,
        staged_event_count: 2,
    };

    let event_transcript =
        NativeBackendWinitAppEventTranscript::from_run_transcript(run_transcript);

    assert_eq!(
        event_transcript.status,
        NativeBackendWinitAppEventTranscriptStatus::CloseObserved
    );
    assert_eq!(event_transcript.window_event_calls, 3);
    assert_eq!(event_transcript.staged_event_count, 2);
    assert!(event_transcript.close_observed);
}

#[test]
fn facade_event_transcript_from_summary_maps_summary() {
    let summary = NativeBackendWinitAppStateSummary {
        resumed_calls: 1,
        window_event_calls: 4,
        create_attempts: 1,
        create_failures: 0,
        window_created: true,
        close_requested: true,
        staged_event_count: 2,
    };

    let transcript = NativeBackendWinitApp::event_transcript_from_summary(summary);

    assert_eq!(
        transcript.status,
        NativeBackendWinitAppEventTranscriptStatus::CloseObserved
    );
    assert_eq!(transcript.window_event_calls, 4);
    assert_eq!(transcript.staged_event_count, 2);
}

#[test]
fn facade_run_until_close_event_transcript_has_expected_shape() {
    let _f: fn(
        NativeBackendWinitApp,
    ) -> Result<
        NativeBackendWinitAppEventTranscript,
        prom_ui_backend_native::winit_placeholder::NativeBackendWinitAppError,
    > = NativeBackendWinitApp::run_until_close_event_transcript;
}

#[test]
#[ignore = "manual NativeBackendWinitApp event transcript smoke; opens a native window"]
fn manual_native_backend_winit_app_facade_returns_event_transcript() {
    let mut backend = NativeBackend::new();
    let config = WindowConfig::new("Semantic Native Event Transcript", 800, 600);

    backend.create_window(&config).unwrap();

    let facade = NativeBackendWinitApp::new(backend).unwrap();

    let transcript = facade
        .run_until_close_event_transcript()
        .expect("manual NativeBackendWinitApp event transcript run should complete");

    assert_eq!(
        transcript.status,
        NativeBackendWinitAppEventTranscriptStatus::CloseObserved
    );
    assert!(transcript.observed_close());
    assert!(transcript.observed_any_event());
    assert!(transcript.staged_event_count >= 1);
}
