//! # `ComputeGraph`
//!
//! `ComputeGraph` is a Rust library for building and executing directed acyclic graphs (DAGs) to handle complex,
//! interdependent computations with high efficiency. It provides a robust framework that prioritizes type safety,
//! while allowing runtime flexibility for changing the computation structure.
//!
//! ## Features
//!
//! - **Dynamic Graph Construction**: Nodes and connections can be added, removed, or modified at runtime, providing great flexibility.
//! - **Custom Node Implementation**: Users can define their own nodes with custom computation logic by using the [`node`] macro.
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

/// Define custom nodes for a [`ComputeGraph`]
///
/// This macro simplifies the creation of custom nodes by generating the necessary boilerplate code.
/// The names of the input parameters are automatically derived from the function parameters,
/// ignoring the first '_' if present. By default, the output parameter is named 'output'.
/// You can specify custom output names by using the `->` symbol followed by the desired output names.
/// If the node has multiple outputs, use a tuple like `(output_name_1, output_name_2)`.
///
/// For type safe usage, the handle returned when calling [`ComputeGraph::add_node`] will allow you to access the node's inputs and outputs.
/// The names of the functions will be `input_{name}` and `output_{name}`, except if the output name is `output`, in which case the function will be `output`.
/// Similarly, if the input name is `input`, the function will be `input`.
///
///
/// ## Examples
///
/// ### Single Output
///
/// If the output name is not specified, the macro will default to naming the output as "output".
///
/// ```rust
/// # use computegraph::{node, NodeFactory, ComputeGraph};
/// #[derive(Debug, Clone)]
/// struct Node {}
///
/// #[node(Node)]
/// fn run(&self) -> usize {
///     42
/// }
///
/// let mut graph = ComputeGraph::new();
/// let node = graph.add_node(Node {}, "node".to_string()).unwrap();
/// let result = graph.compute(node.output()).unwrap();
/// assert_eq!(result, 42);
/// # assert_eq!(<Node as NodeFactory>::outputs()[0].0, "output");
/// ```
///
/// ### Custom Output Name
///
/// You can specify a custom name for a single output.
///
/// ```rust
/// # use computegraph::{node, NodeFactory, ComputeGraph};
/// #[derive(Debug, Clone)]
/// struct Node {}
///
/// #[node(Node -> result)]
/// fn run(&self) -> String {
///     "Hello, world!".to_string()
/// }
///
/// let mut graph = ComputeGraph::new();
/// let node = graph.add_node(Node {}, "node".to_string()).unwrap();
/// let result = graph.compute(node.output_result()).unwrap();
/// assert_eq!(result, "Hello, world!");
/// # assert_eq!(<Node as NodeFactory>::outputs()[0].0, "result");
/// ```
///
/// ### Multiple Outputs
///
/// For nodes with multiple outputs, specify a tuple for the output parameter.
/// Each element of the tuple will be treated as a separate output.
///
/// ```rust
/// # use computegraph::{node, NodeFactory, ComputeGraph};
/// #[derive(Debug, Clone)]
/// struct Node {}
///
/// #[node(Node -> (greeting, target))]
/// fn run(&self) -> (String, String) {
///     ("Hello".to_string(), "world".to_string())
/// }
///
/// let mut graph = ComputeGraph::new();
/// let node = graph.add_node(Node {}, "node".to_string()).unwrap();
///
/// let greeting = graph.compute(node.output_greeting()).unwrap();
/// let target = graph.compute(node.output_target()).unwrap();
///
/// assert_eq!(greeting, "Hello");
/// assert_eq!(target, "world");
/// # assert_eq!(<Node as NodeFactory>::outputs()[0].0, "greeting");
/// # assert_eq!(<Node as NodeFactory>::outputs()[1].0, "target");
/// ```
///
/// ### Input parameters
///
/// Names for input parameters are derived from the function signature, ignoring the first '_' if present.
/// All input parameters should be references to the desired type. This macro
/// will then accept the underlying type without the reference as input.
///
/// ```rust
/// # use computegraph::{node, NodeFactory, ComputeGraph, InputPort};
/// # use std::any::TypeId;
/// # fn typeid<T: std::any::Any>(_: &T) -> TypeId {
/// #     std::any::TypeId::of::<T>()
/// # }
/// #[derive(Debug, Clone)]
/// struct Node {}
///
/// #[node(Node)]
/// fn run(&self, name: &String, age: &usize) -> String {
///    format!("{} is {} years old", name, age)
/// }
///
/// // Or equally:
/// #[derive(Debug, Clone)]
/// struct Node2 {}
///
/// #[node(Node2)]
/// fn run(&self, _name: &String, _age: &usize) -> String {
///    format!("{} is {} years old", _name, _age)
/// }
///
/// let mut graph = ComputeGraph::new();
/// let node = graph.add_node(Node {}, "node".to_string()).unwrap();
/// # let node2 = graph.add_node(Node2 {}, "node2".to_string()).unwrap();
///
/// let input_name = node.input_name();
/// let input_age = node.input_age();
/// assert_eq!(TypeId::of::<InputPort<String>>(), typeid(&input_name));
/// assert_eq!(TypeId::of::<InputPort<usize>>(), typeid(&input_age));
/// #
/// # assert_eq!(<Node as NodeFactory>::inputs()[0].0, "name");
/// # assert_eq!(<Node as NodeFactory>::inputs()[1].0, "age");
/// #
/// # // Test if really equal
/// # let input_name = node.input_name();
/// # let input_age = node.input_age();
/// # assert_eq!(TypeId::of::<InputPort<String>>(), typeid(&input_name));
/// # assert_eq!(TypeId::of::<InputPort<usize>>(), typeid(&input_age));
/// # assert_eq!(<Node as NodeFactory>::inputs(), <Node2 as NodeFactory>::inputs());
/// # assert_eq!(<Node as NodeFactory>::outputs(), <Node2 as NodeFactory>::outputs());
/// ```
pub use computegraph_macros::node;
use dyn_clone::DynClone;
use std::{
    any::{Any, TypeId},
    collections::{BTreeMap, HashSet},
    fmt,
};

/// Represents a computation graph.
///
/// The graph is a collection of nodes and connections between them, where nodes represent computation logic and connections
/// represent data flow between nodes.
#[derive(Default, Debug, Clone)]
pub struct ComputeGraph {
    nodes: Vec<GraphNode>,
    edges: Vec<Connection>,
}

/// Errors that can occur when calling [`ComputeGraph::compute`].
#[derive(thiserror::Error, Debug)]
pub enum ComputeError {
    #[error("Input port {0} not connected")]
    InputPortNotConnected(InputPortUntyped),
    #[error("Node {0} not found")]
    NodeNotFound(NodeHandle),
    #[error("Output port {port:?} not found in node {node:?}")]
    PortNotFound {
        node: NodeHandle,
        port: OutputPortUntyped,
    },
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
    InputPortAlreadyConnected {
        from: OutputPortUntyped,
        to: InputPortUntyped,
    },
    #[error("Node {0} not found")]
    NodeNotFound(NodeHandle),
    #[error("Input port {0} not found")]
    InputPortNotFound(InputPortUntyped),
    #[error("Output port {0} not found")]
    OutputPortNotFound(OutputPortUntyped),
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

trait ClonableAny: Any + DynClone + fmt::Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
}

impl Clone for Box<dyn ClonableAny> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(self.as_ref())
    }
}

impl<T> ClonableAny for T
where
    T: Any + DynClone + fmt::Debug + Send + Sync,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug)]
struct InputPortValue {
    port: InputPortUntyped,
    value: Box<dyn Any + Send>,
}

/// Set predefined values for [`ComputeGraph::compute_with_context`].
///
/// Use this container to:
/// - Override values passed to [`InputPort`]s
/// - Set fallback values for unconnected [`InputPort`]s
///
/// To be used with [`ComputeGraph::compute_with_context`] and [`ComputeGraph::compute_untyped_with_context`].
#[derive(Debug, Default)]
pub struct ComputationContext {
    overrides: Vec<InputPortValue>,
    default_values: Vec<(TypeId, Box<dyn Any + Send>)>,
}

impl ComputationContext {
    /// Create a new empty computation context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Manually override the connection of a [`InputPort`] with the specified value.
    ///
    /// Overriding the [`InputPort`] will pass `value` to the the node of `port`,
    /// no matter if it was connected or not.
    ///
    /// If the type is not known at compile time, use [`ComputationContext::set_override_untyped`] instead.
    ///
    /// # Arguments
    ///
    /// * `port` - The port to override.
    /// * `value` - The value which should be used instead
    pub fn set_override<T: Any + Send>(&mut self, port: InputPort<T>, value: T) {
        let port = port.into();

        self.overrides.retain(|o| o.port != port);
        self.overrides.push(InputPortValue {
            port,
            value: Box::new(value),
        });
    }

    /// Manually override the connection of a [`InputPortUntyped`] with the specified boxed value.
    ///
    /// Dynamic version of [`ComputationContext::set_override`].
    ///
    /// # Arguments
    ///
    /// * `port` - The port to override.
    /// * `value` - The boxed value which should be used instead
    pub fn set_override_untyped(&mut self, port: InputPortUntyped, value: Box<dyn Any + Send>) {
        self.overrides.retain(|o| o.port != port);
        self.overrides.push(InputPortValue { port, value });
    }

    /// Provide a fallback value to all unconnected [`InputPort`]s with the type 'T'
    ///
    /// This fallback will only be used if a [`InputPort`] required for the computation
    /// was unconnected, but required.
    ///
    /// If the type is not known at compile time, use [`ComputationContext::set_fallback_untyped`] instead.
    ///
    /// # Arguments
    ///
    /// * `value`: The value to use for all unconnected [`InputPort`]s of the given type.
    pub fn set_fallback<T: Any + Send>(&mut self, value: T) {
        let type_id = value.type_id();
        self.default_values.retain(|v| v.0 != type_id);
        self.default_values.push((type_id, Box::new(value)));
    }

    /// Provide a fallback value to all unconnected [`InputPortUntyped`]s with the contained type.
    ///
    /// Dynamic version of [`ComputationContext::set_override`].
    /// This will set the fallback to all [`InputPort`]s of the type contained in the box.
    ///
    /// # Arguments
    ///
    /// * `value`: The value to use for all unconnected [`InputPort`]s of the type.
    pub fn set_fallback_untyped(&mut self, value: Box<dyn Any + Send>) {
        let type_id = (*value).type_id();
        self.default_values.retain(|v| v.0 != type_id);
        self.default_values.push((type_id, value));
    }

    /// Remove a previously set override value, returning it in a box
    ///
    /// This method removes and returns a override, previously added using [`ComputationContext::set_override_untyped`].
    ///
    /// # Arguments
    ///
    /// * `port` - The untyped input port used to add the override
    ///
    /// # Returns
    ///
    /// An [`Option`] containing the override value if found, or `None` otherwise.
    pub fn remove_override_untyped(
        &mut self,
        port: &InputPortUntyped,
    ) -> Option<Box<dyn Any + Send>> {
        self.overrides
            .iter()
            .position(|o| &o.port == port)
            .map(|index| self.overrides.swap_remove(index).value)
    }

    /// Remove a previously set override value, returning it
    ///
    /// This method removes and returns a override, previously added using [`ComputationContext::set_override`].
    ///
    /// # Arguments
    ///
    /// * `port` - The input port used to add the override
    ///
    /// # Returns
    ///
    /// An [`Option`] containing the override value if found, or `None` otherwise.
    pub fn remove_override<T: 'static>(&mut self, port: &InputPort<T>) -> Option<T> {
        let index = self.overrides.iter().position(|o| o.port == port.port)?;
        let override_value = self.overrides.swap_remove(index);
        match override_value.value.downcast::<T>() {
            Ok(boxed) => Some(*boxed),
            Err(value) => {
                // Type mismatch, undo the removal
                self.overrides.push(InputPortValue {
                    value,
                    ..override_value
                });
                None
            }
        }
    }

    /// Remove a previously set fallback value, returning it
    ///
    /// This method removes and returns a fallback, previously added using [`ComputationContext::set_fallback`].
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the fallback value to retrieve.
    ///
    /// # Returns
    ///
    /// An [`Option`] containing the fallback value if found, or `None` otherwise.
    pub fn remove_fallback<T: 'static>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        let index = self.default_values.iter().position(|o| o.0 == type_id)?;
        let (id, value) = self.default_values.swap_remove(index);
        match value.downcast::<T>() {
            Ok(boxed) => Some(*boxed),
            Err(value) => {
                // Type mismatch, undo the removal
                self.default_values.push((id, value));
                None
            }
        }
    }

    /// Remove a previously set fallback value, returning it in a box
    ///
    /// This method removes and returns a fallback, previously added using [`ComputationContext::set_fallback_untyped`].
    ///
    /// # Arguments
    ///
    /// * `type_id` - The `TypeId` of the fallback value to retrieve.
    ///
    /// # Returns
    ///
    /// An [`Option`] containing the fallback value if found, or `None` otherwise.
    pub fn remove_fallback_untyped(&mut self, type_id: TypeId) -> Option<Box<dyn Any + Send>> {
        self.default_values
            .iter()
            .position(|o| o.0 == type_id)
            .map(|index| self.default_values.swap_remove(index).1)
    }
}

/// A container for storing and managing metadata associated with nodes in a computation graph.
///
/// The `Metadata` struct allows for the storage of arbitrary data types, identified by their type IDs.
/// This enables the attachment of various types of metadata to nodes in a type-safe manner.
#[derive(Debug, Default, Clone)]
pub struct Metadata {
    data: BTreeMap<TypeId, Box<dyn ClonableAny>>,
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
            .and_then(|v| v.as_ref().as_any().downcast_ref())
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
            .and_then(|v| v.as_mut().as_mut_any().downcast_mut())
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
    pub fn insert<T: 'static + std::clone::Clone + fmt::Debug + Send + Sync>(&mut self, value: T) {
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

/// A dynamic representation of a node in a compute graph.
///
/// This struct encapsulates the input and output port information
/// along with the executable node implementation.
#[derive(Clone, Debug)]
pub struct DynamicNode {
    inputs: Vec<(&'static str, TypeId)>,
    outputs: Vec<(&'static str, TypeId)>,
    executable: Box<dyn ExecutableNode>,
}

impl DynamicNode {
    /// Returns a slice of the input ports.
    #[must_use]
    pub fn inputs(&self) -> &[(&'static str, TypeId)] {
        &self.inputs
    }

    /// Returns a slice of the output ports.
    #[must_use]
    pub fn outputs(&self) -> &[(&'static str, TypeId)] {
        &self.outputs
    }
}

impl<T: NodeFactory + Clone + 'static> From<T> for DynamicNode {
    fn from(factory: T) -> Self {
        Self {
            inputs: T::inputs(),
            outputs: T::outputs(),
            executable: Box::new(factory),
        }
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

    /// Adds a dynamic node to the graph.
    ///
    /// This method is similar to `add_node`, but works with `DynamicNode`
    /// instances, allowing for more flexible node addition.
    ///
    /// # Arguments
    ///
    /// * `node_builder` - A `DynamicNode` instance representing the node to be added.
    /// * `name` - The name of the node, which must be unique within the graph.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `NodeHandle` for the newly added node if successful,
    /// or an `AddError` if the operation fails.
    ///
    /// # Errors
    ///
    /// Returns `AddError::DuplicateName` if a node with the given name already exists in the graph.
    pub fn add_node_dynamic(
        &mut self,
        node_builder: DynamicNode,
        name: String,
    ) -> Result<NodeHandle, AddError> {
        if self.nodes.iter().any(|n| n.handle.node_name == name) {
            return Err(AddError::DuplicateName(name));
        }

        let gnode = GraphNode {
            inputs: node_builder.inputs,
            outputs: node_builder.outputs,
            node: node_builder.executable,
            handle: NodeHandle { node_name: name },
            metadata: Metadata::default(),
        };

        let instance = gnode.handle.clone();
        self.nodes.push(gnode);
        Ok(instance)
    }

    /// Connects an output port to an input port with runtime type checking.
    ///
    /// This function connects an output port to an input port in the graph.
    /// When possible use the typed version of this function, [`ComputeGraph::connect`] that performs type checking at compile time.
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
    pub fn connect_untyped(
        &mut self,
        from: OutputPortUntyped,
        to: InputPortUntyped,
    ) -> Result<Connection, ConnectError> {
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

    /// Connects an output port to an input port.
    ///
    /// This function connects an output port to an input port in the graph.
    /// When the type is not known at compile time, use the untyped version of this function, [`ComputeGraph::connect_untyped`].
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
    pub fn connect<T>(
        &mut self,
        from: OutputPort<T>,
        to: InputPort<T>,
    ) -> Result<Connection, ConnectError> {
        self.connect_untyped(from.port, to.port)
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

    /// Computes the result for a given output port, returning a boxed value.
    ///
    /// This function is the untyped version of [`ComputeGraph::compute`].
    ///
    /// # Arguments
    ///
    /// * `output` - The output port to compute.
    ///
    /// # Returns
    ///
    /// A result containing the computed boxed value or an error.
    ///
    /// # Errors
    ///
    /// An error is returned if:
    /// - The node is not found.
    /// - An input port of the node ar a dependency of the node are not connected.
    /// - A cycle is detected in the graph.
    /// - A error occurs during computation (e.g. type returned by the node does not match the expected type).
    pub fn compute_untyped(
        &self,
        output: OutputPortUntyped,
    ) -> Result<Box<dyn Any + Send>, ComputeError> {
        let mut visited = HashSet::new();
        self.compute_recursive(output, &mut visited, None)
    }

    /// Computes the result for a given output port using the provided context, returning a boxed value.
    ///
    /// This function is the untyped version of [`ComputeGraph::compute_with_context`].
    ///
    /// This function behaves similarly to [`ComputeGraph::compute_untyped`], but uses
    /// the values given in the context as described in [`ComputationContext`].
    ///
    /// # Arguments
    ///
    /// * `output` - The output port to compute.
    /// * `context` - Collection of values used to perform the computation.
    ///
    /// # Returns
    ///
    /// A result containing the computed boxed value or an error.
    ///
    /// # Errors
    ///
    /// An error is returned if:
    /// - The node is not found.
    /// - The node has the incorrect output type
    /// - An input port of the node or a dependency of the node are not connected, and
    ///   no value is provided via the context
    /// - A cycle is detected in the graph.
    pub fn compute_untyped_with_context(
        &self,
        output: OutputPortUntyped,
        context: &ComputationContext,
    ) -> Result<Box<dyn Any + Send>, ComputeError> {
        let mut visited = HashSet::new();
        self.compute_recursive(output, &mut visited, Some(context))
    }

    /// Computes the result for a given output port.
    ///
    /// # Arguments
    ///
    /// * `output` - The output port to compute.
    ///
    /// # Returns
    ///
    /// A result containing the computed boxed value or an error.
    ///
    /// # Errors
    ///
    /// An error is returned if:
    /// - The node is not found.
    /// - The node has the incorrect output type
    /// - An input port of the node or a dependency of the node are not connected.
    /// - A cycle is detected in the graph.
    pub fn compute<T: 'static>(&self, output: OutputPort<T>) -> Result<T, ComputeError> {
        let res = self.compute_untyped(output.port.clone())?;
        let res = res
            .downcast::<T>()
            .map_err(|_| ComputeError::OutputTypeMismatch {
                node: output.port.node,
            })?;
        Ok(*res)
    }

    /// Computes the result for a given output port using the provided context
    ///
    /// This function behaves similarly to [`ComputeGraph::compute`], but uses
    /// the values given in the context as described in [`ComputationContext`].
    ///
    /// # Arguments
    ///
    /// * `output` - The output port to compute.
    /// * `context` - Collection of values used to perform the computation,
    ///
    /// # Returns
    ///
    /// A result containing the computed boxed value or an error.
    ///
    /// # Errors
    ///
    /// An error is returned if:
    /// - The node is not found.
    /// - The node has the incorrect output type
    /// - An input port of the node or a dependency of the node are not connected, and
    ///   no value is provided via the context
    /// - A cycle is detected in the graph.
    pub fn compute_with_context<T: 'static>(
        &self,
        output: OutputPort<T>,
        context: &ComputationContext,
    ) -> Result<T, ComputeError> {
        let res = self.compute_untyped_with_context(output.port.clone(), context)?;
        let res = res
            .downcast::<T>()
            .map_err(|_| ComputeError::OutputTypeMismatch {
                node: output.port.node,
            })?;
        Ok(*res)
    }

    fn compute_recursive(
        &self,
        output: OutputPortUntyped,
        visited: &mut HashSet<NodeHandle>,
        context: Option<&ComputationContext>,
    ) -> Result<Box<dyn Any + Send>, ComputeError> {
        enum OwnedOrBorrowed<'a, T: 'a + Send> {
            Owned(T),
            Borrowed(&'a T),
        }
        impl<'a, T: Send> OwnedOrBorrowed<'a, T> {
            const fn as_ref(&self) -> &T {
                match self {
                    OwnedOrBorrowed::Owned(t) => t,
                    OwnedOrBorrowed::Borrowed(t) => t,
                }
            }
        }
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
        let mut dependencies = vec![];

        for input in &output_node.inputs {
            if let Some(context) = context {
                // Check if we should use a override instead
                if let Some(port_value) = context
                    .overrides
                    .iter()
                    .find(|v| v.port.input_name == input.0)
                {
                    // Override was specified, use it instead
                    dependencies.push(OwnedOrBorrowed::Borrowed(&port_value.value));
                    continue;
                }
            }
            // Find the connection that provides the input
            let connection = self
                .edges
                .iter()
                .find(|c| c.to.node == output_handle && c.to.input_name == input.0);
            let connection = if let Some(connection) = connection {
                Ok(connection)
            } else {
                // Check if the context has a fallback value for this type
                if let Some(context) = context {
                    if let Some(v) = context.default_values.iter().find(|v| v.0 == input.1) {
                        // Found a fallback, we use that instead
                        dependencies.push(OwnedOrBorrowed::Borrowed(&v.1));
                        continue;
                    }
                }
                Err(ComputeError::InputPortNotConnected(InputPortUntyped {
                    node: output_handle.clone(),
                    input_name: input.0,
                }))
            }?;

            // Compute the result of the input
            let result = self.compute_recursive(connection.from.clone(), visited, context)?;
            dependencies.push(OwnedOrBorrowed::Owned(result));
        }

        // The introduction of OwnedOrBorrowed is necessary, since otherwise the computed dependencies
        // would be destroyed after each loop iteration. This converts the list back into the required format
        let dependencies: Vec<&Box<dyn Any + Send>> =
            dependencies.iter().map(OwnedOrBorrowed::as_ref).collect();
        // Run the node with the computed inputs
        let output_result = output_node.node.run(&dependencies);
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

/// Represents an input port of a node, without carrying type information.
///
/// See [`InputPort`] for the typed version, to use this, use untyped versions of functions like [`ComputeGraph::connect_untyped`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InputPortUntyped {
    pub node: NodeHandle,
    pub input_name: &'static str,
}

impl<T> From<InputPort<T>> for InputPortUntyped {
    fn from(value: InputPort<T>) -> Self {
        value.port
    }
}

impl fmt::Display for InputPortUntyped {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.node.node_name, self.input_name)
    }
}

impl InputPortUntyped {
    /// Converts an untyped input port to a typed input port.
    ///
    /// It is the responsibility of the caller to ensure that the type `T` is correct before calling this function.
    #[must_use]
    pub const fn to_typed<T>(self) -> InputPort<T> {
        InputPort {
            port_type: std::marker::PhantomData,
            port: self,
        }
    }
}

/// Represents an input port of a node.
///
/// A port is a connection point for data flow between nodes.
/// The input port is the point where data enters the node.
/// It is connected to an [`OutputPort`] of another node through a [`ComputeGraph::connect`] call.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InputPort<T> {
    pub port_type: std::marker::PhantomData<T>,
    pub port: InputPortUntyped,
}

impl<T> Clone for InputPort<T> {
    fn clone(&self) -> Self {
        Self {
            port_type: std::marker::PhantomData,
            port: self.port.clone(),
        }
    }
}

impl<T> fmt::Display for InputPort<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}.{}<{}>",
            self.port.node,
            self.port.input_name,
            std::any::type_name::<T>()
        )
    }
}

/// Represents an output port of a node, without carrying type information.
///
/// See [`OutputPort`] for the typed version, to use this, use untyped versions of functions like [`ComputeGraph::connect_untyped`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutputPortUntyped {
    pub node: NodeHandle,
    pub output_name: &'static str,
}

impl fmt::Display for OutputPortUntyped {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.node.node_name, self.output_name)
    }
}

impl<T> From<OutputPort<T>> for OutputPortUntyped {
    fn from(value: OutputPort<T>) -> Self {
        value.port
    }
}

impl OutputPortUntyped {
    /// Converts an untyped output port to a typed output port.
    ///
    /// It is the responsibility of the caller to ensure that the type `T` is correct before calling this function.
    #[must_use]
    pub const fn to_typed<T>(self) -> OutputPort<T> {
        OutputPort {
            port_type: std::marker::PhantomData,
            port: self,
        }
    }
}

/// Represents an output port of a node.
///
/// A port is a connection point for data flow between nodes.
/// The output port is the point where data exits the node.
/// It is connected to an [`InputPort`] of another node through a [`ComputeGraph::connect`] call.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutputPort<T> {
    pub port_type: std::marker::PhantomData<T>,
    pub port: OutputPortUntyped,
}

impl<T> Clone for OutputPort<T> {
    fn clone(&self) -> Self {
        Self {
            port_type: std::marker::PhantomData,
            port: self.port.clone(),
        }
    }
}

impl<T> fmt::Display for OutputPort<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}.{}<{}>",
            self.port.node,
            self.port.output_name,
            std::any::type_name::<T>()
        )
    }
}

/// Represents a handle to a node.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeHandle {
    pub node_name: String, // TODO: maybe associate with lifetime of the graph?
}

impl NodeHandle {
    /// Create a [`InputPortUntyped`] from the node handle and the input name.
    ///
    /// This is useful when connecting nodes, when the concrete type of the node is not known at compile time.
    ///
    /// # Parameters
    /// - `name`: The name of the input port.
    ///
    /// # Returns
    ///
    /// An [`InputPortUntyped`] representing the input port of the node.
    /// It is not guaranteed that the input port or the node exists.
    #[must_use]
    pub const fn to_input_port(self, name: &'static str) -> InputPortUntyped {
        InputPortUntyped {
            node: self,
            input_name: name,
        }
    }

    /// Create a [`OutputPortUntyped`] from the node handle and the output name.
    ///
    /// This is useful when connecting nodes, when the concrete type of the node is not known at compile time.
    ///
    /// # Parameters
    /// - `name`: The name of the output port.
    ///
    /// # Returns
    ///
    /// An [`OutputPortUntyped`] representing the output port of the node.
    /// It is not guaranteed that the output port or the node exists.
    #[must_use]
    pub const fn to_output_port(self, name: &'static str) -> OutputPortUntyped {
        OutputPortUntyped {
            node: self,
            output_name: name,
        }
    }
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
    from: OutputPortUntyped,
    to: InputPortUntyped,
}

/// Represents a node in the graph.
#[derive(Debug, Clone)]
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

    #[must_use]
    pub fn get_type_of_input(&self, input: &InputPortUntyped) -> Option<TypeId> {
        self.inputs
            .iter()
            .find(|i| i.0 == input.input_name)
            .map(|i| i.1)
    }

    #[must_use]
    pub fn get_type_of_output(&self, output: &OutputPortUntyped) -> Option<TypeId> {
        self.outputs
            .iter()
            .find(|i| i.0 == output.output_name)
            .map(|i| i.1)
    }
}

/// Trait for executing a node's computation logic.
///
/// This trait defines the interface for nodes that can perform computation
/// within a compute graph. Implementors of this trait are responsible for
/// defining the logic that processes input data and produces output data.
///
/// Implementors of this trait should always also implement the [`NodeFactory`] trait.
pub trait ExecutableNode: std::fmt::Debug + DynClone + Send + Sync {
    /// Executes the node's computation logic.
    ///
    /// This method takes boxed input data, processes it, and returns boxed output data.
    /// Input and output data are exactly as specified by the [`NodeFactory`] trait with
    /// [`NodeFactory::inputs`] and [`NodeFactory::outputs`].
    ///
    /// # Parameters
    ///
    /// - `input`: A slice of boxed dynamic values representing the input data.
    ///
    /// # Returns
    ///
    /// A vector of boxed dynamic values representing the output data.
    // TODO: add error handling
    fn run(&self, input: &[&Box<dyn Any + Send>]) -> Vec<Box<dyn Any + Send>>;
}

dyn_clone::clone_trait_object!(ExecutableNode);

/// Trait for building a node.
///
/// This trait defines the interface for creating nodes within a compute graph.
/// Implementors of this trait are responsible for specifying the input and output ports
/// exactly as they are used by the [`ExecutableNode::run`] method.
// TODO: describe the handle, add example usage
pub trait NodeFactory: ExecutableNode {
    /// The type of handle used to interact with the node, returned by [`ComputeGraph::add_node`].
    type Handle: Into<NodeHandle>;

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
