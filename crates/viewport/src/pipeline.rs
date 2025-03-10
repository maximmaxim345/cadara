use crate::ViewportEvent;
use computegraph::{
    node, ComputationCache, ComputationContext, ComputationOptions, ComputeGraph, DynamicNode,
    InputPort, InputPortUntyped, NodeFactory, NodeHandle, OutputPort, OutputPortUntyped,
};
use project::{CacheValidator, ProjectView, TrackedProjectView};
use std::{
    any::TypeId,
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

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
    node: NodeHandle,
    output: OutputPortUntyped,
    scene_output: OutputPort<SceneGraph>,
    cache_node: NodeHandle,
    scene_output_cached: OutputPort<SceneGraph>,
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

/// Node to allow caching of the last node.
///
/// This node allows the last node of the [`ViewportPipeline`] to be cached, since otherwise
/// the output of that node would be consumed with `compute()`.
#[derive(PartialEq, Debug, Clone)]
struct CloneSceneGraphNode();

#[node(CloneSceneGraphNode -> !)]
fn run(&self, scene: &SceneGraph) -> SceneGraph {
    scene.clone()
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

/// Project aware cache for the Viewport.
///
/// This is a extension of [`ComputationCache`], that allows changes to [`ProjectView`]s, if
/// a node did only access parts of that, that did not change.
#[derive(Default, Debug)]
pub struct ViewportCache {
    prev_project_view: Option<Arc<ProjectView>>,
    cache: Mutex<ComputationCache>,
    version: u64,
    metadata: Option<CacheMetadata>,
}

#[derive(Default, Debug)]
pub struct ViewportPipelineState {
    state: Option<Box<dyn computegraph::SendSyncAny>>,
    prev_project_view: Option<Arc<ProjectView>>,
    scenegraph_cache: ViewportCache,
    pipeline_cache: ViewportCache,
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

/// State of the whole project, wrapper around [`ProjectView`] with caching support.
///
/// While it implements clone, do not use a cloned [`ProjectView`]. This will panic.
pub enum ProjectState {
    Valid(TrackedProjectView, u64),
    Cloned(u64),
}

impl ProjectState {
    const fn new(pvo: TrackedProjectView, version: u64) -> Self {
        Self::Valid(pvo, version)
    }
}

impl Clone for ProjectState {
    fn clone(&self) -> Self {
        match self {
            Self::Cloned(version) | Self::Valid(_, version) => Self::Cloned(*version),
        }
    }
}

impl PartialEq for ProjectState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Valid(_, self_version) | Self::Cloned(self_version),
                Self::Valid(_, other_version) | Self::Cloned(other_version),
            ) => self_version == other_version,
        }
    }
}

impl std::ops::Deref for ProjectState {
    type Target = TrackedProjectView;

    fn deref(&self) -> &Self::Target {
        if let Self::Valid(view, _) = self {
            view
        } else {
            // TODO: this could just be a compile time check if we don't implement Clone
            panic!("A cloned ProjectState must never be used for anything else but comparisons")
        }
    }
}

// helper functions for caching

type AccessRecorders = Arc<Mutex<BTreeMap<String, (project::AccessRecorder, u64)>>>;
type CacheMetadata = BTreeMap<String, (CacheValidator, u64)>;

fn update_cache_versions(
    cache_metadata: &mut CacheMetadata,
    project_view: &Arc<ProjectView>,
    project_view_version: u64,
    prev_project_view: &Arc<ProjectView>,
) {
    for (cache_validator, cached_version) in cache_metadata.values_mut() {
        // Update each nodes version if its cache is no longer valid.
        let cache_still_valid = cache_validator.is_cache_valid(prev_project_view, project_view);
        if !cache_still_valid {
            // Force recomputation by setting to the new version.
            *cached_version = project_view_version;
        }
    }
}

fn initialize_access_tracking(
    ctx: &mut ComputationContext,
    project_view: Arc<ProjectView>,
    project_view_version: u64,
    cache_metadata: Arc<CacheMetadata>,
) -> AccessRecorders {
    // create a shared structure to record access from each node, to later more granualy
    // detect cache validity
    let access_recorders = Arc::new(Mutex::new(BTreeMap::new()));

    // fallback generator is called for every node to provide a ProjectState, even those
    // who will be cached (so not be executed)
    let access_recorders_clone = access_recorders.clone();
    ctx.set_fallback_generator(move |node_name| {
        let (tracked_view, recorder) = TrackedProjectView::new(project_view.clone());
        access_recorders_clone
            .lock()
            .unwrap()
            .insert(node_name.to_string(), (recorder, project_view_version));
        let version = cache_metadata
            .get(node_name)
            .map_or(project_view_version, |e| e.1);
        ProjectState::new(tracked_view, version)
    });
    access_recorders
}

fn update_cache_metadata(metadata: &mut CacheMetadata, access_recorders: &AccessRecorders) {
    let access_recorders = std::mem::take(&mut *(access_recorders.lock().unwrap()));
    // Freeze and collect all access recorder data.
    let mut access_data: BTreeMap<_, _> = access_recorders
        .into_iter()
        .map(|(node_name, (recorder, version))| (node_name, (recorder.freeze(), version)))
        .collect();

    // Update metadata for nodes that already existed.
    for (node_name, (existing_validator, existing_version)) in metadata.iter_mut() {
        if let Some((_, (new_validator, new_version))) = access_data.remove_entry(node_name) {
            if new_validator.was_accessed() {
                *existing_validator = new_validator;
                *existing_version = new_version;
            }
        }
    }
    // Add any new nodes that were not in the previous metadata.
    for (node_name, data) in access_data {
        metadata.insert(node_name, data);
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
    pub fn add_plugin<T: NodeFactory + 'static>(
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

        let handle: NodeHandle = this
            .graph
            .add_node_dynamic(node.0, handle.node_name)
            .unwrap_or_else(|err| match err {
                computegraph::AddError::DuplicateName(_) => {
                    panic!("this should not happen")
                }
            });

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

        let scene_output = scene_output.to_typed();

        let cache_node = self
            .graph
            .add_node(CloneSceneGraphNode(), node.node_name.clone() + "_cache")
            .expect("some text");
        let scene_output_cached = cache_node.output();

        self.graph
            .connect(scene_output.clone(), cache_node.input_scene())
            .expect("test");

        self.nodes.push(ViewportPluginNode {
            node,
            output,
            scene_output,
            cache_node: cache_node.into(),
            scene_output_cached,
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
            self.graph
                .remove_node(node.node)
                .expect("We added it, so it should exist");
            self.graph
                .remove_node(node.cache_node)
                .expect("We added it, so it should exist");
        }
    }

    /// Computes the final [`SceneGraph`] of the viewport pipeline.
    ///
    /// This function traverses the pipeline and computes the `SceneGraph` output
    /// from the last plugin in the chain.
    ///
    /// # Parameters
    /// - `project_view`: This [`ProjectView`] will be passed to all nodes of the [`ViewportPlugin`]s and the [`SceneGraph`].
    ///   (accessible through a [`ProjectState`] in nodes)
    /// - `cache`: A [`ViewportCache`] that will be used to cache the viewport pipeline.
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
    #[expect(clippy::missing_panics_doc, reason = "only panics with poisoned lock")]
    pub fn compute_scene(
        &self,
        project_view: Arc<ProjectView>,
        project_view_version: u64,
        cache: &mut ViewportCache,
    ) -> Result<SceneGraph, ExecuteError> {
        let mut cache_metadata = cache.metadata.take().unwrap_or_default();
        if let Some(prev_project_view) = &cache.prev_project_view {
            if cache.version != project_view_version {
                update_cache_versions(
                    &mut cache_metadata,
                    &project_view,
                    project_view_version,
                    prev_project_view,
                );
            }
        }
        cache.version = project_view_version;
        cache.prev_project_view = Some(project_view.clone());

        let last_node = self.nodes.last().ok_or(ExecuteError::EmptyPipeline)?;
        let mut ctx = ComputationContext::default();

        let cache_metadata = Arc::new(cache_metadata);
        let access_recorders = initialize_access_tracking(
            &mut ctx,
            project_view,
            project_view_version,
            cache_metadata.clone(),
        );

        let scene = self.graph.compute_with(
            last_node.scene_output_cached.clone(),
            &ComputationOptions {
                context: Some(&ctx),
            },
            Some(&mut cache.cache.lock().unwrap()),
        )?;

        drop(ctx);

        let mut cache_metadata =
            Arc::try_unwrap(cache_metadata).expect("we dropped ctx, which had the only other copy");

        update_cache_metadata(&mut cache_metadata, &access_recorders);

        cache.metadata = Some(cache_metadata);

        Ok(scene)
    }

    pub(crate) fn update(
        &self,
        state: &mut ViewportPipelineState,
        events: ViewportEvent,
        project_view: Arc<ProjectView>,
        project_view_version: u64,
    ) -> Result<(), ExecuteError> {
        let mut cache_metadata = state.scenegraph_cache.metadata.take().unwrap_or_default();
        if let Some(prev_project_view) = &state.prev_project_view {
            if state.scenegraph_cache.version != project_view_version {
                update_cache_versions(
                    &mut cache_metadata,
                    &project_view,
                    project_view_version,
                    prev_project_view,
                );
            }
        }
        state.scenegraph_cache.version = project_view_version;
        state.prev_project_view = Some(project_view.clone());

        let scene = self.compute_scene(
            project_view.clone(),
            project_view_version,
            &mut state.pipeline_cache,
        )?;

        let s = state
            .state
            .take()
            .map_or_else(|| scene.graph.compute_untyped(scene.init_state), Ok)?;

        // setup the computation context
        let mut ctx = ComputationContext::default();
        ctx.set_override_untyped(scene.update_state_in.clone(), s);
        ctx.set_override(scene.update_event_in, events);

        let cache_metadata = Arc::new(cache_metadata);
        let access_recorders = initialize_access_tracking(
            &mut ctx,
            project_view,
            project_view_version,
            cache_metadata.clone(),
        );

        let result = scene
            .graph
            .compute_untyped_with(
                scene.update_state_out,
                &ComputationOptions {
                    context: Some(&ctx),
                },
                Some(&mut state.scenegraph_cache.cache.lock().unwrap()),
            )
            .map_err(ExecuteError::ComputeError)?;

        drop(ctx);

        let mut cache_metadata =
            Arc::try_unwrap(cache_metadata).expect("we dropped ctx, which had the only other copy");

        update_cache_metadata(&mut cache_metadata, &access_recorders);

        state.scenegraph_cache.metadata = Some(cache_metadata);

        state.state = Some(result);
        Ok(())
    }

    pub(crate) fn compute_primitive(
        &self,
        state: &mut ViewportPipelineState,
        project_view: Arc<ProjectView>,
        project_view_version: u64,
    ) -> Result<Box<dyn iced::widget::shader::Primitive>, ExecuteError> {
        let mut cache_metadata = state.scenegraph_cache.metadata.take().unwrap_or_default();
        if let Some(prev_project_view) = &state.prev_project_view {
            if state.scenegraph_cache.version != project_view_version {
                update_cache_versions(
                    &mut cache_metadata,
                    &project_view,
                    project_view_version,
                    prev_project_view,
                );
            }
        }
        state.scenegraph_cache.version = project_view_version;
        state.prev_project_view = Some(project_view.clone());

        let scene = self.compute_scene(
            project_view.clone(),
            project_view_version,
            &mut state.pipeline_cache,
        )?;

        let s = state
            .state
            .take()
            .map_or_else(|| scene.graph.compute_untyped(scene.init_state), Ok)?;

        // setup the computation context
        let mut ctx = ComputationContext::default();
        ctx.set_override_untyped(scene.render_state_in.clone(), s);

        let cache_metadata = Arc::new(cache_metadata);
        let access_recorders = initialize_access_tracking(
            &mut ctx,
            project_view,
            project_view_version,
            cache_metadata.clone(),
        );

        let result = scene
            .graph
            .compute_with(
                scene.render_primitive_out,
                &ComputationOptions {
                    context: Some(&ctx),
                },
                Some(&mut state.scenegraph_cache.cache.lock().unwrap()),
            )
            .map_err(ExecuteError::ComputeError);

        let new_state = ctx.remove_override_untyped(&scene.render_state_in);
        debug_assert!(new_state.is_some());
        state.state = new_state;

        drop(ctx);

        let mut cache_metadata =
            Arc::try_unwrap(cache_metadata).expect("we dropped ctx, which had the only other copy");

        update_cache_metadata(&mut cache_metadata, &access_recorders);

        state.scenegraph_cache.metadata = Some(cache_metadata);

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
