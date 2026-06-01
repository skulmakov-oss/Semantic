#![cfg(feature = "winit-backend")]

use prom_ui_backend_native::{
    winit_placeholder::{
        native_backend_winit_app_facade_transcript_available, NativeBackendWinitApp,
        NativeBackendWinitAppRunTranscript, NativeBackendWinitAppRunTranscriptStatus,
        NativeBackendWinitAppStateSummary,
    },
    NativeBackend,
};
use prom_ui_runtime::{UiBackendAdapter, WindowConfig};

#[test]
fn native_backend_winit_app_facade_transcript_is_available() {
    assert!(prom_ui_backend_native::winit_backend_feature_enabled());
    assert!(native_backend_winit_app_facade_transcript_available());
}

#[test]
fn facade_planned_transcript_records_staged_config_without_running() {
    let mut backend = NativeBackend::new();
    let config = WindowConfig::new("Transcript", 800, 600);

    backend.create_window(&config).unwrap();

    let facade = NativeBackendWinitApp::new(backend).unwrap();
    let transcript = facade.planned_transcript();

    assert_eq!(
        transcript.status,
        NativeBackendWinitAppRunTranscriptStatus::Planned
    );
    assert!(transcript.staged_window_config);
    assert!(!transcript.event_loop_requested);
    assert!(!transcript.app_state_created);
    assert!(!transcript.run_app_requested);
    assert_eq!(transcript.resumed_calls, 0);
    assert_eq!(transcript.window_event_calls, 0);
}

#[test]
fn planned_transcript_can_record_missing_config() {
    let transcript = NativeBackendWinitAppRunTranscript::planned(false);

    assert_eq!(
        transcript.status,
        NativeBackendWinitAppRunTranscriptStatus::Planned
    );
    assert!(!transcript.staged_window_config);
    assert!(!transcript.event_loop_requested);
    assert!(!transcript.app_state_created);
    assert!(!transcript.run_app_requested);
    assert!(!transcript.completed_cleanly());
}

#[test]
fn completed_transcript_maps_summary_exactly() {
    let summary = NativeBackendWinitAppStateSummary {
        resumed_calls: 1,
        window_event_calls: 2,
        create_attempts: 1,
        create_failures: 0,
        window_created: true,
        close_requested: true,
        staged_event_count: 1,
    };

    let transcript = NativeBackendWinitAppRunTranscript::completed_from_summary(summary);

    assert_eq!(
        transcript.status,
        NativeBackendWinitAppRunTranscriptStatus::Completed
    );
    assert!(transcript.staged_window_config);
    assert!(transcript.event_loop_requested);
    assert!(transcript.app_state_created);
    assert!(transcript.run_app_requested);

    assert_eq!(transcript.resumed_calls, 1);
    assert_eq!(transcript.window_event_calls, 2);
    assert_eq!(transcript.create_attempts, 1);
    assert_eq!(transcript.create_failures, 0);
    assert!(transcript.window_created);
    assert!(transcript.close_requested);
    assert_eq!(transcript.staged_event_count, 1);
    assert!(transcript.completed_cleanly());
}

#[test]
fn facade_transcript_from_summary_maps_summary() {
    let summary = NativeBackendWinitAppStateSummary {
        resumed_calls: 3,
        window_event_calls: 4,
        create_attempts: 1,
        create_failures: 0,
        window_created: true,
        close_requested: true,
        staged_event_count: 2,
    };

    let transcript = NativeBackendWinitApp::transcript_from_summary(summary);

    assert_eq!(
        transcript.status,
        NativeBackendWinitAppRunTranscriptStatus::Completed
    );
    assert_eq!(transcript.resumed_calls, 3);
    assert_eq!(transcript.window_event_calls, 4);
    assert_eq!(transcript.staged_event_count, 2);
}

#[test]
fn failed_transcript_is_explicit() {
    let transcript = NativeBackendWinitAppRunTranscript::failed_after_request(true);

    assert_eq!(
        transcript.status,
        NativeBackendWinitAppRunTranscriptStatus::Failed
    );
    assert!(transcript.staged_window_config);
    assert!(transcript.event_loop_requested);
    assert!(!transcript.app_state_created);
    assert!(!transcript.run_app_requested);
    assert!(!transcript.completed_cleanly());
}

#[test]
fn facade_run_until_close_transcript_has_expected_shape() {
    let _f: fn(
        NativeBackendWinitApp,
    ) -> Result<
        NativeBackendWinitAppRunTranscript,
        prom_ui_backend_native::winit_placeholder::NativeBackendWinitAppError,
    > = NativeBackendWinitApp::run_until_close_transcript;
}

#[test]
#[ignore = "manual NativeBackendWinitApp transcript smoke; opens a native window"]
fn manual_native_backend_winit_app_facade_returns_transcript() {
    let mut backend = NativeBackend::new();
    let config = WindowConfig::new("Semantic Native Transcript", 800, 600);

    backend.create_window(&config).unwrap();

    let facade = NativeBackendWinitApp::new(backend).unwrap();

    let transcript = facade
        .run_until_close_transcript()
        .expect("manual NativeBackendWinitApp transcript run should complete");

    assert_eq!(
        transcript.status,
        NativeBackendWinitAppRunTranscriptStatus::Completed
    );
    assert!(transcript.completed_cleanly());
    assert!(transcript.close_requested);
    assert!(transcript.staged_event_count >= 1);
}
