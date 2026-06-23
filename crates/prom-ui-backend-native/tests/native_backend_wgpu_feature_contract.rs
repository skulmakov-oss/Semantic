#![cfg(feature = "wgpu-backend")]

use prom_ui_backend_native::{selected_draw_backend_name, wgpu_backend_feature_enabled};

#[test]
fn wgpu_backend_feature_exposes_selected_draw_backend_name() {
    assert!(wgpu_backend_feature_enabled());
    assert_eq!(selected_draw_backend_name(), "wgpu");
}
