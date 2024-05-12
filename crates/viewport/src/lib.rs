#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cognitive_complexity)]

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
use project::ProjectSession;
use std::sync::{Arc, Mutex};

mod pipeline;

#[doc(inline)]
pub use pipeline::{
    DynamicViewportPlugin, ExecuteError, PipelineAddError, RenderNodePorts, SceneGraph,
    SceneGraphBuilder, UpdateNodePorts, ViewportPipeline, ViewportPlugin,
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
    pub project_session: ProjectSession,
}

impl Viewport {
    #[must_use]
    pub fn new(project_session: ProjectSession) -> Self {
        Self {
            pipeline: ViewportPipeline::default(),
            project_session,
        }
    }
}

impl<Message> shader::Program<Message> for Viewport {
    type State = ViewportState;

    type Primitive = ShaderPrimitive;

    fn draw(
        &self,
        state: &Self::State,
        _cursor: iced::advanced::mouse::Cursor,
        _bounds: iced::Rectangle,
    ) -> Self::Primitive {
        ShaderPrimitive {
            pipeline: self.pipeline.clone(),
            state: state.clone(),
            project_session: self.project_session.clone(),
        }
    }

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
                self.project_session.clone(),
            )
            .unwrap();
        (iced::advanced::graphics::core::event::Status::Ignored, None)
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
    pub project_session: ProjectSession,
}

impl shader::Primitive for ShaderPrimitive {
    fn prepare(
        &self,
        format: wgpu::TextureFormat,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: iced::Rectangle,
        target_size: iced::Size<u32>,
        scale_factor: f32,
        storage: &mut shader::Storage,
    ) {
        let a = self
            .pipeline
            .compute_primitive(
                &mut self.state.state.lock().unwrap(),
                self.project_session.clone(),
            )
            .unwrap();
        a.prepare(
            format,
            device,
            queue,
            bounds,
            target_size,
            scale_factor,
            storage,
        );
    }

    fn render(
        &self,
        storage: &shader::Storage,
        target: &wgpu::TextureView,
        target_size: iced::Size<u32>,
        viewport: iced::Rectangle<u32>,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let a = self
            .pipeline
            .compute_primitive(
                &mut self.state.state.lock().unwrap(),
                self.project_session.clone(),
            )
            .unwrap();
        a.render(storage, target, target_size, viewport, encoder);
    }
}