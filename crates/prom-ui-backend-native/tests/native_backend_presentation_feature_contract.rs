#![cfg(all(feature = "wgpu-backend", feature = "winit-backend"))]

use prom_ui_backend_native::wgpu_integration::NativeBackendPresentationSurface;

#[test]
fn native_backend_presentation_surface_is_available() {
    // This is purely a compile-time check to ensure the struct exists
    // under the combined feature flags. We do not try to initialize winit
    // or wgpu context here because creating a real winit Window in headless
    // CI without a real event loop is often fraught.
    let _ = core::mem::size_of::<NativeBackendPresentationSurface>();
}
