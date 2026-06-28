#![cfg(all(feature = "winit-backend", feature = "wgpu-backend"))]

use prom_ui_backend_native::wgpu_integration::{
    NativeBackendPresentationSurface, NativeBackendWgpuContext,
};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

struct StaticDemoApp {
    wgpu_context: NativeBackendWgpuContext,
    window: Option<Arc<Window>>,
    surface: Option<NativeBackendPresentationSurface>,
}

impl StaticDemoApp {
    fn new() -> Option<Self> {
        let wgpu_context = NativeBackendWgpuContext::new()?;
        Some(Self {
            wgpu_context,
            window: None,
            surface: None,
        })
    }
}

impl ApplicationHandler for StaticDemoApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("Semantic UI - Static Visible Demo")
                .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0));

            if let Ok(window) = event_loop.create_window(window_attributes) {
                let window = Arc::new(window);
                self.window = Some(window.clone());

                let surface = NativeBackendPresentationSurface::new(
                    &self.wgpu_context,
                    window.clone(),
                    800,
                    600,
                )
                .expect("Failed to create presentation surface");

                self.surface = Some(surface);

                // Request the first frame
                window.request_redraw();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(surface) = &mut self.surface {
                    surface.resize(
                        &self.wgpu_context,
                        physical_size.width,
                        physical_size.height,
                    );
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(surface) = &self.surface {
                    let _success = surface.present_minimal_clear(&self.wgpu_context);
                }
            }
            _ => {}
        }
    }
}

#[test]
#[ignore = "manual visible UI demo; requires desktop session"]
fn manual_static_visible_demo() {
    let mut app = match StaticDemoApp::new() {
        Some(app) => app,
        None => {
            println!("Skipping static visible demo: No suitable GPU adapter found.");
            return;
        }
    };

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let _ = event_loop.run_app(&mut app);
}
