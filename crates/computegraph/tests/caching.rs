mod common;
use std::{cell::RefCell, collections::HashSet};

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

thread_local! {
    static OP_LOG: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
}

fn get_op_log() -> Vec<&'static str> {
    OP_LOG.with(|log| std::mem::take(&mut *log.borrow_mut()))
}

fn get_op_log_set() -> HashSet<&'static str> {
    get_op_log().into_iter().collect()
}

#[node(OpNode)]
fn run(&self, a: &usize, b: &usize) -> usize {
    OP_LOG.with(|log| {
        log.borrow_mut().push(self.0);
    });
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

#[test]
fn test_discard_old_cache_results() -> Result<()> {
    // The graph will look like this:
    //
    //           ┌──────┐   ┌──────┐
    //        ┌─►│ op1a ├──►│ op1b ├──►
    //        │  └──────┘   └──────┘
    // value──┤
    //        │  ┌──────┐   ┌──────┐
    //        └─►│ op2a ├──►│ op2b ├──►
    //           └──────┘   └──────┘
    let mut graph = ComputeGraph::new();
    let value = graph.add_node(TestNodeConstant::new(11), "value".to_string())?;

    let op1a = graph.add_node(OpNode::sum("op1a"), "op1a".to_string())?;
    let op1b = graph.add_node(OpNode::sum("op1b"), "op1b".to_string())?;
    let op2a = graph.add_node(OpNode::sum("op2a"), "op2a".to_string())?;
    let op2b = graph.add_node(OpNode::sum("op2b"), "op2b".to_string())?;

    graph.connect(value.output(), op1a.input_a())?;
    graph.connect(value.output(), op1a.input_b())?;
    graph.connect(value.output(), op2a.input_a())?;
    graph.connect(value.output(), op2a.input_b())?;

    graph.connect(op1a.output(), op1b.input_a())?;
    graph.connect(op1a.output(), op1b.input_b())?;
    graph.connect(op2a.output(), op2b.input_a())?;
    graph.connect(op2a.output(), op2b.input_b())?;

    let mut graph2 = ComputeGraph::new();
    let graph2_value = graph2.add_node(TestNodeConstant::new(11), "value".to_string())?;
    let graph2_op = graph2.add_node(OpNode::sum("op"), "op".to_string())?;
    graph2.connect(graph2_value.output(), graph2_op.input_a())?;
    graph2.connect(graph2_value.output(), graph2_op.input_b())?;

    let mut cache = ComputationCache::new();

    graph.compute_with(
        op1b.output(),
        &ComputationOptions::default(),
        Some(&mut cache),
    )?;
    assert_eq!(get_op_log_set(), ["op1a", "op1b"].into_iter().collect(),);

    graph.compute_with(
        op1b.output(),
        &ComputationOptions::default(),
        Some(&mut cache),
    )?;
    assert_eq!(
        get_op_log_set(),
        ["op1b"].into_iter().collect(),
        "op1a should have been cached"
    );

    // Use the same cache on a mostly empty graph.
    // This should discard some entries from the cache.
    graph2.compute_with(
        graph2_op.output(),
        &ComputationOptions::default(),
        Some(&mut cache),
    )?;
    assert_eq!(get_op_log_set(), ["op"].into_iter().collect(),);

    graph.compute_with(
        op1b.output(),
        &ComputationOptions::default(),
        Some(&mut cache),
    )?;
    assert_eq!(
        get_op_log_set(),
        ["op1a", "op1b"].into_iter().collect(),
        "op1a should have been discarded from the cache, due to previously running on a graph without that node"
    );

    Ok(())
}

#[test]
fn test_dont_discard_nodes_still_in_the_graph() -> Result<()> {
    // The graph will look like this:
    //
    //           ┌──────┐   ┌──────┐
    //        ┌─►│ op1a ├──►│ op1b ├──►
    //        │  └──────┘   └──────┘
    // value──┤
    //        │  ┌──────┐   ┌──────┐
    //        └─►│ op2a ├──►│ op2b ├──►
    //           └──────┘   └──────┘
    let mut graph = ComputeGraph::new();
    let value = graph.add_node(TestNodeConstant::new(11), "value".to_string())?;

    let op1a = graph.add_node(OpNode::sum("op1a"), "op1a".to_string())?;
    let op1b = graph.add_node(OpNode::sum("op1b"), "op1b".to_string())?;
    let op2a = graph.add_node(OpNode::sum("op2a"), "op2a".to_string())?;
    let op2b = graph.add_node(OpNode::sum("op2b"), "op2b".to_string())?;

    graph.connect(value.output(), op1a.input_a())?;
    graph.connect(value.output(), op1a.input_b())?;
    graph.connect(value.output(), op2a.input_a())?;
    graph.connect(value.output(), op2a.input_b())?;

    graph.connect(op1a.output(), op1b.input_a())?;
    graph.connect(op1a.output(), op1b.input_b())?;
    graph.connect(op2a.output(), op2b.input_a())?;
    graph.connect(op2a.output(), op2b.input_b())?;

    let mut cache = ComputationCache::new();

    graph.compute_with(
        op1b.output(),
        &ComputationOptions::default(),
        Some(&mut cache),
    )?;
    assert_eq!(get_op_log_set(), ["op1a", "op1b"].into_iter().collect(),);

    graph.compute_with(
        op2b.output(),
        &ComputationOptions::default(),
        Some(&mut cache),
    )?;
    assert_eq!(get_op_log_set(), ["op2a", "op2b"].into_iter().collect(),);

    graph.compute_with(
        op1b.output(),
        &ComputationOptions::default(),
        Some(&mut cache),
    )?;
    assert_eq!(
        get_op_log_set(),
        ["op1b"].into_iter().collect(),
        "op1a should still be kept in cache"
    );

    Ok(())
}

#[test]
fn test_caching_with_override() -> Result<()> {
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

    let mut cache = ComputationCache::new();

    let mut context = ComputationContext::new();
    context.set_override(op3.input_a(), 10);
    context.set_override(op3.input_b(), 10);
    context.set_override(op4.input_a(), 5);

    assert_eq!(
        graph.compute_with(
            op5.output(),
            &ComputationOptions {
                context: Some(&context),
            },
            Some(&mut cache),
        )?,
        25
    );
    assert_eq!(
        get_op_log_set(),
        ["op3", "op4", "op5"].into_iter().collect(),
    );

    assert_eq!(
        graph.compute_with(
            op5.output(),
            &ComputationOptions {
                context: Some(&context),
            },
            Some(&mut cache),
        )?,
        25
    );
    assert_eq!(
        get_op_log_set(),
        ["op3", "op4", "op5"].into_iter().collect(),
        "overrides should trigger a recompute"
    );

    // Change the override
    context.set_override(op3.input_a(), 20);
    context.set_override(op3.input_b(), 20);
    assert_eq!(
        graph.compute_with(
            op5.output(),
            &ComputationOptions {
                context: Some(&context),
            },
            Some(&mut cache),
        )?,
        45
    );
    assert_eq!(
        get_op_log_set(),
        ["op3", "op4", "op5"].into_iter().collect(),
        "we changed the context, a recompute is required"
    );

    Ok(())
}

#[test]
fn test_caching_with_fallback() -> Result<()> {
    // The graph will look like this ('►' are unconnected):
    //
    //           ►┌─────┐   ┌─────┐
    //            │ op4 ├──►│ op5 │
    // ►┌─────┐ ┌►└─────┘   └──┬──┘
    //  │ op3 ├─┘              │
    // ►└─────┘                │
    //                         ▼
    //                      result
    let mut graph = ComputeGraph::new();

    let op3 = graph.add_node(OpNode::sum("op3"), "op3".to_string())?;
    let op4 = graph.add_node(OpNode::sum("op4"), "op4".to_string())?;
    let op5 = graph.add_node(OpNode::keep_a("op5"), "op5".to_string())?;

    graph.connect(op3.output(), op4.input_b())?;
    graph.connect(op4.output(), op5.input_a())?;
    graph.connect(op4.output(), op5.input_b())?;

    let mut cache = ComputationCache::new();

    let mut context = ComputationContext::new();
    context.set_fallback(10_usize);

    assert_eq!(
        graph.compute_with(
            op5.output(),
            &ComputationOptions {
                context: Some(&context),
            },
            Some(&mut cache),
        )?,
        30
    );
    assert_eq!(
        get_op_log_set(),
        ["op3", "op4", "op5"].into_iter().collect(),
    );

    assert_eq!(
        graph.compute_with(
            op5.output(),
            &ComputationOptions {
                context: Some(&context),
            },
            Some(&mut cache),
        )?,
        30
    );
    assert_eq!(
        get_op_log_set(),
        ["op3", "op4", "op5"].into_iter().collect(),
        "fallbacks should trigger a recompute"
    );

    // Change the fallback
    context.set_fallback(20_usize);
    assert_eq!(
        graph.compute_with(
            op5.output(),
            &ComputationOptions {
                context: Some(&context),
            },
            Some(&mut cache),
        )?,
        60
    );
    assert_eq!(
        get_op_log_set(),
        ["op3", "op4", "op5"].into_iter().collect(),
        "we changed the context, a recompute is required"
    );

    Ok(())
}

#[test]
fn test_caching_with_cacheable_fallback() -> Result<()> {
    // The graph will look like this ('►' are unconnected):
    //
    //           ►┌─────┐   ┌─────┐
    //            │ op4 ├──►│ op5 │
    // ►┌─────┐ ┌►└─────┘   └──┬──┘
    //  │ op3 ├─┘              │
    // ►└─────┘                │
    //                         ▼
    //                      result
    let mut graph = ComputeGraph::new();

    let op3 = graph.add_node(OpNode::sum("op3"), "op3".to_string())?;
    let op4 = graph.add_node(OpNode::sum("op4"), "op4".to_string())?;
    let op5 = graph.add_node(OpNode::keep_a("op5"), "op5".to_string())?;

    graph.connect(op3.output(), op4.input_b())?;
    graph.connect(op4.output(), op5.input_a())?;
    graph.connect(op4.output(), op5.input_b())?;

    let mut cache = ComputationCache::new();

    let mut context = ComputationContext::new();
    context.set_fallback_cached(10_usize);

    assert_eq!(
        graph.compute_with(
            op5.output(),
            &ComputationOptions {
                context: Some(&context),
            },
            Some(&mut cache),
        )?,
        30
    );
    assert_eq!(
        get_op_log_set(),
        ["op3", "op4", "op5"].into_iter().collect(),
    );

    assert_eq!(
        graph.compute_with(
            op5.output(),
            &ComputationOptions {
                context: Some(&context),
            },
            Some(&mut cache),
        )?,
        30
    );
    assert_eq!(
        get_op_log_set(),
        ["op5"].into_iter().collect(),
        "fallbacks should be cached"
    );

    // Change the fallback
    context.set_fallback_cached(20_usize);
    assert_eq!(
        graph.compute_with(
            op5.output(),
            &ComputationOptions {
                context: Some(&context),
            },
            Some(&mut cache),
        )?,
        60
    );
    assert_eq!(
        get_op_log_set(),
        ["op3", "op4", "op5"].into_iter().collect(),
        "we changed the context, a recompute is required"
    );

    context.set_fallback_cached(20_usize);
    assert_eq!(
        graph.compute_with(
            op5.output(),
            &ComputationOptions {
                context: Some(&context),
            },
            Some(&mut cache),
        )?,
        60
    );
    assert_eq!(
        get_op_log_set(),
        ["op5"].into_iter().collect(),
        "fallbacks should be cached again"
    );

    // Check if the cache is discarded if the fallback is removed (and kept otherwise):
    // Add a new node to not disturb the cache of the other nodes
    let value = graph.add_node(TestNodeConstant::new(10), "value".to_string())?;

    assert_eq!(
        graph.compute_with(
            value.output(),
            &ComputationOptions {
                context: Some(&context),
            },
            Some(&mut cache),
        )?,
        10
    );
    assert_eq!(get_op_log_set(), [].into_iter().collect(),);

    assert_eq!(
        graph.compute_with(
            op5.output(),
            &ComputationOptions {
                context: Some(&context),
            },
            Some(&mut cache),
        )?,
        60
    );
    assert_eq!(
        get_op_log_set(),
        ["op5"].into_iter().collect(),
        "fallbacks should still be in the cache, even though the fallback was not used"
    );

    // now remove the fallback

    context.remove_fallback::<usize>();
    assert_eq!(
        graph.compute_with(
            value.output(),
            &ComputationOptions {
                context: Some(&context),
            },
            Some(&mut cache),
        )?,
        10
    );
    assert_eq!(get_op_log_set(), [].into_iter().collect(),);

    // And readd it (with the same value)
    context.set_fallback_cached(20_usize);

    assert_eq!(
        graph.compute_with(
            op5.output(),
            &ComputationOptions {
                context: Some(&context),
            },
            Some(&mut cache),
        )?,
        60
    );
    assert_eq!(
        get_op_log_set(),
        ["op3", "op5"].into_iter().collect(),
        "fallbacks should be discarded from the cache, but `op4` should still be detected as unchanged"
    );

    Ok(())
}
