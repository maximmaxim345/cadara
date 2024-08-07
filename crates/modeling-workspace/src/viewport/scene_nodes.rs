use computegraph::node;
use iced::widget::shader;
use viewport::ViewportEvent;

use super::{rendering::RenderPrimitive, state::ViewportState};

#[derive(Clone, Debug)]
pub struct RenderNode {}

#[node(RenderNode)]
fn run(
    &self,
    state: &ViewportState,
    _project: &project::ProjectSession,
) -> Box<dyn shader::Primitive> {
    Box::new(RenderPrimitive {
        state: (*state).clone(),
    })
}

#[derive(Clone, Debug)]
pub struct UpdateStateNode {}

#[node(UpdateStateNode)]
fn run(
    &self,
    event: &ViewportEvent,
    state: &ViewportState,
    _project: &project::ProjectSession,
) -> ViewportState {
    let mut state = (*state).clone();
    if let shader::Event::Mouse(m) = event.event {
        match m {
            iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) => {
                state.l_button_pressed = true;
            }
            iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left) => {
                state.l_button_pressed = false;
            }
            iced::mouse::Event::CursorMoved { position } => {
                if state.l_button_pressed {
                    let c = position - state.cursor_position;
                    state.camera_pos += glam::Vec2 { x: c.x, y: c.y };
                }
                state.cursor_position = position;
            }
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
