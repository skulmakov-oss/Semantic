#[cfg(not(feature = "winit-backend"))]
#[test]
fn winit_backend_feature_is_disabled_by_default() {
    assert!(!prom_ui_backend_native::winit_backend_feature_enabled());
}

#[cfg(feature = "winit-backend")]
#[test]
fn winit_backend_feature_exposes_placeholder_module() {
    assert!(prom_ui_backend_native::winit_backend_feature_enabled());
    assert!(prom_ui_backend_native::winit_placeholder::WINIT_BACKEND_PLACEHOLDER);
    assert!(prom_ui_backend_native::winit_placeholder::winit_backend_placeholder_enabled());
}
