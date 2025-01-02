mod common;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex, OnceLock},
};

use anyhow::Result;
use common::*;
use computegraph::*;

#[derive(Debug, Clone, PartialEq)]
pub enum OpNodeType {
    Sum,
    A,
    B,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OpNode(&'static str, OpNodeType);

impl OpNode {
    pub fn sum(name: &'static str) -> Self {
        Self(name, OpNodeType::Sum)
    }

    pub fn keep_a(name: &'static str) -> Self {
        Self(name, OpNodeType::A)
    }

    pub fn keep_b(name: &'static str) -> Self {
        Self(name, OpNodeType::B)
    }
}

static OP_LOG: OnceLock<Arc<Mutex<Vec<&'static str>>>> = OnceLock::new();

fn get_op_log() -> Vec<&'static str> {
    let mut log = OP_LOG.get_or_init(Default::default).lock().unwrap();
    std::mem::take(log.as_mut())
}

fn get_op_log_set() -> HashSet<&'static str> {
    get_op_log().into_iter().collect()
}

#[node(OpNode)]
fn run(&self, a: &usize, b: &usize) -> usize {
    let mut log = OP_LOG.get_or_init(Default::default).lock().unwrap();
    log.push(self.0);
    match self.1 {
        OpNodeType::Sum => *a + *b,
        OpNodeType::A => *a,
        OpNodeType::B => *b,
    }
}

#[test]
fn test_caching() -> Result<()> {
    // The graph will look like this:
    //
    // value1──┐
    //         └─►┌─────┐
    //            │ op1 ├──────┐
    //         ┌─►└─────┘      └──────►┌─────┐   ┌─────┐
    // value2──┤                       │ op4 ├──►│ op5 │
    //         └─►┌─────┐ ┌─►┌─────┐ ┌►└─────┘   └──┬──┘
    //            │ op2 ├─┤  │ op3 ├─┘              │
    //         ┌─►└─────┘ └─►└─────┘                │
    // value3──┘                                    ▼
    //                                           result
    let mut graph = ComputeGraph::new();
    let value1 = graph.add_node(TestNodeConstant::new(5), "value1".to_string())?;
    let value2 = graph.add_node(TestNodeConstant::new(7), "value2".to_string())?;
    let value3 = graph.add_node(TestNodeConstant::new(3), "value3".to_string())?;

    let op1 = graph.add_node(OpNode::sum("op1"), "op1".to_string())?;
    let op2 = graph.add_node(OpNode::sum("op2"), "op2".to_string())?;
    let op3 = graph.add_node(OpNode::sum("op3"), "op3".to_string())?;
    let op4 = graph.add_node(OpNode::sum("op4"), "op4".to_string())?;
    let op5 = graph.add_node(OpNode::keep_a("op5"), "op5".to_string())?;

    graph.connect(value1.output(), op1.input_a())?;
    graph.connect(value2.output(), op1.input_b())?;
    graph.connect(value2.output(), op2.input_a())?;
    graph.connect(value3.output(), op2.input_b())?;
    graph.connect(op2.output(), op3.input_a())?;
    graph.connect(op2.output(), op3.input_b())?;
    graph.connect(op1.output(), op4.input_a())?;
    graph.connect(op3.output(), op4.input_b())?;
    graph.connect(op4.output(), op5.input_a())?;
    graph.connect(op4.output(), op5.input_b())?;

    // First run without a cache
    let result_no_cache = graph.compute_with(op5.output(), &ComputationOptions::default(), None)?;

    // Run with an empty cache
    let _ = get_op_log();
    let mut cache = ComputationCache::default();
    let result = graph.compute_with(
        op5.output(),
        &ComputationOptions::default(),
        Some(&mut cache),
    )?;
    assert_eq!(
        get_op_log_set(),
        ["op1", "op2", "op3", "op4", "op5"].into_iter().collect(),
        "All nodes should be run"
    );
    assert_eq!(result_no_cache, result);

    // Rerun the same graph with the same now populated cache
    let result = graph.compute_with(
        op5.output(),
        &ComputationOptions::default(),
        Some(&mut cache),
    )?;
    assert_eq!(
        get_op_log_set(),
        ["op5"].into_iter().collect(),
        "All nodes should have been cached"
    );
    assert_eq!(result_no_cache, result);

    // Let's replace node `op3` to only take `input_a`. This should only recompute `op3` and `op4`
    graph.remove_node(op3)?;
    let op3 = graph.add_node(OpNode::keep_a("op3"), "op3".to_string())?;
    graph.connect(op2.output(), op3.input_a())?;
    graph.connect(op2.output(), op3.input_b())?;
    graph.connect(op3.output(), op4.input_b())?;

    let result_no_cache = graph.compute_with(op5.output(), &ComputationOptions::default(), None)?;
    let _ = get_op_log();
    let result = graph.compute_with(
        op5.output(),
        &ComputationOptions::default(),
        Some(&mut cache),
    )?;
    assert_eq!(
        get_op_log_set(),
        ["op3", "op4", "op5"].into_iter().collect(),
        "Cache should reuse unchanged nodes"
    );
    assert_eq!(result_no_cache, result);

    // Again replace node `op3` to only take `input_b`. This should only recompute `op3`, since the result
    // of `op3` remains the same.
    graph.remove_node(op3)?;
    let op3 = graph.add_node(OpNode::keep_b("op3"), "op3".to_string())?;
    graph.connect(op2.output(), op3.input_a())?;
    graph.connect(op2.output(), op3.input_b())?;
    graph.connect(op3.output(), op4.input_b())?;

    let result_no_cache = graph.compute_with(op4.output(), &ComputationOptions::default(), None)?;
    let _ = get_op_log();
    let result = graph.compute_with(
        op5.output(),
        &ComputationOptions::default(),
        Some(&mut cache),
    )?;
    assert_eq!(
        get_op_log_set(),
        ["op3", "op5"].into_iter().collect(),
        "Cache should reuse computations if the result of dependencies does not change"
    );
    assert_eq!(result_no_cache, result);

    Ok(())
}
