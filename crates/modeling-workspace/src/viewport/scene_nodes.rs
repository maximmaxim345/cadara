use computegraph::node;
use iced::widget::shader;
use viewport::ViewportEvent;

use super::{
    rendering::{MeshData, RenderPrimitive, Vertex},
    state::ViewportState,
};

#[derive(Clone, Debug)]
pub struct ModelNode {
    pub data_uuid: project::data::DataUuid,
}

#[node(ModelNode)]
fn run(&self, project: &project::ProjectSession) -> occara::shape::Shape {
    let data_session: project::data::DataSession<modeling_module::ModelingModule> =
        project.open_data(self.data_uuid).unwrap();

    data_session.snapshot().persistent.shape()
}

#[derive(Clone, Debug)]
pub struct MeshingNode {}

#[node(MeshingNode)]
fn run(&self, shape: &occara::shape::Shape) -> MeshData {
    let mesh = shape.mesh();
    let vertices = mesh
        .vertices()
        .iter()
        .map(|p| Vertex {
            pos: glam::Vec3::new(p.x() as f32, p.y() as f32, p.z() as f32),
        })
        .collect();
    let indices = mesh.indices().iter().map(|i| *i as u32).collect();

    MeshData {
        vertices,
        indices,
        id: uuid::Uuid::new_v4(),
    }
}

#[derive(Clone, Debug)]
pub struct RenderNode {}

#[node(RenderNode)]
fn run(&self, state: &ViewportState, mesh: &MeshData) -> Box<dyn shader::Primitive> {
    // TODO: remove cloning to reduce overhead once computegraph allows that
    Box::new(RenderPrimitive {
        state: (*state).clone(),
        mesh: (*mesh).clone(),
    })
}

#[derive(Clone, Debug)]
pub struct UpdateStateNode {
    pub data_uuid: project::data::DataUuid,
}

#[node(UpdateStateNode)]
fn run(
    &self,
    event: &ViewportEvent,
    state: &ViewportState,
    project: &project::ProjectSession,
) -> ViewportState {
    let mut state = (*state).clone();
    let mut _data_session: project::data::DataSession<modeling_module::ModelingModule> =
        project.open_data(self.data_uuid).unwrap();
    if let shader::Event::Mouse(m) = event.event {
        match m {
            iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) => {
                state.l_button_pressed = true;
            }
            iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left) => {
                state.l_button_pressed = false;
            }
            iced::mouse::Event::ButtonPressed(iced::mouse::Button::Right) => {
                state.r_button_pressed = true;
            }
            iced::mouse::Event::ButtonReleased(iced::mouse::Button::Right) => {
                state.r_button_pressed = false;
            }
            iced::mouse::Event::CursorMoved { position } => {
                if state.l_button_pressed {
                    let c = (position - state.cursor_position) * -0.01;
                    state.camera.pan(c.x, c.y);
                } else if state.r_button_pressed {
                    let c = position - state.cursor_position;
                    state.camera.rotate(-c.x * 0.003, c.y * 0.003);
                }
                state.cursor_position = position;
            }
            iced::mouse::Event::WheelScrolled { delta } => match delta {
                iced::mouse::ScrollDelta::Lines { x: _, y } => state.camera.move_forward(y * 0.08),
                iced::mouse::ScrollDelta::Pixels { x: _, y } => state.camera.move_forward(y * 0.01),
            },
            _ => {}
        }
    }
    state
}

#[derive(Clone, Debug)]
pub struct InitStateNode {}

#[node(InitStateNode)]
fn run(&self) -> ViewportState {
    ViewportState::default()
}
