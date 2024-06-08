mod common;

use anyhow::{anyhow, Result};
use common::*;
use computegraph::*;

#[test]
fn test_metadata() -> Result<()> {
    #[derive(Debug, PartialEq)]
    struct SomeMetadata;
    #[derive(Debug, PartialEq)]
    struct OtherMetadata(usize);

    let mut graph = ComputeGraph::new();
    let value = graph.add_node(TestNodeConstant::new(5), "value".to_string())?;
    let value_node = graph
        .get_node_mut(&value.handle)
        .ok_or_else(|| anyhow!("value node not found"))?;

    assert_eq!(value_node.metadata.get::<SomeMetadata>(), None);
    value_node.metadata.insert(SomeMetadata);
    assert_eq!(
        value_node.metadata.get::<SomeMetadata>(),
        Some(&SomeMetadata)
    );
    value_node.metadata.remove::<SomeMetadata>();
    value_node.metadata.insert(OtherMetadata(42));

    let value_node = graph
        .get_node(&value.handle)
        .ok_or_else(|| anyhow!("value node not found"))?;
    assert_eq!(value_node.metadata.get::<SomeMetadata>(), None);
    assert_eq!(value_node.metadata.get(), Some(&OtherMetadata(42)));
    Ok(())
}
