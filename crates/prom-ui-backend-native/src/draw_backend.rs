#[cfg(feature = "winit-backend")]
use winit::window::Window;

use crate::sink::UiFrameSink;

use alloc::sync::Arc;

/// Abstract draw backend selection to decouple the windowing loop
/// from the specific GPU API (like `wgpu`) used for rendering.
#[cfg(feature = "winit-backend")]
pub trait UiDrawBackend {
    /// Initialize the draw backend using the given native window.
    /// 
    /// This is where `wgpu::Surface` and related graphics instances
    /// will be created.
    fn initialize(window: Arc<Window>) -> Self;

    /// Obtain the frame sink to submit drawing commands.
    fn sink(&mut self) -> &mut dyn UiFrameSink;
}
