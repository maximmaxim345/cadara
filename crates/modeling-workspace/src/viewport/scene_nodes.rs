use computegraph::node;
use iced::widget::shader;
use viewport::ViewportEvent;

use super::{rendering::RenderPrimitive, state::ViewportState};

#[derive(Clone, Debug)]
pub struct RenderNode {
    pub data_uuid: project::data::DataUuid,
}

#[node(RenderNode)]
fn run(
    &self,
    state: &ViewportState,
    project: &project::ProjectSession,
) -> Box<dyn shader::Primitive> {
    let data_session: project::data::DataSession<modeling_module::ModelingModule> =
        project.open_data(self.data_uuid).unwrap();
    let shape = data_session.snapshot().persistent.shape();
    let mesh = shape.mesh();
    Box::new(RenderPrimitive {
        state: (*state).clone(),
        mesh,
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
