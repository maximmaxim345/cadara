#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::cast_precision_loss)]

mod demo_workspace;

use iced::Sandbox;
use workspace::Workspace;

struct App {
    viewport: viewport::Viewport,
}

impl iced::Sandbox for App {
    type Message = ();

    fn new() -> Self {
        if false {
            let mut viewport = viewport::Viewport::default();
            let workspace = demo_workspace::DemoWorkspace::default();
            // TODO: this should dynamically select the first fitting plugin
            let plugin = workspace.viewport_plugins()[0].clone();
            viewport.pipeline.add_dynamic_plugin(plugin).unwrap();
            Self { viewport }
        } else {
            use modeling_module::ModelingModule;
            use project::data::transaction::TransactionArgs;
            use utils::Transaction;

            let project = project::Project::new("project".to_string()).create_session();
            let doc = project.create_document();
            let doc = project.open_document(doc).unwrap();
            let data = doc.create_data::<ModelingModule>();
            let mut data = doc.open_data_by_uuid::<ModelingModule>(data).unwrap();

            data.apply(TransactionArgs::Persistent(()))
                .expect("apply transaction");

            // shared peristant: steps to build the part - CRDT
            // shared tempoary: current tasks performed by each user - CRDT/STRUCT
            // private peristant: visibility options of subparts - CRDT?
            // private tempoary: position and rotation of camera, selected part - STRUCT
            //
            // In sketch sub workspace:
            // shared peristant: geometry and constraints of the sketch - CRDT (of document)
            // shared tempoary: exact cursor position of the user, selected?
            //      - STRUCT? - visible only in that workspace, (of that document)
            // private peristant: none
            // private tempoary: selected geometry of the sketch - STRUCT
            //
            // This can be achieved with:
            // - A session is no longer exclusively bound to a module
            // - Session can have multiple modules (with each the 4 segments) (even multiple of the same type, if needed)
            // - A Workspace can advertise itself to be able to view a module (but does not have to like with sketch)
            // - Workspace has an update() and render() method
            // - update(state) -> (state, message) - message can modify modules, state can only me received by render(state)
            // - project crate does not handle transforming data, the user is responsible for that (caching is still global to reduce overhead)
            // - expand Session to be a view of the whole project instead of the document
            let mut viewport = viewport::Viewport::default();
            let workspace = modeling_workspace::ModelingWorkspace::default();
            // TODO: this should dynamically select the first fitting plugin
            let plugin = workspace.viewport_plugins()[0].clone();
            viewport.pipeline.add_dynamic_plugin(plugin).unwrap();
            Self { viewport }
        }
    }

    fn title(&self) -> String {
        "CADara".to_string()
    }

    fn update(&mut self, _message: Self::Message) {}

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let viewport_shader = iced::widget::shader(&self.viewport)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill);

        iced::widget::column!(iced::widget::text("Viewport:"), viewport_shader).into()
    }
}

fn main() -> iced::Result {
    App::run(iced::Settings::default())
}
