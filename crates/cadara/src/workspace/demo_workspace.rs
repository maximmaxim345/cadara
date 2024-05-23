//! Demo workspace for testing the workspace system.
//!
//! Draws a simple blue triangle in the viewport.
use computegraph::{node, ComputeGraph};
use iced::widget::shader;
use viewport::SceneGraph;

/// Node that adds the Shader primitive to the final scene graph
#[derive(Clone, Debug)]
struct RenderNode {}

#[node(RenderNode)]
fn run(&self) -> Box<dyn shader::Primitive> {
    Box::new(BlueTrianglePrimitive {})
}

/// Shader primitive that renders a blue triangle
#[derive(Debug)]
struct BlueTrianglePrimitive {}

impl shader::Primitive for BlueTrianglePrimitive {
    fn prepare(
        &self,
        format: wgpu::TextureFormat,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _bounds: iced::Rectangle,
        target_size: iced::Size<u32>,
        _scale_factor: f32,
        storage: &mut shader::Storage,
    ) {
        if !storage.has::<BlueTrianglePipeline>() {
            storage.store(BlueTrianglePipeline::new(
                device,
                queue,
                format,
                target_size,
            ));
        }
        let _pipeline = storage.get_mut::<BlueTrianglePipeline>().unwrap();
    }

    fn render(
        &self,
        storage: &shader::Storage,
        target: &wgpu::TextureView,
        _target_size: iced::Size<u32>,
        viewport: iced::Rectangle<u32>,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let pipeline = storage.get::<BlueTrianglePipeline>().unwrap();

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

        render_pass.set_pipeline(&pipeline.pipeline);
        render_pass.set_viewport(
            viewport.x as f32,
            viewport.y as f32,
            viewport.width as f32,
            viewport.height as f32,
            0.0,
            1.0,
        );
        render_pass.draw(0..3, 0..1);
    }
}

/// Pipeline for rendering the blue triangle, used by the [`BlueTrianglePrimitive`] iced primitive.
#[derive(Debug)]
pub struct BlueTrianglePipeline {
    pipeline: wgpu::RenderPipeline,
}

impl BlueTrianglePipeline {
    fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        _target_size: iced::Size<u32>,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                r"
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(1 - i32(in_vertex_index & 1u) * 2);
    let y = f32(1 - i32(in_vertex_index >> 1u) * 2);
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.tex_coords = vec2<f32>(
        f32(in_vertex_index & 1u),
        f32(in_vertex_index >> 1u),
    );
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 1.0, 1.0);
}
        ",
            )),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
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

        Self { pipeline }
    }
}

/// Output that will be passed to the next `ViewportPlugin` after [`DemoViewportPlugin`]
#[derive(Clone, Debug)]
pub struct DemoViewportPluginOutput {}

/// Simple demo `ViewportPlugin` that renders a blue triangle.
#[derive(Clone, Default, Debug)]
pub struct DemoViewportPlugin {}
#[node(DemoViewportPlugin -> (scene, output))]
fn run(&self) -> (SceneGraph, DemoViewportPluginOutput) {
    let mut graph = ComputeGraph::new();
    let node = graph.add_node(RenderNode {}, "a name".to_string()).unwrap();

    (
        SceneGraph {
            graph,
            primitive: node.output(),
        },
        DemoViewportPluginOutput {},
    )
}
