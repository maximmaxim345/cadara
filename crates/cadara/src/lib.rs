#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cognitive_complexity)]

use iced::{time, Subscription};
use modeling_module::ModelingModule;
use std::sync::Arc;
use workspace::Workspace;

struct App {
    viewport: viewport::Viewport,
    project: project::Project,
    project_view: Arc<project::ProjectView>,
    data_uuid: project::DataId,
    reg: project::ModuleRegistry,
    tick_counter: u32,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
enum Message {
    Tick,
}

impl App {
    fn new() -> Self {
        let mut reg = project::ModuleRegistry::new();
        reg.register::<ModelingModule>();
        let mut project = project::Project::new();
        let project_view = project.create_view(&reg).unwrap();
        let mut cb = project::ChangeBuilder::from(&project_view);

        let mut doc = project_view.create_document(&mut cb, "/doc".try_into().unwrap());
        let data_uuid = *doc.create_data::<ModelingModule>();
        let mut data = doc.create_data::<ModelingModule>();

        data.apply_persistent(
            modeling_module::persistent_data::ModelingTransaction::Create(
                modeling_module::persistent_data::Create {
                    before: None,
                    operation: modeling_module::operation::ModelingOperation::Sketch(
                        modeling_module::operation::sketch::Sketch,
                    ),
                },
            ),
        );
        project.apply_changes(cb, &reg).unwrap();

        let project_view = Arc::new(project.create_view(&reg).unwrap());

        let mut viewport = viewport::Viewport::new(project_view.clone());
        let workspace = modeling_workspace::ModelingWorkspace { data_uuid };
        // TODO: this should dynamically select the first fitting plugin
        let plugin = workspace.viewport_plugins()[0].clone();
        viewport.pipeline.add_dynamic_plugin(plugin).unwrap();
        Self {
            viewport,
            project,
            project_view,
            data_uuid,
            reg,
            tick_counter: 0,
        }
    }

    #[expect(clippy::needless_pass_by_value, reason = "required by iced")]
    fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                let data = self
                    .project_view
                    .open_data_by_id::<ModelingModule>(self.data_uuid)
                    .unwrap();

                let mut cb = project::ChangeBuilder::from(&data);
                if self.tick_counter > 100 {
                    data.apply_persistent(
                        modeling_module::persistent_data::ModelingTransaction::Create(
                            modeling_module::persistent_data::Create {
                                before: None,
                                operation: modeling_module::operation::ModelingOperation::Grow,
                            },
                        ),
                        &mut cb,
                    );

                    println!("grow");
                    self.tick_counter = 0;
                } else {
                    let data = self
                        .project_view
                        .open_data_by_id::<ModelingModule>(self.data_uuid)
                        .unwrap();

                    let mut cb = project::ChangeBuilder::from(&data);

                    data.apply_session((), &mut cb);
                }
                self.project.apply_changes(cb, &self.reg).unwrap();
                self.project_view = Arc::new(self.project.create_view(&self.reg).unwrap());
                self.viewport.update(self.project_view.clone());
                self.tick_counter += 1;
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Message> {
        let viewport_shader = iced::widget::shader(&self.viewport)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill);

        iced::widget::column!(iced::widget::text("Viewport:"), viewport_shader).into()
    }

    #[expect(clippy::unused_self)]
    fn subscription(&self) -> Subscription<Message> {
        time::every(time::Duration::from_millis(20)).map(|_| Message::Tick)
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
        .subscription(App::subscription)
        .run()
        .unwrap();
}
