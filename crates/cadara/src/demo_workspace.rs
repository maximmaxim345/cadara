//! Demo workspace for testing the workspace system.
//!
//! Draws a simple blue triangle in the viewport.
use computegraph::{node, ComputeGraph};
use iced::widget::shader;
use viewport::{
    DynamicViewportPlugin, InputEvents, RenderNodePorts, SceneGraph, SceneGraphBuilder,
    UpdateNodePorts,
};
use workspace::Workspace;

#[derive(Default)]
struct SomeState {
    i: usize,
}

/// Node that adds the Shader primitive to the final scene graph
#[derive(Clone, Debug)]
struct RenderNode {}

#[node(RenderNode)]
fn run(&self, state: &SomeState) -> Box<dyn shader::Primitive> {
    Box::new(BlueTrianglePrimitive {})
}

#[derive(Clone, Debug)]
struct InitState {}

#[node(InitState)]
fn run(&self) -> SomeState {
    SomeState { i: 0 }
}

#[derive(Clone, Debug)]
struct UpdateNode {}

#[node(UpdateNode)]
fn run(&self, _events: &InputEvents, state: &SomeState) -> SomeState {
    SomeState { i: state.i + 1 }
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
struct BlueTrianglePipeline {
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
struct DemoViewportPlugin {}
#[node(DemoViewportPlugin -> (scene, output))]
fn run(&self) -> (SceneGraph, DemoViewportPluginOutput) {
    let mut graph = ComputeGraph::new();
    let render_node = graph.add_node(RenderNode {}, "render".to_string()).unwrap();
    let update_node = graph.add_node(UpdateNode {}, "update".to_string()).unwrap();
    let init_node = graph.add_node(InitState {}, "init".to_string()).unwrap();

    (
        SceneGraphBuilder {
            graph,
            initial_state: init_node.output(),
            render_node: RenderNodePorts {
                state_in: render_node.input_state(),
                primitive_out: render_node.output(),
            },
            update_node: UpdateNodePorts {
                events_in: update_node.input_events(),
                state_in: update_node.input_state(),
                state_out: update_node.output(),
            },
        }
        .into(),
        DemoViewportPluginOutput {},
    )
}

#[derive(Debug, Default)]
pub struct DemoWorkspace {}

impl Workspace for DemoWorkspace {
    fn tools(&self) -> Vec<workspace::Toolgroup> {
        use workspace::{Action, Tool, Toolgroup};
        vec![Toolgroup {
            name: "Some Group".to_string(),
            tools: vec![Tool {
                name: "Some Tool".to_string(),
                action: Action(),
            }],
        }]
    }

    fn viewport_plugins(&self) -> Vec<viewport::DynamicViewportPlugin> {
        vec![DynamicViewportPlugin::new(DemoViewportPlugin::default().into()).unwrap()]
    }
}
