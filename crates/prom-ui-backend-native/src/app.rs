#[cfg(feature = "winit-backend")]
pub mod winit_app {
    use alloc::sync::Arc;
    use winit::application::ApplicationHandler;
    use winit::event::WindowEvent;
    use winit::event_loop::ActiveEventLoop;
    use winit::window::{Window, WindowId};

    /// A minimal seed for the native windowing application.
    /// This is not yet connected to the `prom-ui` core loop.
    pub struct NativeApp {
        window: Option<Arc<Window>>,
    }

    impl NativeApp {
        pub fn new() -> Self {
            Self { window: None }
        }
        
        pub fn window(&self) -> Option<Arc<Window>> {
            self.window.clone()
        }
    }

    impl Default for NativeApp {
        fn default() -> Self {
            Self::new()
        }
    }

    impl ApplicationHandler for NativeApp {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            // Winit 0.30 requires creating the window inside `resumed`
            if self.window.is_none() {
                let attrs = Window::default_attributes()
                    .with_title("Semantic UI Native Seed");
                if let Ok(window) = event_loop.create_window(attrs) {
                    self.window = Some(Arc::new(window));
                }
            }
        }

        fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            window_id: WindowId,
            event: WindowEvent,
        ) {
            if let Some(window) = &self.window {
                if window.id() == window_id {
                    match event {
                        WindowEvent::CloseRequested => {
                            event_loop.exit();
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
