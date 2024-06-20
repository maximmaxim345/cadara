mod common;
use std::any::TypeId;

use anyhow::Result;
use common::*;
use computegraph::*;

#[test]
fn test_basic_graph() -> Result<()> {
    let mut graph = ComputeGraph::new();
    let value1 = graph.add_node(TestNodeConstant::new(9), "value1".to_string())?;
    let value2 = graph.add_node(TestNodeConstant::new(10), "value2".to_string())?;

    let addition = graph.add_node(TestNodeAddition::new(), "addition".to_string())?;

    graph.connect(value1.output(), addition.input_a())?;
    graph.connect(value2.output(), addition.input_b())?;

    assert_eq!(graph.compute(value1.output())?, 9);
    assert_eq!(graph.compute(value2.output())?, 10);
    assert_eq!(graph.compute(addition.output())?, 19);

    Ok(())
}

#[test]
fn test_diamond_dependencies_and_cloning() -> Result<()> {
    // Here we will test a more complex graph with two diamond dependencies between nodes.
    // The graph will look like this:
    //
    // value1──┐
    //         └─►┌─────────┐
    //            │addition1├────────────┐
    //         ┌─►└─────────┘            └────►┌─────────┐
    // value2──┤                               │addition4│
    //         └─►┌─────────┐ ┌─►┌─────────┐ ┌►└────┬────┘
    //            │addition2├─┤  │addition3├─┘      │
    //         ┌─►└─────────┘ └─►└─────────┘        │
    // value3──┘                                    ▼
    //                                            result
    //
    // So The result should be:
    let function = |v1: usize, v2: usize, v3: usize| v1 + v2 + 2 * (v2 + v3);
    let correct_result = function(5, 7, 3);

    let mut graph = ComputeGraph::new();
    let value1 = graph.add_node(TestNodeConstant::new(5), "value1".to_string())?;
    let value2 = graph.add_node(TestNodeConstant::new(7), "value2".to_string())?;
    let value3 = graph.add_node(TestNodeConstant::new(3), "value3".to_string())?;

    let addition1 = graph.add_node(TestNodeAddition::new(), "addition1".to_string())?;
    let addition2 = graph.add_node(TestNodeAddition::new(), "addition2".to_string())?;
    let addition3 = graph.add_node(TestNodeAddition::new(), "addition3".to_string())?;
    let addition4 = graph.add_node(TestNodeAddition::new(), "addition4".to_string())?;

    graph.connect(value1.output(), addition1.input_a())?;
    graph.connect(value2.output(), addition1.input_b())?;
    graph.connect(value2.output(), addition2.input_a())?;
    graph.connect(value3.output(), addition2.input_b())?;
    graph.connect(addition2.output(), addition3.input_a())?;
    graph.connect(addition2.output(), addition3.input_b())?;
    graph.connect(addition1.output(), addition4.input_a())?;
    graph.connect(addition3.output(), addition4.input_b())?;

    assert_eq!(graph.compute(addition4.output())?, correct_result);
    // Cloning should not affect the result
    assert_eq!(graph.clone().compute(addition4.output())?, correct_result);

    Ok(())
}

#[test]
fn test_invalid_graph_missing_input() -> Result<()> {
    let mut graph = ComputeGraph::new();
    let value = graph.add_node(TestNodeConstant::new(5), "value".to_string())?;
    let addition = graph.add_node(TestNodeAddition::new(), "addition".to_string())?;

    graph.connect(value.output(), addition.input_a())?;

    match graph.compute(addition.output()) {
        Err(ComputeError::InputPortNotConnected(err)) => {
            assert_eq!(err.node, addition.handle);
            assert_eq!(err.input_name, "b");
        }
        _ => panic!("Expected ComputeError::InputPortNotConnected"),
    }

    Ok(())
}

#[test]
fn test_invalid_graph_type_mismatch() -> Result<()> {
    let mut graph = ComputeGraph::new();
    let value = graph.add_node(TestNodeConstant::new(5), "value".to_string())?;
    let to_string = graph.add_node(TestNodeNumToString::new(), "to_string".to_string())?;
    let addition = graph.add_node(TestNodeAddition::new(), "addition".to_string())?;

    graph.connect(value.output(), to_string.input())?;
    graph.connect(value.output(), addition.input_a())?;
    let res = graph.connect_untyped(to_string.output().into(), addition.input_b().into());
    match res {
        Err(ConnectError::TypeMismatch { expected, found }) => {
            assert_eq!(expected, TypeId::of::<usize>());
            assert_eq!(found, TypeId::of::<String>());
        }
        _ => panic!("Expected ConnectError::TypeMismatch"),
    }

    Ok(())
}

#[test]
fn test_cycle_detection() -> Result<()> {
    let mut graph = ComputeGraph::new();
    let value = graph.add_node(TestNodeConstant::new(5), "value".to_string())?;
    let node1 = graph.add_node(TestNodeAddition::new(), "node1".to_string())?;
    let node2 = graph.add_node(TestNodeAddition::new(), "node2".to_string())?;
    let node3 = graph.add_node(TestNodeAddition::new(), "node3".to_string())?;

    graph.connect(node1.output(), node2.input_a())?;
    graph.connect(node2.output(), node3.input_a())?;
    graph.connect(node3.output(), node1.input_a())?;

    graph.connect(value.output(), node2.input_b())?;
    graph.connect(value.output(), node3.input_b())?;
    graph.connect(value.output(), node1.input_b())?;

    // The graph contains a cycle: node1 -> node2 -> node3 -> node1 -> ...
    let result = graph.compute(node1.output());
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_disconnected_subgraphs() -> Result<()> {
    let mut graph = ComputeGraph::new();

    // Subgraph 1: Addition
    let value1 = graph.add_node(TestNodeConstant::new(5), "value1".to_string())?;
    let value2 = graph.add_node(TestNodeConstant::new(7), "value2".to_string())?;
    let addition1 = graph.add_node(TestNodeAddition::new(), "addition1".to_string())?;
    graph.connect(value1.output(), addition1.input_a())?;
    graph.connect(value2.output(), addition1.input_b())?;

    // Subgraph 2: Addition
    let value3 = graph.add_node(TestNodeConstant::new(3), "value3".to_string())?;
    let value4 = graph.add_node(TestNodeConstant::new(4), "value4".to_string())?;
    let addition2 = graph.add_node(TestNodeAddition::new(), "addition2".to_string())?;
    graph.connect(value3.output(), addition2.input_a())?;
    graph.connect(value4.output(), addition2.input_b())?;

    // Compute the results of the disconnected subgraphs independently
    assert_eq!(graph.compute(addition1.output())?, 12);

    assert_eq!(graph.compute(addition2.output())?, 7);

    Ok(())
}
