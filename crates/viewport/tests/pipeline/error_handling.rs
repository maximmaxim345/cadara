use crate::common::*;
use computegraph::node;
use viewport::{
    PipelineAddError, SceneGraph, ViewportPipeline, ViewportPlugin, ViewportPluginValidationError,
};

#[test]
fn test_invalid_node_errors() {
    // Test a node without output port
    #[derive(Debug, Clone, PartialEq)]
    struct InvalidNode1;
    #[node(InvalidNode1)]
    fn run(&self) {}

    let result = ViewportPlugin::new(InvalidNode1 {});
    assert!(matches!(
        result,
        Err(ViewportPluginValidationError::InvalidSceneOutputPort)
    ));

    // Test adding a node with invalid graph output type
    #[derive(Debug, Clone, PartialEq)]
    struct InvalidNode2;
    #[node(InvalidNode2 -> graph)]
    fn run(&self) -> String {
        "Invalid graph output".to_string()
    }

    let result = ViewportPlugin::new(InvalidNode2 {});
    assert!(matches!(
        result,
        Err(ViewportPluginValidationError::InvalidSceneOutputPort)
    ));
}

#[test]
fn test_incompatible_input_ports() {
    #[derive(Debug, Clone, PartialEq)]
    struct IncompatibleNode;
    #[node(IncompatibleNode -> (scene, output))]
    fn run(&self, _input: &usize) -> (SceneGraph, usize) {
        unimplemented!()
    }

    let result = ViewportPlugin::new(IncompatibleNode {});
    assert!(matches!(
        result,
        Err(ViewportPluginValidationError::InputPortMismatch)
    ));
}

#[test]
fn test_type_mismatch_error() {
    let mut graph = ViewportPipeline::default();
    graph
        .add_plugin(ViewportPlugin::new(InitialCounterNode {}).unwrap())
        .unwrap();

    // Try to add a node with mismatched input type
    #[derive(Debug, Clone, PartialEq)]
    struct MismatchedNode;
    #[node(MismatchedNode -> (scene, output))]
    fn run(
        &self,
        _scene: &SceneGraph,
        _input: &String,
    ) -> (SceneGraph, (ConstantNodeHandle, usize)) {
        unimplemented!()
    }

    let result = graph.add_plugin(ViewportPlugin::new(MismatchedNode).unwrap());
    assert!(matches!(result, Err(PipelineAddError::TypeMismatch { .. })));
}

#[test]
fn test_dependent_node_in_empty_graph() {
    let mut graph = ViewportPipeline::default();

    // Try to add Node2 to an empty graph
    let result = graph.add_plugin(ViewportPlugin::new(IncrementCounterNode {}).unwrap());
    assert!(matches!(
        result,
        Err(PipelineAddError::SubsequentPluginInEmptyPipeline)
    ));
}
