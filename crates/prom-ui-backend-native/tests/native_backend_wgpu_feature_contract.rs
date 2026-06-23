#![cfg(feature = "wgpu-backend")]

use prom_ui_backend_native::{selected_draw_backend_name, wgpu_backend_feature_enabled};

#[test]
fn wgpu_backend_feature_exposes_selected_draw_backend_name() {
    assert!(wgpu_backend_feature_enabled());
    assert_eq!(selected_draw_backend_name(), "wgpu");
}

#[test]
fn native_backend_wgpu_context_minimal_draw_smoke() {
    // Attempt to initialize the wgpu context.
    // In headless CI environments without a GPU or Lavapipe, this might legitimately return None.
    // We gracefully skip the test if no adapter is found, rather than panicking.
    if let Some(context) = prom_ui_backend_native::wgpu_integration::NativeBackendWgpuContext::new()
    {
        // Execute the minimal draw pass. This proves we can create a dummy texture,
        // begin a render pass, clear it, and submit the encoder to the queue.
        context.execute_minimal_draw_pass();
    } else {
        println!(
            "Skipping wgpu draw smoke test: No suitable GPU adapter found in this environment."
        );
    }
}
