mod common;

use anyhow::Result;
use common::*;
use computegraph::*;

#[test]
fn test_connect_already_connected() -> Result<()> {
    let mut graph = ComputeGraph::new();

    let value1 = graph.add_node(TestNodeConstant::new(5), "value1".to_string())?;
    let value2 = graph.add_node(TestNodeConstant::new(7), "value2".to_string())?;
    let to_string = graph.add_node(TestNodeNumToString::new(), "to_string".to_string())?;

    graph.connect(value1.output(), to_string.input())?;
    let res = graph.connect(value2.output(), to_string.input());
    match res {
        Err(ConnectError::InputPortAlreadyConnected { from, to }) => {
            assert_eq!(from.node, value2.handle);
            assert_eq!(to.node, to_string.handle);
        }
        _ => panic!("Expected ConnectError::InputPortAlreadyConnected"),
    }

    Ok(())
}

#[test]
fn test_duplicate_node_names() -> Result<()> {
    let mut graph = ComputeGraph::new();

    graph.add_node(TestNodeConstant::new(5), "value".to_string())?;
    match graph.add_node(TestNodeConstant::new(7), "value".to_string()) {
        Err(AddError::DuplicateName(name)) => {
            assert_eq!(name, "value");
        }
        _ => panic!("Expected AddError::DuplicateName"),
    }

    Ok(())
}
