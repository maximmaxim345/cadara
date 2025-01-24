#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cognitive_complexity)]
// TODO: look if docs need to be changed

//! # Viewport
//!
//! The viewport is the central UI component for rendering and interacting with documents in `CADara`.
//!
//! ## Overview
//!
//! The viewport uses a [`ViewportPipeline`] to manage a series of plugins (each defined by a workspace)
//! that incrementally build and modify a scene graph. This allows for deeply integrated and flexible workspaces.
//!
//! ## Pipeline
//!
//! The [`ViewportPipeline`] is the core structure that manages plugins. It allows adding plugins
//! sequentially, where each plugin can modify or extend the scene graph created by previous plugins.
//!
//! ## Plugins
//!
//! Plugins come in two types:
//! - [`ViewportPlugin`]: For use when the plugin type is known at compile time.
//! - [`DynamicViewportPlugin`]: For use when the plugin type is not known at compile time.
//!
//! Plugins can be either:
//! - `Initial`: The first plugin in the pipeline, responsible for initializing the scene graph.
//! - `Subsequent`: Added after other plugins, receiving input from the previous plugin.
//!
//! In the case that a `Initial` plugin is added after other plugins, the scene graph of the previous plugins
//! will be ignored, allowing for a fresh start for vastly different workspaces.
//!
//! ### Extensions
//!
//! Nodes (or subgraphs) in the scene graph can be annotated with extensions. Extensions are run by plugins (through a helper library)
//! TODO: I have no idea yet about how extensions should function (if at all)
//!
//! ### Render Nodes
//!
//! Render nodes are responsible for rendering the scene every frame and should be as lightweight as possible.
//! A node can be marked as a render node by having a input channel of type `render_context`. TODO: complete
//! The viewport will automatically the `render_context` input port with the rendering context. TODO: update when it's clear how this works.
//!
//! ## Execution
//!
//! The [`ViewportPipeline`] executes plugins in order, with each subsequent plugin receiving
//! the output and graph from the previous plugin. The final [`computegraph::ComputeGraph`] returned by the last plugin
//! is than executed by the viewport to render to the screen.

use iced::widget::shader;
use project::ProjectView;
use shader::wgpu;
use std::sync::{Arc, Mutex};

mod pipeline;

#[doc(inline)]
pub use pipeline::{
    DynamicViewportPlugin, ExecuteError, PipelineAddError, ProjectState, RenderNodePorts,
    SceneGraph, SceneGraphBuilder, UpdateNodePorts, ViewportPipeline, ViewportPlugin,
    ViewportPluginValidationError,
};

#[derive(Debug)]
pub struct ViewportEvent {
    pub event: shader::Event,
    pub bounds: iced::Rectangle,
    pub cursor: iced::advanced::mouse::Cursor,
}

#[derive(Clone, Debug, Default)]
pub struct ViewportState {
    state: Arc<Mutex<pipeline::ViewportPipelineState>>,
}

#[derive(Clone)]
pub struct Viewport {
    pub pipeline: ViewportPipeline,
    pub project_view: Arc<ProjectView>,
    pub prev_project_view: Option<Arc<ProjectView>>,
    pub project_view_version: u64,
}

impl Viewport {
    #[must_use]
    pub fn new(project_view: Arc<ProjectView>) -> Self {
        Self {
            pipeline: ViewportPipeline::default(),
            project_view,
            prev_project_view: None,
            project_view_version: 1,
        }
    }

    pub fn update(&mut self, project_view: Arc<ProjectView>) {
        self.prev_project_view = Some(std::mem::replace(&mut self.project_view, project_view));
        self.project_view_version += 1;
    }
}

impl<Message> shader::Program<Message> for Viewport {
    type State = ViewportState;

    type Primitive = ShaderPrimitive;

    fn update(
        &self,
        state: &mut Self::State,
        event: shader::Event,
        bounds: iced::Rectangle,
        cursor: iced::advanced::mouse::Cursor,
        _shell: &mut iced::advanced::Shell<'_, Message>,
    ) -> (
        iced::advanced::graphics::core::event::Status,
        Option<Message>,
    ) {
        let event = ViewportEvent {
            event,
            bounds,
            cursor,
        };
        self.pipeline
            .update(
                &mut state.state.lock().unwrap(),
                event,
                self.project_view.clone(),
                self.project_view_version,
            )
            .unwrap();
        (iced::advanced::graphics::core::event::Status::Ignored, None)
    }

    fn draw(
        &self,
        state: &Self::State,
        _cursor: iced::advanced::mouse::Cursor,
        _bounds: iced::Rectangle,
    ) -> Self::Primitive {
        ShaderPrimitive {
            pipeline: self.pipeline.clone(),
            state: state.clone(),
            project_view: self.project_view.clone(),
            prev_project_view: self.prev_project_view.clone(),
            project_view_version: self.project_view_version,
        }
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        _bounds: iced::Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> iced::advanced::mouse::Interaction {
        iced::advanced::mouse::Interaction::default()
    }
}

#[derive(Debug)]
pub struct ShaderPrimitive {
    pub pipeline: ViewportPipeline,
    pub state: ViewportState,
    pub project_view: Arc<ProjectView>,
    pub prev_project_view: Option<Arc<ProjectView>>,
    pub project_view_version: u64,
}

impl shader::Primitive for ShaderPrimitive {
    fn prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        storage: &mut shader::Storage,
        bounds: &iced::Rectangle,
        viewport: &shader::Viewport,
    ) {
        let mut state = self.state.state.lock().unwrap();
        if state.known_version != self.project_view_version {
            let lr = &state.last_run.clone(); // TODO: no clone!
            for (name, (c, _v)) in lr {
                state
                    .sceengraph_cached_project_versions
                    .entry(name.clone())
                    .and_modify(|e| {
                        let valid = self
                            .prev_project_view
                            .as_ref()
                            .is_some_and(|pv| c.is_cache_valid(pv, &self.project_view));
                        if !valid {
                            *e = self.project_view_version;
                        }
                    })
                    .or_insert_with(|| self.project_view_version);
            }
            state.known_version = self.project_view_version;
        }
        let mut llr = std::mem::take(&mut state.last_run);
        let (a, lr) = self
            .pipeline
            .compute_primitive(
                &mut state,
                self.project_view.clone(),
                self.project_view_version,
            )
            .unwrap();
        // let lr = std::mem::take(&mut state.last_run);

        // let mut llr2 = Vec::new();
        for (name, (c, v)) in &mut llr {
            if let Some(a) = lr.get(name) {
                // TODO:  continue here, accessed nodes should be moved from lr to llr
                if a.0.was_accessed() {
                    // llr2.push((name.clone(), a.clone()));
                    *c = a.0.clone();
                    *v = a.1;
                }
            }
        }
        for a in lr {
            if let std::collections::btree_map::Entry::Vacant(e) = llr.entry(a.0) {
                e.insert(a.1);
            }
        }
        // for a in llr2.into_iter() {
        //     llr.insert(a.0, a.1);
        // }
        state.last_run = llr;
        drop(state);
        a.prepare(device, queue, format, storage, bounds, viewport);
    }

    fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        storage: &shader::Storage,
        target: &wgpu::TextureView,
        clip_bounds: &iced::Rectangle<u32>,
    ) {
        let a = self
            .pipeline
            .compute_primitive(
                &mut self.state.state.lock().unwrap(),
                self.project_view.clone(),
                self.project_view_version,
            )
            .unwrap();
        a.0.render(encoder, storage, target, clip_bounds);
    }
}
