use computegraph::{node, ComputeGraph};
use iced::widget::shader::Primitive;
use viewport::{
    InputEvents, RenderNodePorts, SceneGraph, SceneGraphBuilder, UpdateNodePorts, ViewportPipeline,
};

#[derive(Clone)]
pub struct State {}

#[derive(Debug)]
pub struct SomePrimitive();
impl Primitive for SomePrimitive {
    fn prepare(
        &self,
        _format: wgpu::TextureFormat,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _bounds: iced::Rectangle,
        _target_size: iced::Size<u32>,
        _scale_factor: f32,
        _storage: &mut iced::widget::shader::Storage,
    ) {
    }

    fn render(
        &self,
        _storage: &iced::widget::shader::Storage,
        _target: &wgpu::TextureView,
        _target_size: iced::Size<u32>,
        _viewport: iced::Rectangle<u32>,
        _encoder: &mut wgpu::CommandEncoder,
    ) {
    }
}

#[derive(Debug, Clone)]
pub struct InitState();
#[node(InitState)]
fn run(&self) -> State {
    State {}
}

#[derive(Debug, Clone)]
pub struct RenderNode();
#[node(RenderNode)]
fn run(&self, _state: &State) -> Box<dyn Primitive> {
    Box::new(SomePrimitive())
}

#[derive(Debug, Clone)]
pub struct UpdateNode();
#[node(UpdateNode)]
fn run(&self, state: &State, _events: &InputEvents) -> State {
    (*state).clone()
}

#[derive(Debug, Clone)]
pub struct ConstantNode(usize);

#[node(ConstantNode)]
fn run(&self) -> usize {
    self.0
}

#[derive(Debug, Clone)]
pub struct CounterState {
    output_node: ConstantNodeHandle,
    value: usize,
}

#[derive(Debug, Clone)]
pub struct InitialCounterNode;

#[node(InitialCounterNode -> (scene, output))]
fn run(&self) -> (SceneGraph, CounterState) {
    let mut graph = ComputeGraph::default();
    let output_node = graph
        .add_node(ConstantNode(1), "output".to_string())
        .unwrap();
    let init_state_node = graph.add_node(InitState(), "init".to_string()).unwrap();
    let render_node = graph.add_node(RenderNode(), "render".to_string()).unwrap();
    let update_node = graph.add_node(UpdateNode(), "update".to_string()).unwrap();
    (
        SceneGraphBuilder {
            graph,
            initial_state: init_state_node.output(),
            render_node: RenderNodePorts {
                state_in: render_node.input_state(),
                primitive_out: render_node.output(),
            },
            update_node: UpdateNodePorts {
                state_in: update_node.input_state(),
                events_in: update_node.input_events(),
                state_out: update_node.output(),
            },
        }
        .into(),
        CounterState {
            output_node,
            value: 1,
        },
    )
}

#[derive(Debug, Clone)]
pub struct IncrementCounterNode;

#[node(IncrementCounterNode -> (scene, output))]
fn run(&self, scene: &SceneGraph, input: &CounterState) -> (SceneGraph, CounterState) {
    // TODO: it should be possible to opt out of caching to not clone everything everytime
    // TODO: [`SceneGraph::graph`] should not be pub, node macro should therefore support generics
    let mut scene = (*scene).clone();
    scene.graph.remove_node(input.output_node.clone()).unwrap();
    let value = input.value + 1;
    let output_node = scene
        .graph
        .add_node(ConstantNode(value), "output".to_string())
        .unwrap();
    (scene, CounterState { output_node, value })
}

/// the number of ViewportPlugins that were executed in the pipelines
///
/// For use with [`InitialCounterNode`] and [`IncrementCounterNode`]
pub fn node_count(pipeline: &ViewportPipeline) -> Result<usize, Box<dyn std::error::Error>> {
    let g = pipeline.compute_scene()?.graph;
    let out_port = computegraph::OutputPortUntyped {
        node: computegraph::NodeHandle {
            node_name: "output".to_string(),
        },
        output_name: "output",
    }
    .to_typed();
    Ok(g.compute(out_port)?)
}
