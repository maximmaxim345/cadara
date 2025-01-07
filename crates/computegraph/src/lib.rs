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
//! ## Example: Basic Usage
//!
//! ```rust
//! use computegraph::{node, ComputeGraph, NodeFactory, ComputationCache, ComputationOptions};
//!
//! // Define a simple node that adds two numbers
//! #[derive(Debug, Clone, PartialEq)]
//! struct AddNode {}
//!
//! #[node(AddNode)]
//! fn run(&self, a: &i32, b: &i32) -> i32 {
//!     *a + *b
//! }
//!
//! // Define another node that multiplies two numbers
//! #[derive(Debug, Clone, PartialEq)]
//! struct MultiplyNode {}
//!
//! #[node(MultiplyNode)]
//! fn run(&self, a: &i32, b: &i32) -> i32 {
//!     *a * *b
//! }
//!
//! // Create a new compute graph
//! let mut graph = ComputeGraph::new();
//!
//! // Add nodes to the graph
//! let add_node = graph.add_node(AddNode {}, "add".to_string()).unwrap();
//! let multiply_node = graph.add_node(MultiplyNode {}, "multiply".to_string()).unwrap();
//!
//! // Connect nodes
//! graph.connect(add_node.output(), multiply_node.input_a()).unwrap();
//!
//! // Set input values for the add node
//! let mut context = computegraph::ComputationContext::new();
//! context.set_override(add_node.input_a(), 3);
//! context.set_override(add_node.input_b(), 4);
//! context.set_override(multiply_node.input_b(), 2);
//!
//! // Compute the result
//! let options = ComputationOptions {context: Some(&context) };
//! let result = graph.compute_with(multiply_node.output(), &options, None).unwrap();
//! assert_eq!(result, (3 + 4) * 2);
//! ```
//!
//! ## Example: Using the Cache
//!
//! ```rust
//! use computegraph::{node, ComputeGraph, NodeFactory, ComputationCache, ComputationOptions, ComputationContext};
//!
//! // Define a simple node that increments a number
//! #[derive(Debug, Clone, PartialEq)]
//! struct IncrementNode {}
//!
//! #[node(IncrementNode)]
//! fn run(&self, input: &i32) -> i32 {
//!     *input + 1
//! }
//!
//! // Create a new compute graph
//! let mut graph = ComputeGraph::new();
//!
//! // Add an increment node to the graph
//! let increment_node = graph.add_node(IncrementNode {}, "increment".to_string()).unwrap();
//!
//! // Set an initial input value
//! let mut context = ComputationContext::new();
//! context.set_override(increment_node.input(), 5);
//!
//! // Create a cache
//! let mut cache = ComputationCache::new();
//!
//! // Compute the result for the first time, populating the cache
//! let options = ComputationOptions { context: Some(&context) };
//! let result = graph.compute_with(increment_node.output(), &options, Some(&mut cache)).unwrap();
//! assert_eq!(result, 6);
//!
//! // Compute the result again with the same inputs; the cached value will be used
//! let result = graph.compute_with(increment_node.output(), &options, Some(&mut cache)).unwrap();
//! assert_eq!(result, 6); // Result is retrieved from cache
//!
//! // Change the input value
//! context.set_override(increment_node.input(), 10);
//! let options = ComputationOptions { context: Some(&context) };
//!
//! // Compute the result with the new input; the cache will be updated
//! let result = graph.compute_with(increment_node.output(), &options, Some(&mut cache)).unwrap();
//! assert_eq!(result, 11);
//! ```
//!
//! For more examples and usage, refer to the tests included in this crate.
//!

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
/// #[derive(Debug, Clone, PartialEq)]
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
/// #[derive(Debug, Clone, PartialEq)]
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
/// #[derive(Debug, Clone, PartialEq)]
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
/// #     TypeId::of::<T>()
/// # }
/// #[derive(Debug, Clone, PartialEq)]
/// struct Node {}
///
/// #[node(Node)]
/// fn run(&self, name: &String, age: &usize) -> String {
///    format!("{} is {} years old", name, age)
/// }
///
/// // Or equally:
/// #[derive(Debug, Clone, PartialEq)]
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
    collections::{BTreeMap, HashMap, VecDeque},
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
    #[error("Error while building schedule to run the graph: {0:?}")]
    ErrorBuildingSchedule(dagga::DaggaError),
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

trait CloneableAny: Any + DynClone + Send + Sync {
    fn as_any(&self) -> &dyn Any;

    fn as_mut_any(&mut self) -> &mut dyn Any;
}

impl Clone for Box<dyn CloneableAny> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(self.as_ref())
    }
}

impl<T> CloneableAny for T
where
    T: Any + DynClone + Send + Sync,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl fmt::Debug for Box<dyn CloneableAny> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Box<dyn CloneableAny>").finish()
    }
}

/// A version of [`Any`] that allows usage across threads.
#[diagnostic::on_unimplemented(
    message = "Trying to use non thread safe type for nodes",
    label = "`{Self}` does not implement both `Send` and `Sync`"
)]
pub trait SendSyncAny: Any + Send + Sync {
    /// Returns a reference to the object as a `dyn Any`.
    fn as_any(&self) -> &dyn Any;

    /// Returns a mutable reference to the object as a `dyn Any`.
    ///
    /// This method allows for runtime type checking and downcasting
    /// with mutable access.
    fn as_mut_any(&mut self) -> &mut dyn Any;

    /// Converts a `Box<dyn SendSyncAny>` to a `Box<dyn Any>`
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<T> SendSyncAny for T
where
    T: Any + Send + Sync,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl fmt::Debug for Box<dyn SendSyncAny> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Box<dyn SendSyncAny>").finish()
    }
}

#[diagnostic::on_unimplemented(
    message = "Trying to use an incomparable type as a cached node output",
    label = "Use of uncacheable type `{Self}` as a cached output",
    note = "Either implement `PartialEq` for `{Self}`",
    note = "Or if this is not possible, opt out of caching with `#[node(NodeName -> !)]` or `#[node(NodeName -> !output_name)]"
)]
pub trait SendSyncPartialEqAny: SendSyncAny {
    fn into_send_sync(self: Box<Self>) -> Box<dyn SendSyncAny>;
    fn as_ref(&self) -> &dyn SendSyncPartialEqAny;
    fn partial_eq(&self, other: &dyn SendSyncPartialEqAny) -> bool;
}

impl<T> SendSyncPartialEqAny for T
where
    T: SendSyncAny + PartialEq,
{
    fn into_send_sync(self: Box<Self>) -> Box<dyn SendSyncAny> {
        self
    }

    fn as_ref(&self) -> &dyn SendSyncPartialEqAny {
        self
    }

    fn partial_eq(&self, other: &dyn SendSyncPartialEqAny) -> bool {
        other
            .as_any()
            .downcast_ref::<T>()
            .map_or(false, |other| self == other)
    }
}

impl fmt::Debug for Box<dyn SendSyncPartialEqAny> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Box<dyn SendSyncPartialEqAny>").finish()
    }
}

trait CacheableFallback: SendSyncPartialEqAny + DynClone {
    fn as_partial_eq_any(&self) -> &dyn SendSyncPartialEqAny;
}

dyn_clone::clone_trait_object!(CacheableFallback);

impl<T> CacheableFallback for T
where
    T: SendSyncPartialEqAny + DynClone,
{
    fn as_partial_eq_any(&self) -> &dyn SendSyncPartialEqAny {
        self
    }
}

impl fmt::Debug for Box<dyn CacheableFallback> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Box<dyn CacheableFallback>").finish()
    }
}

#[derive(Debug)]
struct InputPortValue {
    port: InputPortUntyped,
    value: Box<dyn SendSyncAny>,
}

struct FallbackGenerator(Box<dyn Fn(&str) -> FallbackValue>);

impl fmt::Debug for FallbackGenerator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Box<dyn Fn(&'static str) -> NodeOutput>")
            .finish()
    }
}

#[derive(Debug)]
enum FallbackValue {
    Opaque(Box<dyn SendSyncAny>),
    Cacheable(Box<dyn CacheableFallback>),
}

impl FallbackValue {
    fn as_any(&self) -> &dyn Any {
        match self {
            Self::Opaque(b) => b.as_ref().as_any(),
            Self::Cacheable(b) => b.as_ref().as_any(),
        }
    }

    fn into_any(self) -> Box<dyn Any> {
        match self {
            Self::Opaque(b) => b.into_any(),
            Self::Cacheable(b) => b.into_any(),
        }
    }
}

#[derive(Debug)]
enum Fallback {
    Value(FallbackValue),
    Generator(FallbackGenerator),
}

/// Set predefined values for [`ComputeGraph::compute_with`].
///
/// Use this container to:
/// - Override values passed to [`InputPort`]s
/// - Set fallback values for unconnected [`InputPort`]s
///
/// To be used with [`ComputeGraph::compute_with`] and [`ComputeGraph::compute_untyped_with`].
#[derive(Debug, Default)]
pub struct ComputationContext {
    overrides: Vec<InputPortValue>,
    fallback_values: Vec<(TypeId, Fallback)>,
}

impl ComputationContext {
    /// Create a new empty computation context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Manually override the connection of a [`InputPort`] with the specified value.
    ///
    /// Overriding the [`InputPort`] will pass `value` to the node of `port`,
    /// no matter if it was connected or not.
    ///
    /// If the type is not known at compile time, use [`ComputationContext::set_override_untyped`] instead.
    ///
    /// # Arguments
    ///
    /// * `port` - The port to override.
    /// * `value` - The value which should be used instead
    pub fn set_override<T: SendSyncAny>(&mut self, port: InputPort<T>, value: T) {
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
    pub fn set_override_untyped(&mut self, port: InputPortUntyped, value: Box<dyn SendSyncAny>) {
        self.overrides.retain(|o| o.port != port);
        self.overrides.push(InputPortValue { port, value });
    }

    /// Provide a fallback value to all unconnected [`InputPort`]s with the type 'T'
    ///
    /// This fallback will only be used if a [`InputPort`] required for the computation
    /// was unconnected, but required.
    ///
    /// If using a [`ComputationCache`], nodes using fallback values will still be rerun every time.
    /// Use [`ComputationContext::set_fallback_cached`] to only rerun on changes.
    ///
    /// If the type is not known at compile time, use [`ComputationContext::set_fallback_untyped`] instead.
    ///
    /// # Arguments
    ///
    /// * `value`: The value to use for all unconnected [`InputPort`]s of the given type.
    pub fn set_fallback<T: SendSyncAny>(&mut self, value: T) {
        let type_id = value.type_id();
        self.fallback_values.retain(|v| v.0 != type_id);
        self.fallback_values.push((
            type_id,
            Fallback::Value(FallbackValue::Opaque(Box::new(value))),
        ));
    }

    /// Provide a comparable fallback value to all unconnected [`InputPort`]s with the type 'T'.
    ///
    /// This fallback will only be used if a [`InputPort`] required for the computation
    /// was unconnected, but required. This differs from [`ComputationContext::set_fallback`]
    /// in that the provided value must also implement [`PartialEq`] and [`Clone`].
    /// To detect changes, a copy of `value` will be made and held in the [`ComputationCache`].
    ///
    /// # Arguments
    ///
    /// * `value`: The value to use for all unconnected [`InputPort`]s of the given type.
    pub fn set_fallback_cached<T: SendSyncAny + Clone + PartialEq>(&mut self, value: T) {
        let type_id = value.type_id();
        self.fallback_values.retain(|v| v.0 != type_id);
        self.fallback_values.push((
            type_id,
            Fallback::Value(FallbackValue::Cacheable(Box::new(value))),
        ));
    }

    /// Provide a dynamically generated comparable fallback value to all unconnected [`InputPort`]s with the type 'T'.
    ///
    /// This fallback will only be used if a [`InputPort`] required for the computation
    /// was unconnected, but required.
    /// The generator will be called with the name of the node, to allow for different
    /// fallback values for different nodes.
    /// To detect changes, a copy of the generated value will be made and held in the [`ComputationCache`].
    /// That copy will only be used for comparisons.
    ///
    /// # Arguments
    ///
    /// * `generator`: A function that takes the node name and returns the fallback value to use for all unconnected [`InputPort`]s of the given type.
    pub fn set_fallback_generator<T: SendSyncAny + PartialEq + Clone>(
        &mut self,
        generator: impl Fn(&str) -> T + 'static,
    ) {
        let type_id = TypeId::of::<T>();
        self.fallback_values.retain(|v| v.0 != type_id);
        self.fallback_values.push((
            TypeId::of::<T>(),
            Fallback::Generator(FallbackGenerator(Box::new(move |s| {
                FallbackValue::Cacheable(Box::new(generator(s)))
            }))),
        ));
    }

    /// Provide a fallback value to all unconnected [`InputPortUntyped`]s with the contained type.
    ///
    /// Dynamic version of [`ComputationContext::set_override`].
    /// This will set the fallback to all [`InputPort`]s of the type contained in the box.
    ///
    /// # Arguments
    ///
    /// * `value`: The value to use for all unconnected [`InputPort`]s of the type.
    pub fn set_fallback_untyped(&mut self, value: Box<dyn SendSyncAny>) {
        let type_id = (*value).type_id();
        self.fallback_values.retain(|v| v.0 != type_id);
        self.fallback_values
            .push((type_id, Fallback::Value(FallbackValue::Opaque(value))));
    }

    /// Remove a previously set override value, returning it in a box
    ///
    /// This method removes and returns an override, previously added using [`ComputationContext::set_override_untyped`].
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
    ) -> Option<Box<dyn SendSyncAny>> {
        self.overrides
            .iter()
            .position(|o| &o.port == port)
            .map(|index| self.overrides.swap_remove(index).value)
    }

    /// Remove a previously set override value, returning it
    ///
    /// This method removes and returns an override, previously added using [`ComputationContext::set_override`].
    ///
    /// # Arguments
    ///
    /// * `port` - The input port used to add the override
    ///
    /// # Returns
    ///
    /// An [`Option`] containing the override value if found, or `None` otherwise.
    #[expect(clippy::missing_panics_doc, reason = "should not happen")]
    pub fn remove_override<T: 'static>(&mut self, port: &InputPort<T>) -> Option<T> {
        let index = self.overrides.iter().position(|o| o.port == port.port)?;
        match self.overrides[index]
            .value
            .as_ref()
            .as_any()
            .downcast_ref::<T>()
        {
            Some(_) => {
                let override_value = self.overrides.swap_remove(index);
                let b = override_value
                    .value
                    .into_any()
                    .downcast::<T>()
                    .expect("type was checked previously");
                Some(*b)
            }
            None => None,
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
    /// An [`Option`] containing the fallback value if found, or `None` if:
    /// - The fallback was added with [`ComputationContext::set_fallback_generator`], this function will still remove the generator.
    /// - The fallback was not set for this type.
    #[expect(clippy::missing_panics_doc, reason = "should not happen")]
    pub fn remove_fallback<T: 'static>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        let index = self.fallback_values.iter().position(|o| o.0 == type_id)?;
        match &self.fallback_values[index].1 {
            Fallback::Value(value) => match value.as_any().downcast_ref::<T>() {
                Some(_) => {
                    let override_value = self.fallback_values.swap_remove(index);
                    let b = if let Fallback::Value(value) = override_value.1 {
                        value
                            .into_any()
                            .downcast::<T>()
                            .expect("type was checked previously")
                    } else {
                        panic!("We just checked that this is of type `Value`");
                    };
                    Some(*b)
                }
                None => None,
            },
            Fallback::Generator(_) => {
                // Just drop the generator
                let _ = self.fallback_values.swap_remove(index);
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
    /// An [`Option`] containing the fallback value if found, or `None` if:
    /// - The fallback was added with [`ComputationContext::set_fallback_generator`], this function will still remove the generator.
    /// - The fallback was not set for this type.
    pub fn remove_fallback_untyped(&mut self, type_id: TypeId) -> Option<Box<dyn SendSyncAny>> {
        self.fallback_values
            .iter()
            .position(|o| o.0 == type_id)
            .map(|index| self.fallback_values.swap_remove(index).1)
            .and_then(|s| match s {
                Fallback::Value(FallbackValue::Opaque(b)) => Some(b),
                Fallback::Value(FallbackValue::Cacheable(b)) => Some(b.into_send_sync()),
                Fallback::Generator(_) => None,
            })
    }
}

#[derive(Debug)]
struct ComputedValue {
    /// The computed output value.
    value: NodeOutput,
    /// A flag indicating whether the value changed during this `compute()` call
    changed: bool,
}

#[derive(Debug)]
struct NodeCacheEntry {
    /// A boxed cloneable and executable representation of the node.
    ///
    /// This allows for comparing the current node with a previously cached version
    /// to determine if recomputation is necessary.
    node: Box<dyn ExecutableNode>,
    /// A vector of `ComputedValue` instances, each representing an output of the node.
    ///
    /// The order of `ComputedValue` instances in this vector corresponds to the order
    /// of outputs defined by the node.
    outputs: Vec<ComputedValue>,
}

/// A cache for storing the results of node computations in a `ComputeGraph`.
///
/// `ComputationCache` is designed to improve performance by avoiding redundant computations.
/// It stores the results of previous node executions, and copies of the nodes itself to detect changes.
/// A cache will be only reused if:
/// - The node is the same (checked with [`PartialEq`])
/// - All inputs are comparable and the same (also checked with [`PartialEq`])
///
/// ## Note
///
/// The last output will never be cached, since it's returned by `compute()`.
#[derive(Debug, Default)]
pub struct ComputationCache {
    /// A hash map that stores `NodeCacheEntry` instances, keyed by `NodeHandle`.
    ///
    /// This map serves as the primary storage for cached node results.
    node_cache: HashMap<NodeHandle, NodeCacheEntry>,
    /// A hash map that stores fallbacks used in the previous computation.
    ///
    /// The fallback in the [`ComputationContext`] will be compared to the one stored
    /// here to check if a recomputation is required.
    fallback_cache: HashMap<TypeId, Box<dyn CacheableFallback>>,
    /// A hash map that stores values generated by a [`FallbackGenerator`].
    ///
    /// Compared to [`Self::fallback_cache`], there can be multiple fallbacks with the same [`TypeId`].
    generated_fallback_cache: HashMap<(NodeHandle, TypeId), Box<dyn CacheableFallback>>,
}

impl ComputationCache {
    /// Creates a new empty cache for use with [`ComputeGraph::compute_with`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

/// Options to customize [`ComputeGraph::compute_with`].
///
/// This struct allows you to configure [`ComputeGraph::compute_with`] and [`ComputeGraph::compute_untyped_with`]
/// by providing a context.
#[derive(Debug, Default)]
pub struct ComputationOptions<'a> {
    /// An optional reference to a `ComputationContext`.
    ///
    /// The [`ComputationContext`] provides a way to override or supply default values
    /// for input ports during computation. If [`None`], no overrides or defaults are used.
    pub context: Option<&'a ComputationContext>,
}

/// A container for storing and managing metadata associated with nodes in a computation graph.
///
/// The `Metadata` struct allows for the storage of arbitrary data types, identified by their type IDs.
/// This enables the attachment of various types of metadata to nodes in a type-safe manner.
#[derive(Debug, Default, Clone)]
pub struct Metadata {
    data: BTreeMap<TypeId, Box<dyn CloneableAny>>,
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
    pub fn insert<T: 'static + Clone + fmt::Debug + Send + Sync>(&mut self, value: T) {
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
    /// Use [`ComputeGraph::compute_untyped_with`] when caching or a context are needed.
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
    ) -> Result<Box<dyn SendSyncAny>, ComputeError> {
        self.compute_untyped_with(output, &ComputationOptions::default(), None)
    }

    /// Computes the result for a given output port using the provided options, returning a boxed value.
    ///
    /// This function is the untyped version of [`ComputeGraph::compute_with`].
    ///
    /// Use the basic [`ComputeGraph::compute_untyped`] method when neither caching nor a context are needed.
    ///
    /// # Arguments
    ///
    /// * `output` - The output port to compute.
    /// * `options` - [`ComputationOptions`] to customize the computation, for example, by passing
    ///               in a [`ComputationContext`].
    /// * `cache` - A [`ComputationCache`] to reuse the output of previous runs when possible.
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
    #[expect(clippy::too_many_lines)]
    #[expect(clippy::missing_panics_doc, reason = "should not happen")]
    pub fn compute_untyped_with(
        &self,
        output: OutputPortUntyped,
        options: &ComputationOptions,
        cache: Option<&mut ComputationCache>,
    ) -> Result<Box<dyn SendSyncAny>, ComputeError> {
        struct Cache<'a> {
            fallback_cache: &'a mut HashMap<TypeId, Box<dyn CacheableFallback>>,
            generated_fallback_cache:
                &'a mut HashMap<(NodeHandle, TypeId), Box<dyn CacheableFallback>>,
        }
        enum InputValue<'a> {
            Borrowed(&'a dyn Any),
            Generated(FallbackValue),
        }
        impl InputValue<'_> {
            fn as_any(&self) -> &dyn Any {
                match self {
                    Self::Borrowed(v) => *v,
                    Self::Generated(v) => v.as_any(),
                }
            }
        }

        let mut visited: Vec<bool> = vec![false; self.nodes.len()];
        let mut queue: VecDeque<usize> = VecDeque::new();

        // Find the index of the node with the requested output port
        let output_node_index = self
            .nodes
            .iter()
            .enumerate()
            .find(|(_, n)| n.handle == output.node)
            .ok_or_else(|| ComputeError::NodeNotFound(output.node.clone()))?
            .0;

        queue.push_back(output_node_index);

        // Perform a breadth-first search to find all dependencies of the output node
        while let Some(node_index) = queue.pop_front() {
            if visited[node_index] {
                continue;
            }
            visited[node_index] = true;

            // Find dependencies of the current node
            let dependencies = self
                .edges
                .iter()
                .filter(|connection| connection.to.node == self.nodes[node_index].handle)
                // Skip dependencies that are overridden by the context
                .filter_map(|connection| {
                    if let Some(context) = options.context {
                        if context.overrides.iter().any(|override_value| {
                            override_value.port.node == connection.to.node
                                && override_value.port.input_name == connection.to.input_name
                        }) {
                            return None;
                        }
                    }
                    // Find the index of the node that provides the input
                    let dependency_index = self
                        .nodes
                        .iter()
                        .enumerate()
                        .find(|(_, node)| node.handle == connection.from.node)
                        .map(|(index, _)| index)?;
                    Some(Ok(dependency_index))
                });

            // Add dependencies to the queue
            for dependency in dependencies {
                queue.push_back(dependency?);
            }
        }

        // Create a DAG to represent the dependencies between nodes
        let mut dependency_graph = dagga::Dag::<usize, usize>::default();

        // Add nodes to the DAG, including their dependencies
        dependency_graph.add_nodes(self.nodes.iter().enumerate().filter_map(
            |(node_index, node)| {
                if visited[node_index] {
                    Some(
                        dagga::Node::new(node_index)
                            .with_name(node.handle.node_name.clone())
                            .with_reads(
                                // Add each input parameter as a read
                                node.inputs.iter().filter_map(|(input_name, _input_type)| {
                                    // This is the connection coming into this input
                                    let connection = self.edges.iter().find(|c| {
                                        c.to.node == node.handle && c.to.input_name == *input_name
                                    })?;
                                    // This is the index of the node, `node` depends on
                                    let result_node_index = self
                                        .nodes
                                        .iter()
                                        .enumerate()
                                        .find(|(_, n)| n.handle == connection.from.node)?
                                        .0;
                                    Some(result_node_index)
                                }),
                            )
                            .with_result(node_index),
                    )
                } else {
                    None
                }
            },
        ));

        // Build the execution schedule from the DAG
        let execution_schedule = dependency_graph
            .build_schedule()
            .map_err(|error| match error.source {
                dagga::DaggaError::Cycle { start: _, path: _ } => ComputeError::CycleDetected,
                err => ComputeError::ErrorBuildingSchedule(err),
            })?;

        // Create a map to store the computed results of each node
        let (computed_results, mut cache) = if let Some(c) = cache {
            (
                &mut c.node_cache,
                Some(Cache {
                    fallback_cache: &mut c.fallback_cache,
                    generated_fallback_cache: &mut c.generated_fallback_cache,
                }),
            )
        } else {
            (&mut HashMap::new(), None)
        };

        // Execute the nodes in the order specified by the schedule
        for batch in execution_schedule.batches {
            for batch_node in batch {
                let node_index = *batch_node.inner();
                let current_node = &self.nodes[node_index];

                // If recompute_required is false, we can reuse the result in the cache.
                // Checking if input arguments where changed is done at the input gathering
                // stage below.
                let mut recompute_required = if cache.is_some() {
                    computed_results
                        .get(&current_node.handle)
                        .map_or(true, |node_cache| {
                            // We already computed a node with the same name, check if it changed
                            let is_equal = node_cache
                                .node
                                .as_ref()
                                .partial_eq(current_node.node.as_ref().as_ref());

                            !is_equal
                        })
                } else {
                    true
                };

                let mut input_values = vec![];

                // Gather input values for the current node
                for input in &current_node.inputs {
                    // Check if the input is overridden by the context
                    if let Some(context) = options.context {
                        if let Some(override_value) =
                            context.overrides.iter().find(|override_value| {
                                override_value.port.node == current_node.handle
                                    && override_value.port.input_name == input.0
                            })
                        {
                            // Use the override value
                            input_values
                                .push(InputValue::Borrowed(override_value.value.as_ref().as_any()));
                            recompute_required = true;
                            continue;
                        }
                    }

                    // Find the connection that provides the input
                    let connection = self.edges.iter().find(|connection| {
                        connection.to.node == current_node.handle
                            && connection.to.input_name == input.0
                    });

                    let connection = if let Some(connection) = connection {
                        Ok(connection)
                    } else {
                        // Check if a fallback value is provided by the context
                        if let Some(context) = options.context {
                            if let Some(fallback_value) = context
                                .fallback_values
                                .iter()
                                .find(|fallback_value| fallback_value.0 == input.1)
                            {
                                // Use the fallback value
                                let value = match &fallback_value.1 {
                                    Fallback::Value(value) => {
                                        // Check if the value changed
                                        if match &value {
                                            FallbackValue::Opaque(_) => true,
                                            FallbackValue::Cacheable(value) => {
                                                cache.as_mut().map_or(true, |cache| {
                                                    if let Some(fb) =
                                                        cache.fallback_cache.get_mut(&input.1)
                                                    {
                                                        if fb.partial_eq(value.as_partial_eq_any())
                                                        {
                                                            false
                                                        } else {
                                                            // update
                                                            *fb = value.clone();
                                                            true
                                                        }
                                                    } else {
                                                        // insert
                                                        cache
                                                            .fallback_cache
                                                            .insert(input.1, value.clone());
                                                        true
                                                    }
                                                })
                                            }
                                        } {
                                            recompute_required = true;
                                        }
                                        InputValue::Borrowed(value.as_any())
                                    }
                                    Fallback::Generator(gen) => {
                                        let handle = current_node.handle.clone();
                                        let generated_value = (gen.0)(&handle.node_name);

                                        let should_recompute = match &mut cache {
                                            None => true,
                                            Some(cache) => match cache
                                                .generated_fallback_cache
                                                .get_mut(&(handle.clone(), input.1))
                                            {
                                                None => {
                                                    // No existing cache entry
                                                    if let FallbackValue::Cacheable(b) =
                                                        &generated_value
                                                    {
                                                        cache
                                                            .generated_fallback_cache
                                                            .insert((handle, input.1), b.clone());
                                                    }
                                                    true
                                                }
                                                Some(fb) => match &generated_value {
                                                    FallbackValue::Opaque(_) => true,
                                                    FallbackValue::Cacheable(b) => {
                                                        if fb.partial_eq(b.as_partial_eq_any()) {
                                                            false // Value didn't change
                                                        } else {
                                                            *fb = b.clone(); // Update cache
                                                            true
                                                        }
                                                    }
                                                },
                                            },
                                        };

                                        if should_recompute {
                                            recompute_required = true;
                                        }
                                        InputValue::Generated(generated_value)
                                    }
                                };
                                input_values.push(value);
                                continue;
                            }
                        }
                        // No connection or fallback value found, return an error
                        Err(ComputeError::InputPortNotConnected(InputPortUntyped {
                            node: current_node.handle.clone(),
                            input_name: input.0,
                        }))
                    }?;

                    let providing_node_handle = &connection.from.node;
                    let providing_node = self
                        .nodes
                        .iter()
                        .find(|node| node.handle == *providing_node_handle)
                        .ok_or_else(|| ComputeError::NodeNotFound(providing_node_handle.clone()))?;

                    let providing_output_name = connection.from.output_name;
                    let providing_output = providing_node
                        .outputs
                        .iter()
                        .enumerate()
                        .find(|(_, (name, _))| *name == providing_output_name)
                        .ok_or_else(|| ComputeError::PortNotFound {
                            node: providing_node_handle.clone(),
                            port: OutputPortUntyped {
                                node: providing_node_handle.clone(),
                                output_name: providing_output_name,
                            },
                        })?;
                    let providing_output_index = providing_output.0;

                    // Get the computed result of the providing node
                    let cached_output = &computed_results
                        .get(&providing_node.handle)
                        .and_then(|e| e.outputs.get(providing_output_index))
                        .expect("Result should be computed in a previous batch");
                    if cached_output.changed {
                        recompute_required = true;
                    }
                    input_values.push(InputValue::Borrowed(cached_output.value.as_any()));
                }

                if recompute_required {
                    // Execute the node and store the result
                    let input: Vec<_> = input_values.iter().map(InputValue::as_any).collect();
                    let output_values = current_node.node.run(&input);

                    // Validate output types
                    if output_values
                        .iter()
                        .zip(current_node.outputs.iter())
                        .any(|(result, output)| result.type_id() != output.1)
                        // Check if the length is the same, .zip() stops at the shortest iterator
                        || output_values.len() != current_node.outputs.len()
                    {
                        return Err(ComputeError::OutputTypeMismatch {
                            node: current_node.handle.clone(),
                        });
                    }

                    // Store the results of the node, indexed by (node handle, output name)
                    if let Some(a) = computed_results.get_mut(&current_node.handle) {
                        for (cache, output) in a.outputs.iter_mut().zip(output_values.into_iter()) {
                            cache.changed = match (&cache.value, &output) {
                                (NodeOutput::Opaque(_any1), NodeOutput::Opaque(_any2)) => true,
                                (NodeOutput::Comparable(any1), NodeOutput::Comparable(any2)) => {
                                    !any1.as_ref().partial_eq(any2.as_ref())
                                }
                                (_, _) => panic!(
                                    "This should not happen. Node changed its output type???"
                                ),
                            };
                            cache.value = output;
                        }
                    } else {
                        computed_results
                            .entry(current_node.handle.clone())
                            .insert_entry(NodeCacheEntry {
                                node: current_node.node.clone(),
                                outputs: output_values
                                    .into_iter()
                                    .map(|output| {
                                        let was_changed = true;
                                        ComputedValue {
                                            value: output,
                                            changed: was_changed,
                                        }
                                    })
                                    .collect(),
                            });
                    }
                }
            }
        }

        if let Some(cache) = cache {
            // Discard all cached values of nodes that are no longer in the graph.
            computed_results.retain(|node, _cache| self.nodes.iter().any(|n| n.handle == *node));
            for (_, c) in computed_results.iter_mut() {
                for a in &mut c.outputs {
                    a.changed = false;
                }
            }
            if let Some(context) = options.context {
                cache.fallback_cache.retain(|t, _cache| {
                    context
                        .fallback_values
                        .iter()
                        .any(|(fb_type, fb)| fb_type == t && matches!(fb, Fallback::Value(_)))
                });
                cache.generated_fallback_cache.retain(|(_node, t), _cache| {
                    context
                        .fallback_values
                        .iter()
                        .any(|(fb_type, fb)| fb_type == t && matches!(fb, Fallback::Generator(_)))
                });
            } else {
                cache.fallback_cache.clear();
                cache.generated_fallback_cache.clear();
            }
        }

        // Return the computed result for the requested output port
        if self.nodes.iter().all(|n| n.handle != output.node) {
            Err(ComputeError::NodeNotFound(output.node))
        } else {
            let result_index = self.nodes[output_node_index]
                .outputs
                .iter()
                .enumerate()
                .find(|(_, (name, _))| *name == output.output_name)
                .ok_or_else(|| ComputeError::PortNotFound {
                    node: output.node.clone(),
                    port: output.clone(),
                })?
                .0;
            Ok(computed_results
                .remove_entry(&output.node)
                .expect("Should be computed by now")
                .1
                .outputs
                .swap_remove(result_index)
                .value
                .into_send_sync())
        }
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
    #[deprecated]
    pub fn compute_untyped_with_context(
        &self,
        output: OutputPortUntyped,
        context: &ComputationContext,
    ) -> Result<Box<dyn SendSyncAny>, ComputeError> {
        self.compute_untyped_with(
            output,
            &ComputationOptions {
                context: Some(context),
            },
            None,
        )
    }

    /// Computes the result for a given output port.
    ///
    /// Use [`ComputeGraph::compute_with`] when caching or a context are needed.
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
            .into_any()
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
    #[deprecated]
    pub fn compute_with_context<T: 'static>(
        &self,
        output: OutputPort<T>,
        context: &ComputationContext,
    ) -> Result<T, ComputeError> {
        let res = self.compute_untyped_with(
            output.port.clone(),
            &ComputationOptions {
                context: Some(context),
            },
            None,
        )?;
        let res = res
            .into_any()
            .downcast::<T>()
            .map_err(|_| ComputeError::OutputTypeMismatch {
                node: output.port.node,
            })?;
        Ok(*res)
    }

    /// Computes the result for a given output port using the provided options.
    ///
    /// This function is the primary way to execute computations in the [`ComputeGraph`].
    /// It takes [`ComputationOptions`] to enable caching and/or provide a [`ComputationContext`].
    ///
    /// Use the basic [`ComputeGraph::compute`] method when neither caching nor a context are needed.
    ///
    /// # Arguments
    ///
    /// * `output` - The output port to compute.
    /// * `options` - [`ComputationOptions`] to customize the computation, for example, by passing
    ///               in a [`ComputationContext`].
    /// * `cache` - A [`ComputationCache`] to reuse the output of previous runs when possible.
    ///
    /// # Returns
    ///
    /// A result containing the computed value or an error.
    ///
    /// # Errors
    ///
    /// An error is returned if:
    /// - The node is not found.
    /// - The node has the incorrect output type.
    /// - An input port of the node or a dependency of the node is not connected, and
    ///   no value is provided via the context.
    /// - A cycle is detected in the graph.
    pub fn compute_with<T: 'static>(
        &self,
        output: OutputPort<T>,
        options: &ComputationOptions,
        cache: Option<&mut ComputationCache>,
    ) -> Result<T, ComputeError> {
        let res = self.compute_untyped_with(output.port.clone(), options, cache)?;
        let res = res
            .into_any()
            .downcast::<T>()
            .map_err(|_| ComputeError::OutputTypeMismatch {
                node: output.port.node,
            })?;
        Ok(*res)
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

/// Type returned by an [`OutputPort`] of a [`ExecutableNode`].
#[derive(Debug)]
pub enum NodeOutput {
    /// Should be returned if the type does not implement [`PartialEq`].
    Opaque(Box<dyn SendSyncAny>),
    /// Should be returned if the type implements [`PartialEq`].
    ///
    /// The comparison will be used to detect changes when running with a cache.
    Comparable(Box<dyn SendSyncPartialEqAny>),
}

impl NodeOutput {
    #[must_use]
    pub fn type_id(&self) -> TypeId {
        match self {
            Self::Opaque(a) => a.as_ref().type_id(),
            Self::Comparable(a) => a.as_ref().type_id(),
        }
    }
    #[must_use]
    pub fn as_any(&self) -> &dyn Any {
        match self {
            Self::Opaque(a) => a.as_ref().as_any(),
            Self::Comparable(a) => a.as_ref().as_any(),
        }
    }
    #[must_use]
    pub fn into_send_sync(self) -> Box<dyn SendSyncAny> {
        match self {
            Self::Opaque(a) => a,
            Self::Comparable(a) => a.into_send_sync(),
        }
    }
}

/// Trait for executing a node's computation logic.
///
/// This trait defines the interface for nodes that can perform computation
/// within a compute graph. Implementors of this trait are responsible for
/// defining the logic that processes input data and produces output data.
///
/// Implementors of this trait should always also implement the [`NodeFactory`] trait.
pub trait ExecutableNode: fmt::Debug + DynClone + SendSyncPartialEqAny {
    /// Executes the node's computation logic.
    ///
    /// This method takes boxed input data, processes it, and returns boxed output data.
    /// Input and output data are exactly as specified by the [`NodeFactory`] trait with
    /// [`NodeFactory::inputs`] and [`NodeFactory::outputs`].
    ///
    /// # Parameters
    ///
    /// - `input`: A slice of dynamic values representing the input data.
    ///
    /// # Returns
    ///
    /// A vector of boxed dynamic values representing the output data.
    // TODO: add error handling
    fn run(&self, input: &[&dyn Any]) -> Vec<NodeOutput>;
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

// Used by the proc-macro
#[doc(hidden)]
pub mod __private {
    pub const fn check_cached_input_impl<T: PartialEq + Send + Sync>() {}
    pub const fn check_uncached_input_impl<T: Send + Sync>() {}
}
