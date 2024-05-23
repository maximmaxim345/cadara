//! # Viewport
//!
//! The viewport is the central UI component for rendering and interacting with documents in `CADara`.
//! It manages a scene graph and coordinates plugins and extensions to enable rich visualization and editing.
//!
//! ## Scene Graph
//!
//! Instead of directly rendering a scene, the user of this library can define a scene graph using a [`ComputeGraph`] struct.
//!
//! This allows for a declarative approach to defining scenes, handling asynchronous operations and caching seamlessly.
//! TODO: update this when it's clear how the scene graph will work.
//!
//! ## Building the Scene Graph
//!
//! The scene graph can be modified in two ways: plugins and extensions.
//!
//! ### Plugins
//!
//! Plugins are the main way to modify the scene graph, and often correspond to a specific workspace.
//! Multiple plugins can be added to the viewport and will be executed in order.
//! Plugins come in two forms: add and replace.
//! - Add plugins receive the scene graph from the previous plugin and can fully modify it.
//! - Replace plugins first reset the scene graph before executing, effectively replacing all previous plugins.
//!
//! This allows Plugins to modify the scene graph incrementally or completely replace it.
//! Imagine editing a Sketch in a CAD document: the viewport should still show the rest of the document, while
//! displaying extra information and tools specific to the selected Sketch.
//!
//! ### Extensions
//!
//! Nodes (or subgraphs) in the scene graph can be annotated with extensions. Extensions are run by plugins (through a helper library)
//! TODO: I have no idea yet about how extensions should function (if at all)
//!
//! ### Render Nodes
//!
//! Render nodes are responsible for rendering the scene every frame and should be as lightweight as possible.
//! A node can be marked as a render node by setting the `render` flag to true. TODO: or is it metadata?
//! The viewport will automatically the `context` input port with the rendering context. TODO: update when it's clear how this works.
//!
//! ### Edges
//!
//! An edge connects the output of one node to the input of another.
//! The type of the output must match the type of the input.
//!
//! ### Execution
//!
//! TODO: move this section to the ComputeGraph documentation, here we should link to it.
//!
//! Depending on the type of the nodes, they are run at different times:
//! - Operation nodes are run when their inputs change or when the external data they listen to changes.
//! - Render nodes are run every frame.
//!
//! If a Operation node hase no path to a Render node, it can be optimized out.
//! This allows for lazy computation and caching.
//!
//! Nodes are only run after the complete scene graph has been built.

use computegraph::ComputeGraph;
use iced::widget::shader;

// TODO: or is this a plugin?
pub trait SceneExtension {
    type Input;
    type Output;

    fn run(&self, scene_graph: ComputeGraph, input: Self::Input) -> (ComputeGraph, Self::Output);
}

#[derive(Clone, Default)]
pub struct Viewport {}

impl<Message> shader::Program<Message> for Viewport {
    type State = ();

    type Primitive = Primitive;

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: iced::advanced::mouse::Cursor,
        _bounds: iced::Rectangle,
    ) -> Self::Primitive {
        Primitive::default()
    }
}

#[derive(Debug, Default)]
pub struct Primitive {}

#[derive(Debug)]
pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
}

impl Pipeline {
    fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        _target_size: iced::Size<u32>,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                r#"
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
        "#,
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

impl shader::Primitive for Primitive {
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
        if !storage.has::<Pipeline>() {
            storage.store(Pipeline::new(device, queue, format, target_size));
        }
        let _pipeline = storage.get_mut::<Pipeline>().unwrap();
    }

    fn render(
        &self,
        storage: &shader::Storage,
        target: &wgpu::TextureView,
        _target_size: iced::Size<u32>,
        viewport: iced::Rectangle<u32>,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let pipeline = storage.get::<Pipeline>().unwrap();

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
