use crate::common::*;
use computegraph::node;
use viewport::{DynamicViewportPlugin, ViewportPipeline, ViewportPluginValidationError};

#[test]
fn test_add_plugin_dynamic() {
    let mut graph = ViewportPipeline::default();

    // Test adding a valid dynamic node
    let dynamic_node = DynamicViewportPlugin::new(InitialCounterNode {}.into()).unwrap();
    assert!(graph.add_dynamic_plugin(dynamic_node).is_ok());
    assert_eq!(graph.len(), 1);

    // Test adding another valid dynamic node
    let dynamic_node2 = DynamicViewportPlugin::new(IncrementCounterNode {}.into()).unwrap();
    assert!(graph.add_dynamic_plugin(dynamic_node2).is_ok());
    assert_eq!(graph.len(), 2);

    // Test adding an invalid dynamic node
    #[derive(Clone, Debug, PartialEq)]
    struct InvalidNode;
    #[node(InvalidNode)]
    fn run(&self) {}

    let result = DynamicViewportPlugin::new(InvalidNode {}.into());
    assert!(matches!(
        result,
        Err(ViewportPluginValidationError::InvalidSceneOutputPort)
    ));
}
