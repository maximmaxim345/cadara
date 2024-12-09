#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cognitive_complexity)]

use modeling_module::ModelingModule;
use project::data::transaction::TransactionArgs;
use workspace::Workspace;

struct App {
    viewport: viewport::Viewport,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
enum Message {}

impl App {
    fn new() -> Self {
        let project = project::Project::new("project".to_string()).create_view();
        let doc = project.create_document();
        let doc = project.open_document(doc).unwrap();
        let data_uuid = doc.create_data::<ModelingModule>();
        let mut data = doc.open_data_by_uuid::<ModelingModule>(data_uuid).unwrap();

        data.apply(TransactionArgs::Persistent(
            modeling_module::persistent_data::ModelingTransaction::Create(
                modeling_module::persistent_data::Create {
                    before: None,
                    operation: modeling_module::operation::ModelingOperation::Sketch(
                        modeling_module::operation::sketch::Sketch,
                    ),
                },
            ),
        ))
        .expect("apply transaction");
        let mut viewport = viewport::Viewport::new(project);
        let workspace = modeling_workspace::ModelingWorkspace { data_uuid };
        // TODO: this should dynamically select the first fitting plugin
        let plugin = workspace.viewport_plugins()[0].clone();
        viewport.pipeline.add_dynamic_plugin(plugin).unwrap();
        Self { viewport }
    }

    #[expect(clippy::unused_self)] // required by `iced::application`
    fn update(&mut self, _message: Message) {}

    fn view(&self) -> iced::Element<'_, Message> {
        let viewport_shader = iced::widget::shader(&self.viewport)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill);

        iced::widget::column!(iced::widget::text("Viewport:"), viewport_shader).into()
    }
}

/// Initializes and runs `CADara`.
///
/// Sets up logging and runs the application. This function must only be called once.
///
/// # Panics
///
/// Panics if `iced::application::run` fails.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn run_cadara() {
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init().expect("Initialize logger");
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        wasm_libc::init();
    }

    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt::init();

    iced::application("CADara", App::update, App::view)
        .run()
        .unwrap();
}
