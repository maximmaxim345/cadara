mod common;

use anyhow::Result;
use common::*;
use computegraph::*;
use std::any::TypeId;

#[test]
fn test_context_override() -> Result<()> {
    let mut graph = ComputeGraph::new();
    let addition = graph.add_node(TestNodeAddition::new(), "addition".to_string())?;

    let mut ctx = ComputationContext::new();
    ctx.set_override(addition.input_a(), 1);
    ctx.set_override(addition.input_b(), 2);

    ctx.set_override(addition.input_b(), 3);
    ctx.set_override(addition.input_a(), 5);

    assert_eq!(
        graph.compute_with_context(addition.output(), &ctx)?,
        8,
        "ctx should use the latest given value"
    );
    assert_eq!(
        *graph
            .compute_untyped_with_context(addition.output().into(), &ctx)?
            .as_ref()
            .as_any()
            .downcast_ref::<usize>()
            .unwrap(),
        8,
    );

    Ok(())
}

#[test]
fn test_context_override_skip_dependencies() -> Result<()> {
    let mut graph = ComputeGraph::new();
    let invalid_dep = graph.add_node(TestNodeAddition::new(), "invalid_addition".to_string())?;
    let value = graph.add_node(TestNodeConstant::new(10), "value".to_string())?;
    let addition = graph.add_node(TestNodeAddition::new(), "addition".to_string())?;

    graph.connect(invalid_dep.output(), addition.input_a())?;
    graph.connect(value.output(), addition.input_b())?;

    assert!(matches!(
        graph.compute(addition.output()),
        Err(ComputeError::InputPortNotConnected(_))
    ));

    let mut ctx = ComputationContext::new();
    ctx.set_override(addition.input_a(), 5);

    assert_eq!(
        graph
            .compute_with_context(addition.output(), &ctx)
            .expect("This should skip 'invalid_dep' entirely"),
        15
    );

    let input_port_with_wrong_type: InputPort<String> =
        InputPortUntyped::from(addition.input_a()).to_typed();
    assert_eq!(ctx.remove_override(&input_port_with_wrong_type), None);

    assert_eq!(
        ctx.remove_override(&addition.input_a()),
        Some(5),
        "remove_override should keep the value if the type is incorrect"
    );
    assert_eq!(ctx.remove_override(&addition.input_a()), None);

    Ok(())
}

#[test]
fn test_context_fallback() -> Result<()> {
    let mut graph = ComputeGraph::new();

    let addition = graph.add_node(TestNodeAddition::new(), "addition".to_string())?;

    let mut ctx = ComputationContext::new();
    ctx.set_fallback(5_usize);
    ctx.set_fallback(10_usize);

    assert_eq!(graph.compute_with_context(addition.output(), &ctx)?, 20);
    assert_eq!(
        *graph
            .compute_untyped_with_context(addition.output().into(), &ctx)?
            .as_ref()
            .as_any()
            .downcast_ref::<usize>()
            .unwrap(),
        20
    );

    assert_eq!(ctx.remove_fallback::<usize>(), Some(10));
    assert_eq!(ctx.remove_fallback::<usize>(), None);

    Ok(())
}

#[test]
fn test_context_priority() -> Result<()> {
    let mut graph = ComputeGraph::new();

    let zero = graph.add_node(TestNodeConstant::new(0), "zero".to_string())?;
    let value = graph.add_node(TestNodeConstant::new(5), "value".to_string())?;
    let addition = graph.add_node(TestNodeAddition::new(), "addition".to_string())?;

    graph.connect(zero.output(), addition.input_a())?;
    graph.connect(value.output(), addition.input_b())?;

    let mut ctx = ComputationContext::new();
    ctx.set_override(addition.input_b(), 1);
    ctx.set_fallback(10_usize);

    assert_eq!(
        graph.compute_with_context(addition.output(), &ctx)?,
        1,
        "priortiy should be override > connected > fallback"
    );

    assert!(ctx
        .remove_override_untyped(&addition.input_b().into())
        .is_some());
    assert!(ctx
        .remove_override_untyped(&addition.input_b().into())
        .is_none());
    assert!(ctx.remove_fallback_untyped(TypeId::of::<usize>()).is_some());
    assert!(ctx.remove_fallback_untyped(TypeId::of::<usize>()).is_none());

    Ok(())
}
