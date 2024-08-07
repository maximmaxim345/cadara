use computegraph::{node, ComputeGraph};
use iced::widget::shader;
use viewport::DynamicViewportPlugin;
use viewport::SceneGraph;
use workspace::Workspace;

#[derive(Debug, Default, Clone)]
pub struct ModelingViewportPlugin {}

#[derive(Clone, Debug)]
pub struct RenderNode {}

#[derive(Debug)]
struct Primitive {}
impl shader::Primitive for Primitive {
    fn prepare(
        &self,
        _format: shader::wgpu::TextureFormat,
        _device: &shader::wgpu::Device,
        _queue: &shader::wgpu::Queue,
        _bounds: iced::Rectangle,
        _target_size: iced::Size<u32>,
        _scale_factor: f32,
        _storage: &mut shader::Storage,
    ) {
    }

    fn render(
        &self,
        _storage: &shader::Storage,
        _target: &shader::wgpu::TextureView,
        _target_size: iced::Size<u32>,
        _viewport: iced::Rectangle<u32>,
        _encoder: &mut shader::wgpu::CommandEncoder,
    ) {
    }
}

#[node(RenderNode)]
fn run(&self) -> Box<dyn shader::Primitive> {
    Box::new(Primitive {})
}

#[node(ModelingViewportPlugin -> (scene, output))]
fn run(&self) -> (SceneGraph, ()) {
    let mut graph = ComputeGraph::new();
    let node = graph.add_node(RenderNode {}, "a name".to_string()).unwrap();

    (todo!(), ())
}

#[derive(Debug, Clone, Default)]
pub struct ModelingWorkspace {}

impl Workspace for ModelingWorkspace {
    fn tools(&self) -> Vec<workspace::Toolgroup> {
        vec![]
    }

    fn viewport_plugins(&self) -> Vec<DynamicViewportPlugin> {
        vec![DynamicViewportPlugin::new(ModelingViewportPlugin::default().into()).unwrap()]
    }
}
