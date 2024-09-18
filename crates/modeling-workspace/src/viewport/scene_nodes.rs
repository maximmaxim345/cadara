use computegraph::node;
use iced::widget::shader;
use project::data::transaction::TransactionArgs;
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
    use utils::Transaction;

    let mut state = (*state).clone();
    let mut data_session: project::data::DataSession<modeling_module::ModelingModule> =
        project.open_data(self.data_uuid).unwrap();
    if let shader::Event::Mouse(m) = event.event {
        match m {
            iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) => {
                data_session.apply(TransactionArgs::Shared(())).unwrap();
                let persistent = data_session.snapshot().persistent;
                let s = persistent.shape();
                println!("mesh len: {:?}", s.mesh().vertices().len());
                println!("persistent: {persistent:?}");
                state.l_button_pressed = true;
            }
            iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left) => {
                state.l_button_pressed = false;
            }
            iced::mouse::Event::CursorMoved { position } => {
                if state.l_button_pressed {
                    let c = position - state.cursor_position;
                    state.camera_offset = state.camera_offset + c;
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
