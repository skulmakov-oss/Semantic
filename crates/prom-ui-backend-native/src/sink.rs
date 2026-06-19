use alloc::vec::Vec;
use prom_ui::paint::UiDrawCommand;

/// A trait that defines how a backend consumes and renders a frame.
///
/// This isolates the core layout logic from knowing about `wgpu::Device`,
/// `wgpu::TextureView`, or swapchains. The UI core will simply pass a
/// `Vec<UiDrawCommand>` to the sink for rendering.
pub trait UiFrameSink {
    fn submit_frame(&mut self, commands: Vec<UiDrawCommand>);
}
