use computegraph::{node, ExecutableNode, NodeFactory};
use std::any::{Any, TypeId};

#[test]
fn test_macro_node() {
    #[derive(Debug, Clone)]
    struct Node1 {}
    #[node(Node1)]
    fn run(&self) {}

    #[derive(Debug, Clone)]
    struct Node2 {}
    #[node(Node2)]
    fn run(&self) -> usize {
        21
    }

    #[derive(Debug, Clone)]
    struct Node3 {}
    #[node(Node3 -> hello)]
    fn run(&self) -> String {
        "hello".to_string()
    }

    #[derive(Debug, Clone)]
    struct Node4 {}

    #[node(Node4 -> (hello, world))]
    fn run(&self) -> (String, String) {
        ("hello".to_string(), "world".to_string())
    }

    #[derive(Debug, Clone)]
    struct Node5 {}
    #[node(Node5)]
    fn run(&self, input: &usize) -> usize {
        *input
    }

    #[derive(Debug, Clone)]
    struct Node6 {}
    #[node(Node6 -> output)]
    fn run(&self, text: &String, repeat_count: &usize) -> String {
        text.repeat(*repeat_count)
    }

    // TODO: generics support

    assert_eq!(<Node1 as NodeFactory>::inputs(), vec![]);
    assert_eq!(<Node1 as NodeFactory>::outputs(), vec![]);
    let res = ExecutableNode::run(&Node1 {}, &[]);
    assert_eq!(res.len(), 0);

    assert_eq!(<Node2 as NodeFactory>::inputs(), vec![]);
    assert_eq!(
        <Node2 as NodeFactory>::outputs(),
        vec![("output", TypeId::of::<usize>())]
    );

    assert_eq!(<Node3 as NodeFactory>::inputs(), vec![]);
    assert_eq!(
        <Node3 as NodeFactory>::outputs(),
        vec![("hello", TypeId::of::<String>())]
    );

    assert_eq!(<Node4 as NodeFactory>::inputs(), vec![]);
    assert_eq!(
        <Node4 as NodeFactory>::outputs(),
        vec![
            ("hello", TypeId::of::<String>()),
            ("world", TypeId::of::<String>())
        ]
    );
    let res = ExecutableNode::run(&Node4 {}, &[]);
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].downcast_ref::<String>().unwrap(), "hello");
    assert_eq!(res[1].downcast_ref::<String>().unwrap(), "world");

    assert_eq!(
        <Node5 as NodeFactory>::inputs(),
        vec![("input", TypeId::of::<usize>())]
    );
    assert_eq!(
        <Node5 as NodeFactory>::outputs(),
        vec![("output", TypeId::of::<usize>())]
    );
    let res = ExecutableNode::run(
        &Node6 {},
        &[
            &(Box::new("hi".to_string()) as Box<dyn Any>),
            &(Box::new(3_usize) as Box<dyn Any>),
        ],
    );
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].downcast_ref::<String>().unwrap(), "hihihi");

    assert_eq!(
        <Node6 as NodeFactory>::inputs(),
        vec![
            ("text", TypeId::of::<String>()),
            ("repeat_count", TypeId::of::<usize>())
        ]
    );
    assert_eq!(
        <Node6 as NodeFactory>::outputs(),
        vec![("output", TypeId::of::<String>())]
    );
    let res = ExecutableNode::run(
        &Node6 {},
        &[
            &(Box::new("hi".to_string()) as Box<dyn Any>),
            &(Box::new(3_usize) as Box<dyn Any>),
        ],
    );
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].downcast_ref::<String>().unwrap(), "hihihi");
}
