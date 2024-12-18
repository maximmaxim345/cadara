use crate::ViewportEvent;
use computegraph::{
    ComputationContext, ComputeGraph, DynamicNode, InputPort, InputPortUntyped, NodeFactory,
    NodeHandle, OutputPort, OutputPortUntyped,
};
use project::ProjectView;
use std::any::{Any, TypeId};

/// Errors that can occur when creating a new [`ViewportPlugin`] or [`DynamicViewportPlugin`]
#[derive(thiserror::Error, Debug)]
pub enum ViewportPluginValidationError {
    /// Plugin is missing required output port 'output'
    #[error("Plugin is missing required output port 'output'")]
    MissingOutputPort,
    /// Output port 'scene' is missing or has invalid type
    #[error("Output port 'scene' is missing or has invalid type")]
    InvalidSceneOutputPort,
    /// Incompatible configuration for 'input' and 'scene' input ports
    #[error("Incompatible configuration for 'input' and 'scene' input ports")]
    InputPortMismatch,
}

/// Errors that can occur when adding a new plugin to the [`ViewportPipeline`]
#[derive(thiserror::Error, Debug)]
pub enum PipelineAddError {
    /// Mismatch between output type of last layer and input type
    #[error(
        "Mismatch between output type of last layer ({output_type:?}) and input type ({input_type:?})"
    )]
    TypeMismatch {
        input_type: TypeId,
        output_type: TypeId,
    },
    /// Cannot add subsequent plugin to empty pipeline
    #[error("Cannot add subsequent plugin to empty pipeline")]
    SubsequentPluginInEmptyPipeline,
}

/// Errors that can occur when executing a [`ViewportPipeline`]
#[derive(thiserror::Error, Debug)]
pub enum ExecuteError {
    /// [`ViewportPipeline`] is empty
    #[error("ViewportPipeline is empty")]
    EmptyPipeline,
    /// No 'result' output found
    #[error("No 'result' output found")]
    NoResultOutput,
    /// Failed to execute the final [`SceneGraph`]
    #[error("Failed to execute the final SceneGraph: {0}")]
    ComputeError(#[from] computegraph::ComputeError),
}

#[derive(Debug, Clone)]
struct ViewportPluginNode {
    node: computegraph::NodeHandle,
    output: computegraph::OutputPortUntyped,
    scene_output: computegraph::OutputPort<SceneGraph>,
}

#[derive(Clone)]
pub struct SceneGraphBuilder<T> {
    pub graph: ComputeGraph,
    pub initial_state: OutputPort<T>,
    pub render_node: RenderNodePorts<T>,
    pub update_node: UpdateNodePorts<T>,
}

#[derive(Clone)]
pub struct RenderNodePorts<T> {
    // state in
    pub state_in: InputPort<T>,
    // primitive out
    pub primitive_out: OutputPort<Box<dyn iced::widget::shader::Primitive>>,
}

#[derive(Clone)]
pub struct UpdateNodePorts<T> {
    // event in
    pub event_in: InputPort<ViewportEvent>,
    // state in
    pub state_in: InputPort<T>,
    // state out
    pub state_out: OutputPort<T>,
}

#[derive(Debug, Clone)]
pub struct SceneGraph {
    pub graph: ComputeGraph,
    init_state: OutputPortUntyped,
    render_state_in: InputPortUntyped,
    render_primitive_out: OutputPort<Box<dyn iced::widget::shader::Primitive>>,
    update_event_in: InputPort<ViewportEvent>,
    update_state_in: InputPortUntyped,
    update_state_out: OutputPortUntyped,
}

impl<T> From<SceneGraphBuilder<T>> for SceneGraph {
    fn from(scene_graph: SceneGraphBuilder<T>) -> Self {
        Self {
            graph: scene_graph.graph,
            init_state: scene_graph.initial_state.into(),
            render_state_in: scene_graph.render_node.state_in.into(),
            render_primitive_out: scene_graph.render_node.primitive_out,
            update_event_in: scene_graph.update_node.event_in,
            update_state_in: scene_graph.update_node.state_in.into(),
            update_state_out: scene_graph.update_node.state_out.into(),
        }
    }
}

/// Represents a pipeline for managing and executing viewport plugins.
///
/// The [`ViewportPipeline`] struct is the core component for managing and executing
/// [`ViewportPlugin`]s, which themselves are used to describe the exact behavior of
/// the viewport.
///
/// TODO: add examples
#[derive(Debug, Default, Clone)]
pub struct ViewportPipeline {
    graph: ComputeGraph,
    nodes: Vec<ViewportPluginNode>,
}

#[derive(Default, Debug)]
pub struct ViewportPipelineState {
    state: Option<Box<dyn Any + Send>>,
}

/// Represents the position of a plugin in the viewport pipeline.
#[derive(Debug, Clone)]
enum PluginPosition {
    /// Indicates that the plugin is the first in the pipeline.
    ///
    /// An `Initial` plugin:
    /// - Does not receive input from previous plugins.
    /// - Is responsible for initializing the scene graph
    Initial,
    /// Indicates that the plugin is added after other plugins in the pipeline.
    ///
    /// A `Subsequent` plugin:
    /// - Receives input from the previous plugin in the pipeline.
    /// - May add to or transform the scene graph created by earlier plugins.
    Subsequent,
}

/// A plugin for the [`ViewportPipeline`].
///
/// A [`ViewportPlugin`] is used to define the behavior of the viewport, and can be either an
/// initial plugin (no input) or a subsequent plugin (receives input from the previous plugin).
///
/// To create a new [`ViewportPlugin`], use the [`ViewportPlugin::new`] method on a 'node' created
/// by the [`computegraph::node`] macro.
///
/// If the specific plugin type is not known at compile time, use [`DynamicViewportPlugin`] instead.
///
/// # Requirements
///
/// For a 'node' to be a valid [`ViewportPlugin`], it must adhere to the following requirements:
/// 1. It must have an output port named "output". The type of "output" is application-specific, but
///    should contain any data needed for subsequent plugins to change the scene graph.
/// 2. It must have an output port named "scene" of type [`SceneGraph`] that contains
///    the scene graph, which is used to render the viewport.
///
/// If the plugin is an subsequent plugin, (i.e., the plugin should be connected to the "scene" and "output" output ports of the previous plugin),
/// it also must additionally implement the following requirements:
/// 3. It must have an input port named "input". The type of "input" should be exactly the same as the type of "output" of the previous plugin.
/// 4. It must have an input port named "scene" of type [`SceneGraph`] that will contain the scene graph as generated by the previous plugin.
///
/// TODO: add examples
///
/// In this example, `InitialCounterNode` is an initial plugin (no input),
/// while `IncrementCounterNode` is a subsequent plugin (has both "input" and "scene" inputs).
#[derive(Debug, Clone)]
pub struct ViewportPlugin<T: NodeFactory>(T, PluginPosition);

impl<T: NodeFactory> ViewportPlugin<T> {
    /// Creates a new [`ViewportPlugin`] from a 'node'.
    /// Refer to the [`ViewportPlugin`] documentation for more information on the requirements for a valid plugin.
    ///
    /// # Errors
    ///
    /// This function will return an error if the node does not meet the requirements as specified in [`ViewportPlugin`].
    ///
    /// See [`ViewportPluginValidationError`] for specific error types.
    pub fn new(node: T) -> Result<Self, ViewportPluginValidationError> {
        validate_plugin(&T::inputs(), &T::outputs()).map(|t| Self(node, t))
    }
}

/// A dynamic plugin for the [`ViewportPipeline`].
///
/// This is the dynamic version of a [`ViewportPlugin`].
/// It is recommended to use [`ViewportPlugin`] if the specific plugin type is known at compile time.
///
/// Refer to [`ViewportPlugin`] for more information on how to create a valid plugin.
/// TODO: add examples
#[derive(Debug, Clone)]
pub struct DynamicViewportPlugin(DynamicNode, PluginPosition);

impl DynamicViewportPlugin {
    /// Creates a new [`DynamicViewportPlugin`] from a [`DynamicNode`].
    /// Refer to the [`ViewportPlugin`] and [`DynamicViewportPlugin`] documentation for more information on the requirements for a valid plugin.
    ///
    /// # Errors
    ///
    /// This function will return an error if the node does not meet the requirements as specified in [`ViewportPlugin`].
    ///
    /// See [`ViewportPluginValidationError`] for specific error types.
    pub fn new(node: DynamicNode) -> Result<Self, ViewportPluginValidationError> {
        validate_plugin(node.inputs(), node.outputs()).map(|t| Self(node, t))
    }
}

fn validate_plugin(
    inputs: &[(&str, TypeId)],
    outputs: &[(&str, TypeId)],
) -> Result<PluginPosition, ViewportPluginValidationError> {
    let input_type = inputs
        .iter()
        .find_map(|n| if n.0 == "input" { Some(n.1) } else { None });
    let output_type = outputs
        .iter()
        .find_map(|n| if n.0 == "output" { Some(n.1) } else { None });
    let scene_input_type = inputs
        .iter()
        .find_map(|n| if n.0 == "scene" { Some(n.1) } else { None });
    let scene_output_type = outputs
        .iter()
        .find_map(|n| if n.0 == "scene" { Some(n.1) } else { None });

    if let Some(t) = scene_output_type {
        if t != TypeId::of::<SceneGraph>() {
            return Err(ViewportPluginValidationError::InvalidSceneOutputPort);
        }
    } else {
        return Err(ViewportPluginValidationError::InvalidSceneOutputPort);
    }

    if output_type.is_none() {
        return Err(ViewportPluginValidationError::MissingOutputPort);
    }

    match (input_type, scene_input_type) {
        (Some(_), None) | (None, Some(_)) => Err(ViewportPluginValidationError::InputPortMismatch),
        (Some(_), Some(scene)) => {
            if scene == TypeId::of::<SceneGraph>() {
                Ok(PluginPosition::Subsequent)
            } else {
                Err(ViewportPluginValidationError::InputPortMismatch)
            }
        }
        (None, None) => Ok(PluginPosition::Initial),
    }
}

impl ViewportPipeline {
    /// Adds a plugin to the viewport pipeline.
    ///
    /// This method appends the given plugin to the end of the pipeline. The plugin's position
    /// (Initial or Subsequent) determines how it's integrated:
    ///
    /// - `Initial`: Standalone plugin without inputs from previous plugins.
    /// - `Subsequent`: Plugin that receives inputs from the preceding plugin in the pipeline.
    ///
    /// If the specific plugin type is not known at compile time, use [`ViewportPipeline::add_dynamic_plugin`] with a
    /// [`DynamicViewportPlugin`] instead.
    ///
    /// # Parameters
    /// - `plugin`: The plugin to add to the pipeline
    ///
    /// # Returns
    ///
    /// Ok(()) if the plugin was successfully added to the pipeline
    ///
    /// # Errors
    ///
    /// Returns a `Err(PipelineAddError)` if the plugin could not be added to the pipeline.
    ///
    /// # Panics
    ///
    /// This function will panic if a duplicate node name is generated internally.
    /// This should never happen under normal circumstances as node names are generated uniquely.
    pub fn add_plugin<T: computegraph::NodeFactory + 'static>(
        &mut self,
        plugin: ViewportPlugin<T>,
    ) -> Result<(), PipelineAddError> {
        let this = &mut *self;
        let node = NodeHandle {
            node_name: format!("node-{}", this.nodes.len()),
        };

        let handle: NodeHandle = match this.graph.add_node(plugin.0, node.node_name.clone()) {
            Ok(node) => node.into(),
            Err(err) => match err {
                computegraph::AddError::DuplicateName(_) => {
                    panic!("Node names are only given by our selves")
                }
            },
        };

        match this.connect_plugin(handle, plugin.1) {
            Ok(v) => Ok(v),
            Err(err) => {
                // Clean up
                let _ = this.graph.remove_node(node);
                Err(err)
            }
        }
    }

    /// Adds a dynamic plugin to the viewport pipeline.
    ///
    /// If the specific plugin type is not known at compile time, use this method,
    /// otherwise use [`ViewportPipeline::add_plugin`] instead.
    ///
    /// See [`ViewportPipeline::add_plugin`] for more information on how plugins are integrated.
    ///
    /// # Parameters
    /// - `plugin`: The plugin to add to the pipeline
    ///
    /// # Returns
    ///
    /// Ok(()) if the plugin was successfully added to the pipeline
    ///
    /// # Errors
    ///
    /// Returns a `Err(PipelineAddError)` if the plugin could not be added to the pipeline.
    ///
    /// # Panics
    ///
    /// This function will panic if a duplicate node name is generated internally.
    /// This should never happen under normal circumstances as node names are generated uniquely.
    pub fn add_dynamic_plugin(
        &mut self,
        node: DynamicViewportPlugin,
    ) -> Result<(), PipelineAddError> {
        let this = &mut *self;
        let handle = NodeHandle {
            node_name: format!("node-{}", this.nodes.len()),
        };

        let handle: NodeHandle = match this.graph.add_node_dynamic(node.0, handle.node_name) {
            Ok(node) => node,
            Err(err) => match err {
                computegraph::AddError::DuplicateName(_) => {
                    panic!("this should not happen")
                }
            },
        };

        match this.connect_plugin(handle.clone(), node.1) {
            Ok(v) => Ok(v),
            Err(err) => {
                // Clean up
                let _ = this.graph.remove_node(handle);
                Err(err)
            }
        }
    }

    fn connect_plugin(
        &mut self,
        node: NodeHandle,
        node_type: PluginPosition,
    ) -> Result<(), PipelineAddError> {
        let input = node.clone().to_input_port("input");
        let output = node.clone().to_output_port("output");
        let scene_input = node.clone().to_input_port("scene");
        let scene_output = node.clone().to_output_port("scene");

        match (node_type, self.nodes.last()) {
            (PluginPosition::Initial, None) => {
                // No connecting needed, we are done!
                Ok(())
            }
            (PluginPosition::Initial, Some(_)) => {
                // This node will be added as if it is the first node
                Ok(())
            }
            (PluginPosition::Subsequent, Some(prev_node)) => {
                // We now need to connect the node with the previous node
                self.graph
                    .connect_untyped(prev_node.output.clone(), input)
                    .map_err(|e| match e {
                        computegraph::ConnectError::TypeMismatch { expected, found } => {
                            PipelineAddError::TypeMismatch {
                                input_type: expected,
                                output_type: found,
                            }
                        }
                        computegraph::ConnectError::InputPortAlreadyConnected { .. }
                        | computegraph::ConnectError::NodeNotFound(_) => {
                            panic!("We just added the node")
                        }
                        computegraph::ConnectError::InputPortNotFound(_)
                        | computegraph::ConnectError::OutputPortNotFound(_) => {
                            panic!("We checked that already")
                        }
                    })?;
                match self
                    .graph
                    .connect_untyped(prev_node.scene_output.clone().into(), scene_input)
                {
                    Ok(_) => (),
                    Err(e) => match e {
                        computegraph::ConnectError::InputPortAlreadyConnected { .. }
                        | computegraph::ConnectError::NodeNotFound(_) => {
                            panic!("We just added the node")
                        }
                        computegraph::ConnectError::TypeMismatch { .. }
                        | computegraph::ConnectError::InputPortNotFound(_)
                        | computegraph::ConnectError::OutputPortNotFound(_) => {
                            panic!("We checked that already")
                        }
                    },
                }
                Ok(())
            }
            (PluginPosition::Subsequent, None) => {
                Err(PipelineAddError::SubsequentPluginInEmptyPipeline)
            }
        }?;

        self.nodes.push(ViewportPluginNode {
            node,
            output,
            scene_output: scene_output.to_typed(),
        });

        Ok(())
    }

    /// Removes the most recently added plugin from the viewport pipeline.
    ///
    /// This method removes the last plugin added to the pipeline, effectively
    /// undoing the last [`ViewportPipeline::add_plugin`] or [`ViewportPipeline::add_dynamic_plugin`] operation.
    ///
    /// # Panics
    ///
    /// This function will panic if the last added node is not found in the graph.
    /// This should never happen under normal circumstances as the node was previously added to the graph.
    pub fn remove_last_plugin(&mut self) {
        if let Some(node) = self.nodes.pop() {
            match self.graph.remove_node(node.node) {
                Ok(()) => {}
                Err(computegraph::RemoveNodeError::NodeNotFound(_)) => {
                    panic!("We added it, so it should exist")
                }
            }
        }
    }

    /// Computes the final [`SceneGraph`] of the viewport pipeline.
    ///
    /// This function traverses the pipeline and computes the `SceneGraph` output
    /// from the last plugin in the chain.
    ///
    /// # Parameters
    /// - `project_view`: This [`ProjectView`] will be passed to all nodes of the [`ViewportPlugin`]s and the [`SceneGraph`].
    ///
    /// # Returns
    ///
    /// The final [`SceneGraph`] for rendering inside the viewport.
    ///
    /// # Errors
    ///
    /// - `Err(ExecuteError::EmptyPipeline)` if the pipeline is empty.
    /// - `Err(ExecuteError::ComputeError)` if there's an error during computation
    ///     of the added [`ViewportPlugin`]s.
    pub fn compute_scene(&self, project_view: ProjectView) -> Result<SceneGraph, ExecuteError> {
        // TODO: pass ProjectView to ViewportPluginNodes
        let last_node = self.nodes.last().ok_or(ExecuteError::EmptyPipeline)?;
        let mut ctx = ComputationContext::default();
        ctx.set_fallback(project_view);
        let scene = self
            .graph
            .compute_with_context(last_node.scene_output.clone(), &ctx)?;

        Ok(scene)
    }

    pub(crate) fn update(
        &self,
        state: &mut ViewportPipelineState,
        events: ViewportEvent,
        project_view: ProjectView,
    ) -> Result<(), ExecuteError> {
        let scene = self.compute_scene(project_view.clone())?;

        let s = state.state.take();
        let s = match s {
            Some(s) => s,
            None => scene.graph.compute_untyped(scene.init_state).unwrap(),
        };

        let mut ctx = ComputationContext::default();
        ctx.set_override_untyped(scene.update_state_in.clone(), s);
        ctx.set_override(scene.update_event_in, events);
        ctx.set_fallback(project_view);

        let result = scene
            .graph
            .compute_untyped_with_context(scene.update_state_out, &ctx)
            .map_err(ExecuteError::ComputeError)?;
        state.state = Some(result);
        Ok(())
    }

    pub(crate) fn compute_primitive(
        &self,
        state: &mut ViewportPipelineState,
        project_view: ProjectView,
    ) -> Result<Box<dyn iced::widget::shader::Primitive>, ExecuteError> {
        let scene = self.compute_scene(project_view.clone())?;

        let s = state.state.take();
        let s = match s {
            Some(s) => s,
            None => scene.graph.compute_untyped(scene.init_state).unwrap(),
        };

        let mut ctx = ComputationContext::default();
        ctx.set_override_untyped(scene.render_state_in.clone(), s);
        ctx.set_fallback(project_view);

        let result = scene
            .graph
            .compute_with_context(scene.render_primitive_out, &ctx)
            .map_err(ExecuteError::ComputeError);
        let a = ctx.remove_override_untyped(&scene.render_state_in);
        debug_assert!(a.is_some());
        state.state = a;
        result
    }

    /// Returns the number of plugins in the viewport pipeline.
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
