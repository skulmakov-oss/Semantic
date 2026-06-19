#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use pollster::block_on;
use prom_ui::layout::UiRect;
use prom_ui::paint::{UiColor, UiDrawCommand};
use prom_ui_backend_native::sink::UiFrameSink;

struct HeadlessSink {
    device: wgpu::Device,
    queue: wgpu::Queue,
    texture_view: wgpu::TextureView,
}

impl UiFrameSink for HeadlessSink {
    fn submit_frame(&mut self, commands: alloc::vec::Vec<UiDrawCommand>) {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Headless Encoder"),
        });

        // Use the first FillRect command's color as the clear color for this test
        let mut clear_color = wgpu::Color::BLACK;
        for cmd in commands {
            if let UiDrawCommand::FillRect { color, .. } = cmd {
                clear_color = wgpu::Color {
                    r: color.r as f64 / 255.0,
                    g: color.g as f64 / 255.0,
                    b: color.b as f64 / 255.0,
                    a: color.a as f64 / 255.0,
                };
                break;
            }
        }

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Headless Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.texture_view,
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
            // Just clearing the buffer validates wgpu rendering a frame
        }

        self.queue.submit(Some(encoder.finish()));
    }
}

#[test]
fn test_offscreen_draw_sink() {
    block_on(async {
        let instance = wgpu::Instance::default();
        
        let adapter_opt = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }).await;

        // In CI without a GPU driver, adapter might be None. We gracefully ignore to prevent CI flakes.
        if let Some(adapter) = adapter_opt {
            let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default(), None).await.unwrap();

            let texture = device.create_texture(&wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width: 256,
                    height: 256,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                label: Some("Offscreen Texture"),
                view_formats: &[],
            });

            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            let mut sink = HeadlessSink {
                device,
                queue,
                texture_view,
            };

            let commands = vec![
                UiDrawCommand::FillRect {
                    rect: UiRect::from_xywh(0, 0, 100, 100),
                    color: UiColor::rgb(255, 0, 0), // Red
                },
                UiDrawCommand::DrawText {
                    rect: UiRect::from_xywh(10, 10, 80, 20),
                    text: alloc::string::String::from("Test"),
                    color: UiColor::rgb(255, 255, 255), // White
                },
            ];

            sink.submit_frame(commands);
        }
    });
}
