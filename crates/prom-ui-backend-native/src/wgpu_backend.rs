#[cfg(feature = "winit-backend")]
use alloc::sync::Arc;
use alloc::vec::Vec;
use pollster::block_on;
use winit::window::Window;

use crate::draw_backend::UiDrawBackend;
use crate::sink::UiFrameSink;
use prom_ui::paint::UiDrawCommand;

/// The Wgpu draw backend responsible for painting `UiDrawCommand`s
/// onto a `winit` native window.
pub struct WgpuDrawBackend {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

impl UiDrawBackend for WgpuDrawBackend {
    fn initialize(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        
        // Because we pass `Arc<Window>`, wgpu infers a `'static` lifetime for the surface.
        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface");

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .expect("Failed to find wgpu adapter");

        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
            },
            None,
        ))
        .expect("Failed to request device");

        let size = window.inner_size();
        let mut width = size.width;
        let mut height = size.height;
        if width == 0 {
            width = 1;
        }
        if height == 0 {
            height = 1;
        }

        let config = surface
            .get_default_config(&adapter, width, height)
            .expect("Surface unsupported by adapter");

        surface.configure(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
        }
    }

    fn sink(&mut self) -> &mut dyn UiFrameSink {
        self
    }
}

impl UiFrameSink for WgpuDrawBackend {
    fn submit_frame(&mut self, commands: Vec<UiDrawCommand>) {
        let output = match self.surface.get_current_texture() {
            Ok(tex) => tex,
            Err(_) => {
                // Surface lost or outdated; winit resize event will reconfigure it soon.
                return;
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut clear_color = wgpu::Color::BLACK;

        for cmd in commands {
            match cmd {
                UiDrawCommand::FillRect { rect: _, color } => {
                    clear_color = wgpu::Color {
                        r: f64::from(color.r) / 255.0,
                        g: f64::from(color.g) / 255.0,
                        b: f64::from(color.b) / 255.0,
                        a: f64::from(color.a) / 255.0,
                    };
                }
                UiDrawCommand::DrawText { .. } => {}
            }
        }

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        self.queue.submit(Some(encoder.finish()));
        output.present();
    }
}
