use std::sync::Arc;

use viewport::{ExecuteError, ViewportCache, ViewportPipeline, ViewportPlugin};

use crate::common::ProjectAwarePlugin;

fn empty_view() -> Arc<project::ProjectView> {
    let project = project::Project::new();
    Arc::new(
        project
            .create_view(&project::ModuleRegistry::default())
            .unwrap(),
    )
}

/// Removing a plugin while the pipeline stays non-empty evicts the dead node's
/// metadata instead of accumulating it on top of the survivor's.
#[test]
fn evicts_removed_node_while_nonempty() {
    let mut pipeline = ViewportPipeline::default();
    let mut cache = ViewportCache::default();
    let view = empty_view();

    pipeline
        .add_plugin(ViewportPlugin::new(ProjectAwarePlugin).unwrap())
        .unwrap();
    pipeline
        .add_plugin(ViewportPlugin::new(ProjectAwarePlugin).unwrap())
        .unwrap();

    // Only the last plugin's node executes, so exactly one node is tracked.
    pipeline.compute_scene(view.clone(), 1, &mut cache).unwrap();
    assert_eq!(cache.stats().metadata_nodes.len(), 1);

    // A leak would add the new last node without dropping the removed one,
    // pushing the count to 2.
    pipeline.remove_last_plugin();
    pipeline.compute_scene(view, 2, &mut cache).unwrap();
    assert_eq!(cache.stats().metadata_nodes.len(), 1);
}

/// Emptying the pipeline drops both caches rather than retaining the last
/// non-empty state forever.
#[test]
fn clears_caches_when_pipeline_empty() {
    let mut pipeline = ViewportPipeline::default();
    let mut cache = ViewportCache::default();
    let view = empty_view();

    pipeline
        .add_plugin(ViewportPlugin::new(ProjectAwarePlugin).unwrap())
        .unwrap();
    pipeline.compute_scene(view.clone(), 1, &mut cache).unwrap();
    let stats = cache.stats();
    assert!(!stats.metadata_nodes.is_empty());
    assert!(stats.computation_entries > 0);

    pipeline.remove_last_plugin();
    assert!(matches!(
        pipeline.compute_scene(view, 2, &mut cache),
        Err(ExecuteError::EmptyPipeline)
    ));

    let stats = cache.stats();
    assert!(stats.metadata_nodes.is_empty());
    assert_eq!(stats.computation_entries, 0);
}
