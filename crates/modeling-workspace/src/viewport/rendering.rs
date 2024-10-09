use iced::widget::shader::{
    self,
    wgpu::{self, util::DeviceExt, DepthStencilState, RenderPassDepthStencilAttachment},
};

use super::{
    camera::{Camera, CameraUniform},
    state::ViewportState,
};

#[derive(Debug)]
pub struct RenderPrimitive {
    pub state: ViewportState,
    pub mesh: occara::shape::Mesh,
}

#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vertex {
    pub pos: glam::Vec3,
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![
        0 => Float32x3,
    ];

    pub const fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

impl shader::Primitive for RenderPrimitive {
    fn prepare(
        &self,
        format: wgpu::TextureFormat,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: iced::Rectangle,
        target_size: iced::Size<u32>,
        scale_factor: f32,
        storage: &mut shader::Storage,
    ) {
        if !storage.has::<RenderPipeline>() {
            storage.store(RenderPipeline::new(device, queue, format, target_size));
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
        pipeline.update(device, queue, bounds, target_size, scale_factor, self, &a);
    }

    fn render(
        &self,
        storage: &shader::Storage,
        target: &wgpu::TextureView,
        target_size: iced::Size<u32>,
        viewport: iced::Rectangle<u32>,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let pipeline = storage.get::<RenderPipeline>().unwrap();

        pipeline.render(encoder, target, target_size, viewport);
    }
}

#[derive(Debug)]
struct RenderPipeline {
    pipeline: wgpu::RenderPipeline,
    camera_bind_group: wgpu::BindGroup,
    camera: wgpu::Buffer,
    mesh_buffer: Option<wgpu::Buffer>,
    vertex_count: u32,
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
}

impl RenderPipeline {
    fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        target_size: iced::Size<u32>,
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
        let camera = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera.as_entire_binding(),
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
            depth_stencil: Some(DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: target_size.width,
                height: target_size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            label: Some("depth_texture"),
            view_formats: &[],
        });

        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            pipeline,
            camera_bind_group,
            camera,
            mesh_buffer: None,
            vertex_count: 0,
            depth_texture,
            depth_texture_view,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: iced::Rectangle,
        target_size: iced::Size<u32>,
        _scale_factor: f32,
        primitive: &RenderPrimitive,
        v: &[Vertex],
    ) {
        let mut camera: Camera = primitive.state.camera.clone();
        camera.set_aspect(bounds.width, bounds.height);
        let camera_uniform = CameraUniform::from(&camera);

        queue.write_buffer(&self.camera, 0, bytemuck::cast_slice(&[camera_uniform]));

        // Create and update mesh buffer
        let mesh_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Buffer"),
            contents: bytemuck::cast_slice(v),
            usage: wgpu::BufferUsages::VERTEX,
        });
        self.mesh_buffer = Some(mesh_buffer);
        self.vertex_count = v.len() as u32;

        // Update depth texture if target size changed
        if self.depth_texture.size().width != target_size.width
            || self.depth_texture.size().height != target_size.height
        {
            self.depth_texture = device.create_texture(&wgpu::TextureDescriptor {
                size: wgpu::Extent3d {
                    width: target_size.width,
                    height: target_size.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                label: Some("depth_texture"),
                view_formats: &[],
            });
            self.depth_texture_view = self
                .depth_texture
                .create_view(&wgpu::TextureViewDescriptor::default());
        }
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        _target_size: iced::Size<u32>,
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
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);

        #[allow(clippy::cast_precision_loss)]
        render_pass.set_viewport(
            viewport.x as f32,
            viewport.y as f32,
            viewport.width as f32,
            viewport.height as f32,
            0.0,
            1.0,
        );
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        if let Some(m) = self.mesh_buffer.as_ref() {
            render_pass.set_vertex_buffer(0, m.slice(..));
            render_pass.draw(0..self.vertex_count, 0..1);
        }
    }
}
