mod common;

use anyhow::Result;
use common::*;
use computegraph::*;

#[test]
fn test_edge_disconnection() -> Result<()> {
    let mut graph = ComputeGraph::new();
    let value = graph.add_node(TestNodeConstant::new(5), "value".to_string())?;
    let one = graph.add_node(TestNodeConstant::new(1), "one".to_string())?;
    let addition = graph.add_node(TestNodeAddition::new(), "addition".to_string())?;
    let to_string = graph.add_node(TestNodeNumToString::new(), "to_string".to_string())?;

    let value_to_addition = graph.connect(value.output(), addition.input_a())?;
    graph.connect(one.output(), addition.input_b())?;
    graph.connect(addition.output(), to_string.input())?;

    // Test that the graph works before disconnecting the edge
    assert_eq!(graph.compute(to_string.output())?, "6".to_string());

    // Disconnect the edge between value and addition nodes
    graph.disconnect(&value_to_addition)?;

    // Test that the graph fails after disconnecting the edge with the expected error
    match graph.compute(to_string.output()) {
        Err(ComputeError::InputPortNotConnected(port)) => {
            assert_eq!(port.node, addition.handle);
            assert_eq!(port.input_name, "a");
        }
        _ => panic!("Expected ComputeError::InputPortNotConnected"),
    }

    // Now reconnect the edge and test that the graph works again
    graph.connect(value.output(), addition.input_a())?;
    assert_eq!(graph.compute(to_string.output())?, "6".to_string());

    Ok(())
}

#[test]
fn test_node_removal() -> Result<()> {
    let mut graph = ComputeGraph::new();
    let value1 = graph.add_node(TestNodeConstant::new(5), "value1".to_string())?;
    let value2 = graph.add_node(TestNodeConstant::new(7), "value2".to_string())?;
    let addition = graph.add_node(TestNodeAddition::new(), "addition".to_string())?;

    graph.connect(value1.output(), addition.input_a())?;
    graph.connect(value2.output(), addition.input_b())?;

    // Compute the result before removing a node
    assert_eq!(graph.compute(addition.output())?, 12);

    // Remove the 'value2' node from the graph
    graph.remove_node(value2.handle)?;

    // After removing 'value2', the 'addition' node should have a missing input
    match graph.compute(addition.output()) {
        Err(ComputeError::InputPortNotConnected(port)) => {
            assert_eq!(port.node, addition.handle);
            assert_eq!(port.input_name, "b");
        }
        _ => panic!("Expected ComputeError::InputPortNotConnected"),
    }

    // Ensure that the 'value1' node can still be computed
    assert_eq!(graph.compute(value1.output())?, 5);

    // Now connect value1 to both inputs of the addition node
    graph.connect(value1.output(), addition.input_b())?;

    // Compute the result after reconnecting the edge
    assert_eq!(graph.compute(addition.output())?, 10);

    Ok(())
}
