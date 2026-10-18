pub mod cad_pipeline;
pub mod probe_pipeline;
pub mod picker;

use std::sync::Arc;
use wgpu::util::DeviceExt;
use glam::{Mat4, Vec3};
use crate::cad::importer::TessellatedCadMesh;
use cad_pipeline::CadRenderPipeline;
use probe_pipeline::ProbeRenderPipeline;
use picker::RaycastPicker;

pub struct CameraUniforms {
    pub view_proj: Mat4,
    pub eye_pos: Vec3,
}

pub struct MetrologyRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    cad_pipeline: CadRenderPipeline,
    probe_pipeline: ProbeRenderPipeline,
    pub picker: RaycastPicker,

    // Uniform Buffers
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
}

impl MetrologyRenderer {
    pub async fn new(window: Arc<winit::window::Window>, width: u32, height: u32) -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12 | wgpu::Backends::VULKAN,
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.ok_or_else(|| anyhow::anyhow!("Uygun GPU adaptörü bulunamadı"))?;

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("CMM_GPU_Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
        }, None).await?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Kamera Matrisi Uniform
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera_Uniform_Buffer"),
            size: std::mem::size_of::<[f32; 16]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera_Bind_Group_Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera_Bind_Group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let cad_pipeline = CadRenderPipeline::new(&device, format, &camera_bind_group_layout);
        let probe_pipeline = ProbeRenderPipeline::new(&device, format, &camera_bind_group_layout);
        let picker = RaycastPicker::new();

        Ok(Self {
            surface,
            device,
            queue,
            config,
            cad_pipeline,
            probe_pipeline,
            picker,
            camera_buffer,
            camera_bind_group,
        })
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_width > 0 && new_height > 0 {
            self.config.width = new_width;
            self.config.height = new_height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn update_camera(&self, view_proj: &Mat4) {
        let raw_matrix: [f32; 16] = view_proj.to_cols_array();
        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&raw_matrix));
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("CMM_Render_Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main_Render_Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.08,
                            g: 0.08,
                            b: 0.10,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.cad_pipeline.draw(&mut render_pass, &self.camera_bind_group);
            self.probe_pipeline.draw(&mut render_pass, &self.camera_bind_group);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}
