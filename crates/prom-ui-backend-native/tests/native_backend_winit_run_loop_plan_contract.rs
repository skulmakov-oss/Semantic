#![cfg(feature = "winit-backend")]

use prom_ui_backend_native::{
    winit_placeholder::{
        current_native_backend_winit_run_loop_plan,
        native_backend_winit_run_loop_plan_available,
        native_backend_winit_run_loop_readiness,
        NativeBackendWinitRunLoopOwnership,
        NativeBackendWinitRunLoopReadiness,
        NativeBackendWinitRunLoopStage,
    },
    NativeBackend,
};
use prom_ui_runtime::{UiBackendAdapter, WindowConfig};

#[test]
fn native_backend_winit_run_loop_plan_is_available() {
    assert!(prom_ui_backend_native::winit_backend_feature_enabled());
    assert!(native_backend_winit_run_loop_plan_available());
}

#[test]
fn current_plan_is_manual_app_state_run() {
    let plan = current_native_backend_winit_run_loop_plan();

    assert_eq!(plan.stage, NativeBackendWinitRunLoopStage::ManualAppStateRun);
    assert_eq!(
        plan.ownership,
        NativeBackendWinitRunLoopOwnership::ConsumesBackendReturnsSummary
    );

    assert!(plan.requires_staged_window_config);
    assert!(plan.uses_native_backend_app_state);
    assert!(plan.creates_event_loop);
    assert!(plan.may_create_native_window);
    assert!(plan.may_call_run_app);
}

#[test]
fn current_plan_does_not_integrate_backend_run_event_loop_yet() {
    let plan = current_native_backend_winit_run_loop_plan();

    assert!(!plan.integrates_with_backend_run_event_loop);
    assert!(!plan.mutates_prom_ui_runtime);
    assert!(!plan.changes_ui_backend_adapter);
    assert!(plan.keeps_runtime_boundary_clean());
}

#[test]
fn current_plan_excludes_renderer_and_frame_presentation() {
    let plan = current_native_backend_winit_run_loop_plan();

    assert!(!plan.includes_renderer);
    assert!(!plan.presents_frames);
    assert!(plan.keeps_rendering_out_of_scope());
}

#[test]
fn current_plan_is_g16_admissible() {
    let plan = current_native_backend_winit_run_loop_plan();

    assert!(plan.is_g16_admissible());
}

#[test]
fn run_loop_readiness_requires_staged_window_config() {
    let backend = NativeBackend::new();

    assert_eq!(
        native_backend_winit_run_loop_readiness(&backend),
        NativeBackendWinitRunLoopReadiness::MissingWindowConfig
    );
}

#[test]
fn run_loop_readiness_accepts_staged_window_config() {
    let mut backend = NativeBackend::new();
    let config = WindowConfig::new("RunLoop Plan", 800, 600);

    backend.create_window(&config).unwrap();

    assert_eq!(
        native_backend_winit_run_loop_readiness(&backend),
        NativeBackendWinitRunLoopReadiness::ReadyForManualAppStateRun
    );
}

#[test]
fn run_loop_readiness_does_not_mutate_backend_state() {
    let mut backend = NativeBackend::new();
    let config = WindowConfig::new("No Mutation", 640, 480);

    backend.create_window(&config).unwrap();

    let before_pending = backend.pending_event_count();
    let before_frames = backend.submitted_frames();
    let before_run_calls = backend.run_loop_calls();

    let readiness = native_backend_winit_run_loop_readiness(&backend);

    assert_eq!(
        readiness,
        NativeBackendWinitRunLoopReadiness::ReadyForManualAppStateRun
    );
    assert_eq!(backend.pending_event_count(), before_pending);
    assert_eq!(backend.submitted_frames(), before_frames);
    assert_eq!(backend.run_loop_calls(), before_run_calls);
    assert!(!backend.is_platform_wired());
}
