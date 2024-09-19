#![allow(clippy::cast_precision_loss)]

use iced::widget::shader::{
    self,
    wgpu::{
        self,
        util::{BufferInitDescriptor, DeviceExt},
    },
};

use super::state::{Uniforms, Vertex, ViewportState};

#[derive(Debug)]
pub struct RenderPrimitive {
    pub(crate) state: ViewportState,
    pub(crate) mesh: occara::shape::Mesh,
}

impl shader::Primitive for RenderPrimitive {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        storage: &mut shader::Storage,
        bounds: &iced::Rectangle,
        viewport: &iced::widget::shader::Viewport,
    ) {
        if !storage.has::<RenderPipeline>() {
            storage.store(RenderPipeline::new(
                device,
                queue,
                format,
                viewport.physical_size(),
            ));
        }
        let pipeline = storage.get_mut::<RenderPipeline>().unwrap();
        let mut a = Vec::new();
        let indices = self.mesh.indices();
        let vertices = self.mesh.vertices();
        for i in indices {
            let v = Vertex {
                pos: glam::Vec3::new(
                    vertices[i].x() as f32,
                    vertices[i].y() as f32,
                    vertices[i].z() as f32,
                ),
            };
            a.push(v);
        }
        pipeline.update(
            device,
            queue,
            *bounds,
            viewport.physical_size(),
            viewport.scale_factor() as f32,
            self,
            &a,
        );
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        storage: &shader::Storage,
        target: &wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        let pipeline = storage.get::<RenderPipeline>().unwrap();

        pipeline.render(encoder, target, *clip_bounds);
    }
}

#[derive(Debug)]
struct RenderPipeline {
    pipeline: wgpu::RenderPipeline,
    camera_bind_group: wgpu::BindGroup,
    uniforms: wgpu::Buffer,
    mesh_buffer: Option<wgpu::Buffer>,
}

impl RenderPipeline {
    fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        _target_size: iced::Size<u32>,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                "shader.wgsl"
            ))),
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            pipeline,
            camera_bind_group,
            uniforms,
            mesh_buffer: None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: iced::Rectangle,
        _target_size: iced::Size<u32>,
        _scale_factor: f32,
        primitive: &RenderPrimitive,
        v: &[Vertex],
    ) {
        let p = primitive.state.camera_offset;
        let x = ((p.x - bounds.x) / bounds.width).mul_add(2.0, -1.0);
        let y = ((p.y - bounds.y) / bounds.height).mul_add(-2.0, 1.0);
        let uniforms = Uniforms { x, y };
        queue.write_buffer(&self.uniforms, 0, bytemuck::cast_slice(&[uniforms]));
        let mesh_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("mesh_buffer"),
            usage: wgpu::BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(v),
        });
        self.mesh_buffer = Some(mesh_buffer);
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        viewport: iced::Rectangle<u32>,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_viewport(
            viewport.x as f32,
            viewport.y as f32,
            viewport.width as f32,
            viewport.height as f32,
            0.0,
            1.0,
        );
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        let m = self.mesh_buffer.as_ref().unwrap();
        render_pass.set_vertex_buffer(0, m.slice(..));
        render_pass.draw(0..1884, 0..1);
    }
}
