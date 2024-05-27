//! # `ComputeGraph`
//!
//! `ComputeGraph` is a Rust library for building and executing directed acyclic graphs (DAGs) to handle complex,
//! interdependent computations with high efficiency. It provides a robust framework that prioritizes type safety,
//! while allowing runtime flexibility for changing the computation structure.
//!
//! ## Features
//!
//! - **Dynamic Graph Construction**: Nodes and connections can be added, removed, or modified at runtime, providing great flexibility.
//! - **Custom Node Implementation**: Users can define their own nodes with custom computation logic by implementing the [`NodeFactory`] trait.
//! - **Concurrency Support**: Nodes that can be computed independently are executed in parallel, enhancing performance.
//! - **Cache Optimization**: The graph automatically caches intermediate results to avoid redundant computations.
//!
//! ## Usage
//!
//! This crate is particularly useful for scenarios where complex, dependent computations need to be modeled and executed efficiently, or
//! where the computation structure changes dynamically at runtime. It was developed for use in `CADara`'s viewport system,
//! where the scene graph is dynamically built every frame, necessitating efficient caching of past results.
//!
//! For examples and usage, refer to the tests included in this crate.

#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cognitive_complexity)]

use std::{
    any::{Any, TypeId},
    collections::{BTreeMap, HashSet},
    fmt,
};

/// Represents a computation graph.
///
/// The graph is a collection of nodes and connections between them, where nodes represent computation logic and connections
/// represent data flow between nodes.
#[derive(Default, Debug)]
pub struct ComputeGraph {
    nodes: Vec<GraphNode>,
    edges: Vec<Connection>,
}

/// Errors that can occur when calling [`ComputeGraph::compute`].
#[derive(thiserror::Error, Debug)]
pub enum ComputeError {
    #[error("Input port {0} not connected")]
    InputPortNotConnected(InputPort),
    #[error("Node {0} not found")]
    NodeNotFound(NodeHandle),
    #[error("Output port {port:?} not found in node {node:?}")]
    PortNotFound { node: NodeHandle, port: OutputPort },
    #[error("Cycle detected in the computation graph")]
    CycleDetected,
    #[error("Output type mismatch when computing node {node:?}")]
    OutputTypeMismatch { node: NodeHandle },
}

/// Errors that can occur when connecting nodes with [`ComputeGraph::connect`].
#[derive(thiserror::Error, Debug)]
pub enum ConnectError {
    #[error("Type mismatch for output: expected {expected:?}, found {found:?}")]
    TypeMismatch { expected: TypeId, found: TypeId },
    #[error("Connection already exists from {from:?} to {to:?}")]
    InputPortAlreadyConnected { from: OutputPort, to: InputPort },
    #[error("Node {0} not found")]
    NodeNotFound(NodeHandle),
    #[error("Input port {0} not found")]
    InputPortNotFound(InputPort),
    #[error("Output port {0} not found")]
    OutputPortNotFound(OutputPort),
}

/// Errors that can occur during node removal through [`ComputeGraph::remove_node`].
#[derive(thiserror::Error, Debug)]
pub enum RemoveNodeError {
    #[error("Node with handle {0} not found")]
    NodeNotFound(NodeHandle),
}

/// Errors that can occur during disconnecting nodes with [`ComputeGraph::disconnect`].
#[derive(thiserror::Error, Debug)]
pub enum DisconnectError {
    #[error("Connection not found")]
    ConnectionNotFound,
}

/// Errors that can occur when adding new nodes with [`ComputeGraph::add_node`].
#[derive(thiserror::Error, Debug)]
pub enum AddError {
    #[error("Node with the name {0} already exists")]
    DuplicateName(String),
}

/// A container for storing and managing metadata associated with nodes in a computation graph.
///
/// The `Metadata` struct allows for the storage of arbitrary data types, identified by their type IDs.
/// This enables the attachment of various types of metadata to nodes in a type-safe manner.
#[derive(Debug, Default)]
pub struct Metadata {
    data: BTreeMap<TypeId, Box<dyn Any>>,
}

impl Metadata {
    /// Creates a new, empty `Metadata` instance.
    ///
    /// # Returns
    ///
    /// A new `Metadata` instance with no data.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieves a reference to the metadata of the specified type.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the metadata to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the metadata if it exists, or `None` if no metadata of the specified type is found.
    #[must_use]
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.data
            .get(&TypeId::of::<T>())
            .and_then(|v| v.downcast_ref())
    }

    /// Retrieves a mutable reference to the metadata of the specified type.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the metadata to retrieve.
    ///
    /// # Returns
    ///
    /// An `Option` containing a mutable reference to the metadata if it exists, or `None` if no metadata of the specified type is found.
    #[must_use]
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.data
            .get_mut(&TypeId::of::<T>())
            .and_then(|v| v.downcast_mut())
    }

    /// Inserts metadata of the specified type.
    ///
    /// If metadata of the same type already exists, it will be replaced.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the metadata to insert.
    ///
    /// # Arguments
    ///
    /// * `value` - The metadata value to insert.
    pub fn insert<T: 'static>(&mut self, value: T) {
        self.data.insert(TypeId::of::<T>(), Box::new(value));
    }

    /// Removes metadata of the specified type.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the metadata to remove.
    pub fn remove<T: 'static>(&mut self) {
        self.data.remove(&TypeId::of::<T>());
    }
}

impl ComputeGraph {
    /// Creates a new, empty `ComputeGraph`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a node to the graph.
    ///
    /// # Arguments
    ///
    /// * `node_builder` - The builder for the node to be added.
    /// * `name` - The name of the node, must be unique for the whole graph.
    ///
    /// # Returns
    ///
    /// A handle to the newly added node.
    ///
    /// # Errors
    ///
    /// An error is returned if the node name is not unique.
    pub fn add_node<N: NodeFactory + 'static>(
        &mut self,
        node_builder: N,
        name: String,
    ) -> Result<N::Handle, AddError> {
        if self.nodes.iter().any(|n| n.handle.node_name == name) {
            return Err(AddError::DuplicateName(name));
        }

        let gnode = GraphNode {
            inputs: N::inputs(),
            outputs: N::outputs(),
            node: Box::new(node_builder),
            handle: NodeHandle { node_name: name },
            metadata: Metadata::default(),
        };
        let instance = N::create_handle(&gnode); // TODO: maybe this should not be defined by the impl
        self.nodes.push(gnode);
        Ok(instance)
    }

    /// Connects an output port to an input port.
    ///
    /// # Arguments
    ///
    /// * `from` - The output port.
    /// * `to` - The input port.
    ///
    /// # Returns
    ///
    /// A result containing the connection or an error.
    ///
    /// # Errors
    ///
    /// An error is returned if:
    /// - The input port is already connected.
    /// - The nodes or ports do not exist.
    /// - The types of the two ports do not match.
    pub fn connect(&mut self, from: OutputPort, to: InputPort) -> Result<Connection, ConnectError> {
        // Check if the input port is already connected
        if self.edges.iter().any(|e| e.to == to) {
            return Err(ConnectError::InputPortAlreadyConnected { to, from });
        }

        // Find the nodes and ports
        let from_node = self
            .nodes
            .iter()
            .find(|n| n.handle == from.node)
            .ok_or_else(|| ConnectError::NodeNotFound(from.node.clone()))?;
        let to_node = self
            .nodes
            .iter()
            .find(|n| n.handle == to.node)
            .ok_or_else(|| ConnectError::NodeNotFound(to.node.clone()))?;

        let from_port = from_node
            .outputs
            .iter()
            .find(|o| o.0 == from.output_name)
            .ok_or_else(|| ConnectError::OutputPortNotFound(from.clone()))?;

        let to_port = to_node
            .inputs
            .iter()
            .find(|i| i.0 == to.input_name)
            .ok_or_else(|| ConnectError::InputPortNotFound(to.clone()))?;

        // Check if the types of the ports match
        if from_port.1 != to_port.1 {
            return Err(ConnectError::TypeMismatch {
                expected: to_port.1,
                found: from_port.1,
            });
        }

        // Create the connection
        let connection = Connection { from, to };
        self.edges.push(connection.clone());

        Ok(connection)
    }

    /// Removes a node from the graph.
    ///
    /// # Arguments
    ///
    /// * `node` - The handle of the node to be removed.
    ///
    /// # Returns
    ///
    /// A result indicating success or an error.
    ///
    /// # Errors
    ///
    /// An error is returned if the node is not found in the graph.
    pub fn remove_node(&mut self, node: impl Into<NodeHandle>) -> Result<(), RemoveNodeError> {
        // TODO: maybe make this fail silently?
        let node_handle = node.into();

        // Remove all connections associated with the node
        self.edges
            .retain(|conn| conn.from.node != node_handle && conn.to.node != node_handle);

        // Remove the node itself
        if !self.nodes.iter().any(|n| n.handle == node_handle) {
            return Err(RemoveNodeError::NodeNotFound(node_handle));
        }
        self.nodes.retain(|n| n.handle != node_handle);

        Ok(())
    }

    /// Disconnects a connection.
    ///
    /// # Arguments
    ///
    /// * `connection` - The connection to be disconnected.
    ///
    /// # Returns
    ///
    /// A result indicating success or an error.
    ///
    /// # Errors
    ///
    /// An error is returned if the connection is not found in the graph.
    pub fn disconnect(&mut self, connection: &Connection) -> Result<(), DisconnectError> {
        if !self.edges.contains(connection) {
            return Err(DisconnectError::ConnectionNotFound);
        }
        self.edges.retain(|conn| conn != connection);

        Ok(())
    }

    /// Computes the result for a given output port.
    ///
    /// # Arguments
    ///
    /// * `output` - The output port to compute.
    ///
    /// # Returns
    ///
    /// A result containing the computed value or an error.
    ///
    /// # Errors
    ///
    /// An error is returned if:
    /// - The node is not found.
    /// - An input port of the node ar a dependency of the node are not connected.
    /// - A cycle is detected in the graph.
    /// - A error occurs during computation (e.g. type returned by the node does not match the expected type).
    pub fn compute(&self, output: OutputPort) -> Result<Box<dyn Any>, ComputeError> {
        let mut visited = HashSet::new();
        self.compute_recursive(output, &mut visited)
    }

    fn compute_recursive(
        &self,
        output: OutputPort,
        visited: &mut HashSet<NodeHandle>,
    ) -> Result<Box<dyn Any>, ComputeError> {
        // For now we use a simple, but more inefficient approach for computing the result:
        // Here we simply recursively compute the dependencies of the requested node in breadth first order.
        //
        // This code can later be improved in multiple ways:
        // 1. Caching:
        // If we encounter a node that was already computed with the same input (by hashing the input parameters),
        // we reuse the result using a hash map.
        // 2. Cycle detection:
        // Currently, cycles are not supported and result in a stack overflow.
        // 3. Parallel computation
        // The system should detect independent nodes and be able to compute their results simultaneously
        // If the need arises, we could also support optimized computation of multiple OutputPort in one call to
        // compute(). This shhould then also be paralelized if possible.

        // Find the node with the requested output port
        let output_node = self
            .nodes
            .iter()
            .find(|n| n.handle == output.node)
            .ok_or_else(|| {
                ComputeError::NodeNotFound(NodeHandle {
                    node_name: output.node.node_name.clone(),
                })
            })?;
        let output_handle = output_node.handle.clone();

        // Check for cycles, we use a simple set to detect if in the current path we already visited the node
        if visited.contains(&output_handle) {
            return Err(ComputeError::CycleDetected);
        }
        visited.insert(output_handle.clone());

        // Find the index of the output port
        let output_result_index = output_node
            .outputs
            .iter()
            .position(|o| o.0 == output.output_name)
            .ok_or_else(|| ComputeError::PortNotFound {
                node: output_handle.clone(),
                port: output,
            })?;

        // Compute all dependencies recursively
        let mut dependency_results = vec![];

        for input in &output_node.inputs {
            // Find the connection that provides the input
            let connection = self
                .edges
                .iter()
                .find(|c| c.to.node == output_handle && c.to.input_name == input.0)
                .ok_or_else(|| {
                    ComputeError::InputPortNotConnected(InputPort {
                        node: output_handle.clone(),
                        input_name: input.0,
                    })
                })?;

            // Compute the result of the input
            let result = self.compute_recursive(connection.from.clone(), visited)?;
            dependency_results.push(result);
        }

        // Run the node with the computed inputs
        let output_result = output_node.node.run(&dependency_results);
        // check if the result has the correct type
        if output_result
            .iter()
            .zip(output_node.outputs.iter())
            .any(|(result, output)| (**result).type_id() != output.1)
            // .zip() will stop at the shortest iterator, so we need to check the length separately
            || output_result.len() != output_node.outputs.len()
        {
            return Err(ComputeError::OutputTypeMismatch {
                node: output_handle.clone(),
            });
        }
        let output = output_result
            .into_iter()
            .nth(output_result_index)
            .expect("this should not happen, since we checked the length before");

        // Return the result, we can not use clone here, because the type is not known at compile time

        // Remove the node from the visited set after computation
        visited.remove(&output_handle);

        Ok(output)
    }

    /// Returns an iterator over the nodes in the graph.
    pub fn iter_nodes(&self) -> impl Iterator<Item = &GraphNode> {
        self.nodes.iter()
    }

    /// Gets a node by its handle.
    ///
    /// This function searches for a node within the graph using the provided handle and returns a reference to the node if found.
    ///
    /// # Arguments
    ///
    /// * `handle` - A reference to the handle of the node to be retrieved.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the node if found, or `None` if no node with the given handle exists.
    #[must_use]
    pub fn get_node(&self, handle: &NodeHandle) -> Option<&GraphNode> {
        self.nodes.iter().find(|node| &node.handle == handle)
    }

    /// Gets a mutable reference to a node by its handle.
    ///
    /// This function searches for a node within the graph using the provided handle and returns a mutable reference to the node if found.
    ///
    /// # Arguments
    ///
    /// * `handle` - A reference to the handle of the node to be retrieved.
    ///
    /// # Returns
    ///
    /// An `Option` containing a mutable reference to the node if found, or `None` if no node with the given handle exists.
    #[must_use]
    pub fn get_node_mut(&mut self, handle: &NodeHandle) -> Option<&mut GraphNode> {
        self.nodes.iter_mut().find(|node| &node.handle == handle)
    }
}

/// Represents an input port of a node.
///
/// A port is a connection point for data flow between nodes.
/// The input port is the point where data enters the node.
/// It is connected to an [`OutputPort`] of another node through a [`ComputeGraph::connect`] call.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InputPort {
    pub node: NodeHandle,
    pub input_name: &'static str,
}

impl fmt::Display for InputPort {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.node.node_name, self.input_name)
    }
}

/// Represents an output port of a node.
///
/// A port is a connection point for data flow between nodes.
/// The output port is the point where data exits the node.
/// It is connected to an [`InputPort`] of another node through a [`ComputeGraph::connect`] call.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutputPort {
    pub node: NodeHandle,
    pub output_name: &'static str,
}

impl fmt::Display for OutputPort {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.node.node_name, self.output_name)
    }
}

/// Represents a handle to a node.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeHandle {
    pub node_name: String, // TODO: maybe associate with lifetime of the graph?
}

impl fmt::Display for NodeHandle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.node_name)
    }
}

/// Represents a connection between two nodes.
///
/// Represents a directed edge in the graph, where data flows from the `from` node to the `to`
/// node, as specified through the [`ComputeGraph::connect`] method.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Connection {
    from: OutputPort,
    to: InputPort,
}

/// Represents a node in the graph.
#[derive(Debug)]
pub struct GraphNode {
    inputs: Vec<(&'static str, TypeId)>,
    outputs: Vec<(&'static str, TypeId)>,
    node: Box<dyn ExecutableNode>,
    handle: NodeHandle,
    pub metadata: Metadata,
}

impl GraphNode {
    #[must_use]
    pub const fn handle(&self) -> &NodeHandle {
        &self.handle
    }

    #[must_use]
    pub const fn get_inputs(&self) -> &Vec<(&'static str, TypeId)> {
        &self.inputs
    }

    #[must_use]
    pub const fn get_outputs(&self) -> &Vec<(&'static str, TypeId)> {
        &self.outputs
    }
}

/// Trait for executing a node's computation logic.
pub trait ExecutableNode: std::fmt::Debug {
    fn run(&self, input: &[Box<dyn Any>]) -> Vec<Box<dyn Any>>;
}

/// Trait for building a node.
/// TODO: this should be reworked for better type safety.
pub trait NodeFactory: ExecutableNode {
    type Handle;

    /// Returns a vector of tuples representing the input ports of the node.
    ///
    /// Each tuple contains the name of the input port and its type identifier.
    ///
    /// # Returns
    ///
    /// A vector of tuples where each tuple consists of:
    /// - A static string representing the name of the input port.
    /// - A `TypeId` representing the type of the input port.
    // TODO: add support of Option<T> to mark an input as optional
    fn inputs() -> Vec<(&'static str, TypeId)>;

    /// Returns a vector of tuples representing the output ports of the node.
    ///
    /// Each tuple contains the name of the output port and its type identifier.
    ///
    /// # Returns
    ///
    /// A vector of tuples where each tuple consists of:
    /// - A static string representing the name of the output port.
    /// - A `TypeId` representing the type of the output port.
    fn outputs() -> Vec<(&'static str, TypeId)>;

    /// Creates a handle for interacting with the node.
    ///
    /// This method takes a reference to a `GraphNode` and returns a handle
    /// that can be used to reference the node's ports.
    /// This output of this method is passed to the user when a node is added to the graph using [`ComputeGraph::add_node`].
    ///
    /// # Parameters
    ///
    /// - `gnode`: A reference to the `GraphNode` for which the handle is being created.
    ///
    /// # Returns
    ///
    /// A handle of type `Self::Handle` that can be used to interact with the node.
    fn create_handle(gnode: &GraphNode) -> Self::Handle;
}

#[cfg(test)]
mod tests {
    use anyhow::{anyhow, Result};
    use std::any::TypeId;

    use super::*;

    #[derive(Debug)]
    struct TestNodeConstant {
        value: usize,
    }

    #[derive(Debug)]
    struct TestNodeConstantHandle {
        handle: NodeHandle,
    }

    impl TestNodeConstantHandle {
        pub fn port_output(&self) -> OutputPort {
            OutputPort {
                node: self.handle.clone(),
                output_name: "output",
            }
        }
    }

    impl TestNodeConstant {
        pub const fn new(value: usize) -> Self {
            Self { value }
        }
    }

    impl ExecutableNode for TestNodeConstant {
        fn run(&self, _input: &[Box<dyn Any>]) -> Vec<Box<dyn Any>> {
            vec![Box::new(self.value)]
        }
    }

    impl NodeFactory for TestNodeConstant {
        type Handle = TestNodeConstantHandle;

        fn inputs() -> Vec<(&'static str, TypeId)> {
            vec![] // TODO: return type should not be a Vec
        }

        fn outputs() -> Vec<(&'static str, TypeId)> {
            vec![("output", TypeId::of::<usize>())]
        }

        fn create_handle(gnode: &GraphNode) -> Self::Handle {
            Self::Handle {
                handle: gnode.handle().clone(),
            }
        }
    }

    #[derive(Debug)]
    struct TestNodeAddition {}

    #[derive(Debug)]
    struct TestNodeAdditionHandle {
        handle: NodeHandle,
    }

    impl TestNodeAdditionHandle {
        pub fn port_input_a(&self) -> InputPort {
            InputPort {
                node: self.handle.clone(),
                input_name: "a",
            }
        }
        pub fn port_input_b(&self) -> InputPort {
            InputPort {
                node: self.handle.clone(),
                input_name: "b", // TODO: multiple ports with the same name should error out
            }
        }
        pub fn port_output(&self) -> OutputPort {
            OutputPort {
                node: self.handle.clone(),
                output_name: "result",
            }
        }
    }

    impl TestNodeAddition {
        pub const fn new() -> Self {
            Self {}
        }
    }

    impl ExecutableNode for TestNodeAddition {
        fn run(&self, input: &[Box<dyn Any>]) -> Vec<Box<dyn Any>> {
            let a = input[0]
                .downcast_ref::<usize>()
                .expect("expected usize as input");
            let b = input[1]
                .downcast_ref::<usize>()
                .expect("expected usize as input");
            vec![Box::new(a + b)]
        }
    }

    impl NodeFactory for TestNodeAddition {
        type Handle = TestNodeAdditionHandle;

        fn inputs() -> Vec<(&'static str, TypeId)> {
            vec![("a", TypeId::of::<usize>()), ("b", TypeId::of::<usize>())]
        }

        fn outputs() -> Vec<(&'static str, TypeId)> {
            vec![("result", TypeId::of::<usize>())]
        }

        fn create_handle(gnode: &GraphNode) -> Self::Handle {
            Self::Handle {
                handle: gnode.handle().clone(),
            }
        }
    }

    #[derive(Debug)]
    struct TestNodeNumToString {}

    #[derive(Debug)]
    struct TestNodeNumToStringHandle {
        handle: NodeHandle,
    }

    impl TestNodeNumToStringHandle {
        pub fn port_input(&self) -> InputPort {
            InputPort {
                node: self.handle.clone(),
                input_name: "input",
            }
        }
        pub fn port_output(&self) -> OutputPort {
            OutputPort {
                node: self.handle.clone(),
                output_name: "result",
            }
        }
    }

    impl TestNodeNumToString {
        pub const fn new() -> Self {
            Self {}
        }
    }

    impl ExecutableNode for TestNodeNumToString {
        fn run(&self, input: &[Box<dyn Any>]) -> Vec<Box<dyn Any>> {
            let a = input[0]
                .downcast_ref::<usize>()
                .expect("expected usize as input");
            vec![Box::new(a.to_string())]
        }
    }

    impl NodeFactory for TestNodeNumToString {
        type Handle = TestNodeNumToStringHandle;
        fn inputs() -> Vec<(&'static str, TypeId)> {
            vec![("input", TypeId::of::<usize>())]
        }
        fn outputs() -> Vec<(&'static str, TypeId)> {
            vec![("result", TypeId::of::<String>())]
        }
        fn create_handle(gnode: &GraphNode) -> Self::Handle {
            Self::Handle {
                handle: gnode.handle().clone(),
            }
        }
    }

    #[test]
    fn test_basic_graph() -> Result<()> {
        let mut graph = ComputeGraph::new();
        let value1 = graph.add_node(TestNodeConstant::new(9), "value1".to_string())?;
        let value2 = graph.add_node(TestNodeConstant::new(10), "value2".to_string())?;

        let addition = graph.add_node(TestNodeAddition::new(), "addition".to_string())?;

        graph.connect(value1.port_output(), addition.port_input_a())?;
        graph.connect(value2.port_output(), addition.port_input_b())?;

        let value1_result = graph
            .compute(value1.port_output())?
            .downcast::<usize>()
            .or(Err(anyhow!("result is not of type usize")))?;
        let value2_result = graph
            .compute(value2.port_output())?
            .downcast::<usize>()
            .or(Err(anyhow!("result is not of type usize")))?;
        let addition_result = graph
            .compute(addition.port_output())?
            .downcast::<usize>()
            .or(Err(anyhow!("result is not of type usize")))?;

        assert_eq!(*value1_result, 9);
        assert_eq!(*value2_result, 10);
        assert_eq!(*addition_result, 19);

        Ok(())
    }

    #[test]
    fn test_diamond_dependencies() -> Result<()> {
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

        let mut graph = ComputeGraph::new();
        let value1 = graph.add_node(TestNodeConstant::new(5), "value1".to_string())?;
        let value2 = graph.add_node(TestNodeConstant::new(7), "value2".to_string())?;
        let value3 = graph.add_node(TestNodeConstant::new(3), "value3".to_string())?;

        let addition1 = graph.add_node(TestNodeAddition::new(), "addition1".to_string())?;
        let addition2 = graph.add_node(TestNodeAddition::new(), "addition2".to_string())?;
        let addition3 = graph.add_node(TestNodeAddition::new(), "addition3".to_string())?;
        let addition4 = graph.add_node(TestNodeAddition::new(), "addition4".to_string())?;

        graph.connect(value1.port_output(), addition1.port_input_a())?;
        graph.connect(value2.port_output(), addition1.port_input_b())?;
        graph.connect(value2.port_output(), addition2.port_input_a())?;
        graph.connect(value3.port_output(), addition2.port_input_b())?;
        graph.connect(addition2.port_output(), addition3.port_input_a())?;
        graph.connect(addition2.port_output(), addition3.port_input_b())?;
        graph.connect(addition1.port_output(), addition4.port_input_a())?;
        graph.connect(addition3.port_output(), addition4.port_input_b())?;

        let result = graph
            .compute(addition4.port_output())?
            .downcast::<usize>()
            .or(Err(anyhow!("result is not of type usize")))?;
        assert_eq!(*result, function(5, 7, 3));

        Ok(())
    }

    #[test]
    fn test_invalid_graph_missing_input() -> Result<()> {
        let mut graph = ComputeGraph::new();
        let value = graph.add_node(TestNodeConstant::new(5), "value".to_string())?;
        let addition = graph.add_node(TestNodeAddition::new(), "addition".to_string())?;

        graph.connect(value.port_output(), addition.port_input_a())?;

        match graph.compute(addition.port_output()) {
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

        graph.connect(value.port_output(), to_string.port_input())?;
        graph.connect(value.port_output(), addition.port_input_a())?;
        let res = graph.connect(to_string.port_output(), addition.port_input_b());
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

        graph.connect(node1.port_output(), node2.port_input_a())?;
        graph.connect(node2.port_output(), node3.port_input_a())?;
        graph.connect(node3.port_output(), node1.port_input_a())?;

        graph.connect(value.port_output(), node2.port_input_b())?;
        graph.connect(value.port_output(), node3.port_input_b())?;
        graph.connect(value.port_output(), node1.port_input_b())?;

        // The graph contains a cycle: node1 -> node2 -> node3 -> node1 -> ...
        let result = graph.compute(node1.port_output());
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_edge_disconnection() -> Result<()> {
        let mut graph = ComputeGraph::new();
        let value = graph.add_node(TestNodeConstant::new(5), "value".to_string())?;
        let one = graph.add_node(TestNodeConstant::new(1), "one".to_string())?;
        let addition = graph.add_node(TestNodeAddition::new(), "addition".to_string())?;
        let to_string = graph.add_node(TestNodeNumToString::new(), "to_string".to_string())?;

        let value_to_addition = graph.connect(value.port_output(), addition.port_input_a())?;
        graph.connect(one.port_output(), addition.port_input_b())?;
        graph.connect(addition.port_output(), to_string.port_input())?;

        // Test that the graph works before disconnecting the edge
        let result = graph
            .compute(to_string.port_output())?
            .downcast::<String>()
            .or(Err(anyhow!("result is not of type String")))?;
        assert_eq!(*result, "6".to_string());

        // Disconnect the edge between value and addition nodes
        graph.disconnect(&value_to_addition)?;

        // Test that the graph fails after disconnecting the edge with the expected error
        match graph.compute(to_string.port_output()) {
            Err(ComputeError::InputPortNotConnected(port)) => {
                assert_eq!(port.node, addition.handle);
                assert_eq!(port.input_name, "a");
            }
            _ => panic!("Expected ComputeError::InputPortNotConnected"),
        }

        // Now reconnect the edge and test that the graph works again
        graph.connect(value.port_output(), addition.port_input_a())?;
        let result = graph
            .compute(to_string.port_output())?
            .downcast::<String>()
            .or(Err(anyhow!("result is not of type String")))?;
        assert_eq!(*result, "6".to_string());

        Ok(())
    }

    #[test]
    fn test_disconnected_subgraphs() -> Result<()> {
        let mut graph = ComputeGraph::new();

        // Subgraph 1: Addition
        let value1 = graph.add_node(TestNodeConstant::new(5), "value1".to_string())?;
        let value2 = graph.add_node(TestNodeConstant::new(7), "value2".to_string())?;
        let addition1 = graph.add_node(TestNodeAddition::new(), "addition1".to_string())?;
        graph.connect(value1.port_output(), addition1.port_input_a())?;
        graph.connect(value2.port_output(), addition1.port_input_b())?;

        // Subgraph 2: Addition
        let value3 = graph.add_node(TestNodeConstant::new(3), "value3".to_string())?;
        let value4 = graph.add_node(TestNodeConstant::new(4), "value4".to_string())?;
        let addition2 = graph.add_node(TestNodeAddition::new(), "addition2".to_string())?;
        graph.connect(value3.port_output(), addition2.port_input_a())?;
        graph.connect(value4.port_output(), addition2.port_input_b())?;

        // Compute the results of the disconnected subgraphs independently
        let addition1_result = graph
            .compute(addition1.port_output())?
            .downcast::<usize>()
            .or(Err(anyhow!("result is not of type usize")))?;
        assert_eq!(*addition1_result, 12);

        let addition2_result = graph
            .compute(addition2.port_output())?
            .downcast::<usize>()
            .or(Err(anyhow!("result is not of type usize")))?;
        assert_eq!(*addition2_result, 7);

        Ok(())
    }

    #[test]
    fn test_node_removal() -> Result<()> {
        let mut graph = ComputeGraph::new();
        let value1 = graph.add_node(TestNodeConstant::new(5), "value1".to_string())?;
        let value2 = graph.add_node(TestNodeConstant::new(7), "value2".to_string())?;
        let addition = graph.add_node(TestNodeAddition::new(), "addition".to_string())?;

        graph.connect(value1.port_output(), addition.port_input_a())?;
        graph.connect(value2.port_output(), addition.port_input_b())?;

        // Compute the result before removing a node
        let result_before_removal = graph
            .compute(addition.port_output())?
            .downcast::<usize>()
            .or(Err(anyhow!("result is not of type usize")))?;
        assert_eq!(*result_before_removal, 12);

        // Remove the 'value2' node from the graph
        graph.remove_node(value2.handle)?;

        // After removing 'value2', the 'addition' node should have a missing input
        match graph.compute(addition.port_output()) {
            Err(ComputeError::InputPortNotConnected(port)) => {
                assert_eq!(port.node, addition.handle);
                assert_eq!(port.input_name, "b");
            }
            _ => panic!("Expected ComputeError::InputPortNotConnected"),
        }

        // Ensure that the 'value1' node can still be computed
        let value1_result = graph
            .compute(value1.port_output())?
            .downcast::<usize>()
            .or(Err(anyhow!("result is not of type usize")))?;
        assert_eq!(*value1_result, 5);

        // Now connect value1 to both inputs of the addition node
        graph.connect(value1.port_output(), addition.port_input_b())?;

        // Compute the result after reconnecting the edge
        let result = graph
            .compute(addition.port_output())?
            .downcast::<usize>()
            .or(Err(anyhow!("result is not of type usize")))?;
        assert_eq!(*result, 10);

        Ok(())
    }

    #[test]
    fn test_connect_already_connected() -> Result<()> {
        let mut graph = ComputeGraph::new();

        let value1 = graph.add_node(TestNodeConstant::new(5), "value1".to_string())?;
        let value2 = graph.add_node(TestNodeConstant::new(7), "value2".to_string())?;
        let to_string = graph.add_node(TestNodeNumToString::new(), "to_string".to_string())?;

        graph.connect(value1.port_output(), to_string.port_input())?;
        let res = graph.connect(value2.port_output(), to_string.port_input());
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
}
