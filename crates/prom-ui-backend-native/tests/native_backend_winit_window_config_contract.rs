#![cfg(feature = "winit-backend")]

use prom_ui_backend_native::winit_placeholder::{
    window_config_to_winit_attributes, winit_window_config_translation_available,
};
use prom_ui_runtime::WindowConfig;
use winit::{dpi::Size, window::WindowAttributes};

#[test]
fn winit_window_config_translation_scaffold_is_available() {
    assert!(prom_ui_backend_native::winit_backend_feature_enabled());
    assert!(winit_window_config_translation_available());
}

#[test]
fn window_config_translates_title() {
    let config = WindowConfig::new("Semantic Native", 800, 600);

    let attrs = window_config_to_winit_attributes(&config);

    assert_eq!(attrs.title, "Semantic Native");
}

#[test]
fn window_config_translates_inner_size() {
    let config = WindowConfig::new("Size", 1024, 768);

    let attrs = window_config_to_winit_attributes(&config);

    let size = attrs
        .inner_size
        .expect("inner_size must be set by translation");

    match size {
        Size::Logical(logical) => {
            assert_eq!(logical.width, 1024.0);
            assert_eq!(logical.height, 768.0);
        }
        other => panic!("expected logical size, got {other:?}"),
    }
}

#[test]
fn window_config_translation_sets_safe_defaults() {
    let config = WindowConfig::new("Defaults", 640, 480);

    let attrs = window_config_to_winit_attributes(&config);

    assert!(attrs.visible);
    assert!(attrs.resizable);
}

#[test]
fn window_config_translation_returns_winit_window_attributes() {
    fn assert_attrs(_: WindowAttributes) {}

    let config = WindowConfig::new("TypeCheck", 320, 240);
    let attrs = window_config_to_winit_attributes(&config);

    assert_attrs(attrs);
}
