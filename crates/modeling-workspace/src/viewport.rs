use computegraph::{node, ComputeGraph};
use viewport::{ProjectState, RenderNodePorts, SceneGraphBuilder, UpdateNodePorts};

mod camera;
mod rendering;
mod scene_nodes;
mod state;

#[derive(Clone, Debug)]
pub struct ModelingViewportPluginOutput {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelingViewportPlugin {
    pub data_uuid: project::DataId,
}

#[node(ModelingViewportPlugin -> (scene, output))]
fn run(&self, _project: &ProjectState) -> (viewport::SceneGraph, ModelingViewportPluginOutput) {
    let mut graph = ComputeGraph::new();
    let model_node = graph
        .add_node(
            scene_nodes::ModelNode {
                data_uuid: self.data_uuid,
            },
            "model".to_string(),
        )
        .unwrap();
    let meshing_node = graph
        .add_node(scene_nodes::MeshingNode {}, "meshing".to_string())
        .unwrap();
    let render_node = graph
        .add_node(scene_nodes::RenderNode {}, "render".to_string())
        .unwrap();

    graph
        .connect(model_node.output(), meshing_node.input_shape())
        .expect("just created the nodes");
    graph
        .connect(meshing_node.output(), render_node.input_mesh())
        .expect("just created the nodes");

    let update_node = graph
        .add_node(
            scene_nodes::UpdateStateNode {
                data_uuid: self.data_uuid,
            },
            "update".to_string(),
        )
        .unwrap();
    let init_node = graph
        .add_node(scene_nodes::InitStateNode {}, "init".to_string())
        .unwrap();

    (
        SceneGraphBuilder {
            graph,
            initial_state: init_node.output(),
            render_node: RenderNodePorts {
                state_in: render_node.input_state(),
                primitive_out: render_node.output(),
            },
            update_node: UpdateNodePorts {
                event_in: update_node.input_event(),
                state_in: update_node.input_state(),
                state_out: update_node.output(),
            },
        }
        .into(),
        ModelingViewportPluginOutput {},
    )
}
