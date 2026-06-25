use prom_ui_backend_native::{
    native_run_loop_safety_lock_available, winit_backend_feature_enabled, NativeBackend,
};
use prom_ui_runtime::{InputEvent, InputEventKind, LoopControl, UiBackendAdapter};

#[test]
fn native_run_loop_safety_lock_contract() {
    assert!(native_run_loop_safety_lock_available());

    // Check presentation path existence constraint (wgpu-backend + winit-backend)
    // Here we can just assert that if wgpu is enabled, it means wgpu surface/presentation is guarded.
    // The test shouldn't be able to access PresentationSurface if the features are off.

    // We test the deterministic offline loop behavior.
    if winit_backend_feature_enabled() {
        // If winit is enabled, run_event_loop tries to create a real EventLoop.
        // We only check initial state and that NativeBackend remains a transport boundary.
        let backend = NativeBackend::new();
        assert_eq!(backend.run_loop_calls(), 0);
        return;
    }

    let mut backend = NativeBackend::new();

    // Stage events to prove the backend acts only as a transport boundary
    // and does NOT attempt to parse semantic intent or dispatch actions.
    backend.push_pending_event(InputEvent {
        kind: InputEventKind::KeyDown { key_code: 10 },
    });
    backend.push_pending_event(InputEvent {
        kind: InputEventKind::CloseRequested,
    });
    // This event should NOT be processed because CloseRequested breaks the loop.
    backend.push_pending_event(InputEvent {
        kind: InputEventKind::KeyDown { key_code: 20 },
    });

    let mut loop_controls = Vec::new();

    // The backend only takes a closure and yields `LoopControl`.
    // It has NO access to `InteractionPipeline`, NO capability authority,
    // and NO ability to dispatch `SemanticIntent`.
    backend
        .run_event_loop(|control, _frame| {
            loop_controls.push(control);
        })
        .expect("Deterministic run_event_loop should succeed");

    // Verify correct tracking of loop metrics
    assert_eq!(backend.run_loop_calls(), 1);
    assert_eq!(backend.run_loop_ticks(), 2);

    // Verify correct translation to LoopControl without semantic interference
    assert_eq!(loop_controls.len(), 2);
    assert!(matches!(loop_controls[0], LoopControl::Continue));
    assert!(matches!(loop_controls[1], LoopControl::ExitRequested));
}
