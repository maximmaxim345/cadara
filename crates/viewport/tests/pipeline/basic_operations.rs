use crate::common::*;
use viewport::{ExecuteError, ViewportPipeline, ViewportPlugin};

#[test]
fn test_viewport_plugins() {
    let mut pipeline = ViewportPipeline::default();
    assert_eq!(pipeline.len(), 0);

    let initial_node = ViewportPlugin::new(InitialCounterNode {}).unwrap();
    let subsequent_node = ViewportPlugin::new(IncrementCounterNode {}).unwrap();

    pipeline.add_plugin(initial_node.clone()).unwrap();
    assert_eq!(pipeline.len(), 1);
    assert_eq!(node_count(&pipeline).unwrap(), 1);

    pipeline.add_plugin(initial_node.clone()).unwrap();
    assert_eq!(pipeline.len(), 2);
    assert_eq!(node_count(&pipeline).unwrap(), 1);

    pipeline.add_plugin(subsequent_node.clone()).unwrap();
    assert_eq!(pipeline.len(), 3);
    assert_eq!(node_count(&pipeline).unwrap(), 2);

    pipeline.add_plugin(initial_node.clone()).unwrap();
    assert_eq!(pipeline.len(), 4);
    assert_eq!(node_count(&pipeline).unwrap(), 1);

    pipeline.add_plugin(subsequent_node.clone()).unwrap();
    assert_eq!(pipeline.len(), 5);
    assert_eq!(node_count(&pipeline).unwrap(), 2);

    pipeline.add_plugin(subsequent_node.clone()).unwrap();
    assert_eq!(pipeline.len(), 6);
    assert_eq!(node_count(&pipeline).unwrap(), 3);

    // Now remove all nodes step by step

    pipeline.remove_last_plugin();
    assert_eq!(pipeline.len(), 5);
    assert_eq!(node_count(&pipeline).unwrap(), 2);

    pipeline.remove_last_plugin();
    assert_eq!(pipeline.len(), 4);
    assert_eq!(node_count(&pipeline).unwrap(), 1);

    pipeline.remove_last_plugin();
    assert_eq!(pipeline.len(), 3);
    assert_eq!(node_count(&pipeline).unwrap(), 2);

    pipeline.remove_last_plugin();
    assert_eq!(pipeline.len(), 2);
    assert_eq!(node_count(&pipeline).unwrap(), 1);

    pipeline.remove_last_plugin();
    assert_eq!(pipeline.len(), 1);
    assert_eq!(node_count(&pipeline).unwrap(), 1);

    pipeline.remove_last_plugin();
    assert_eq!(pipeline.len(), 0);

    // Try to readd a node, checks if the node was really deleted from the internal graph
    pipeline.add_plugin(initial_node.clone()).unwrap();
    assert_eq!(pipeline.len(), 1);
}

#[test]
fn test_remove_node_from_empty_pipeline() {
    let mut pipeline = ViewportPipeline::default();
    pipeline.remove_last_plugin();
    assert_eq!(pipeline.len(), 0);
}

#[test]
fn test_compute_single_node_pipeline() {
    let mut pipeline = ViewportPipeline::default();
    pipeline
        .add_plugin(ViewportPlugin::new(InitialCounterNode {}).unwrap())
        .unwrap();
    let result = node_count(&pipeline);
    assert_eq!(result.unwrap(), 1);
}

#[test]
fn test_compute_empty_pipeline() {
    let pipeline = ViewportPipeline::default();
    let project = project::Project::new("project".to_string());
    let result = pipeline.compute_scene(project.create_session());
    assert!(matches!(result, Err(ExecuteError::EmptyPipeline)));
}
