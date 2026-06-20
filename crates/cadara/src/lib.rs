#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cognitive_complexity)]

use iced::{time, Subscription};
use modeling_module::operation::{
    extrude::{Extrude, ExtrudeChange, ExtrudeDirection, ExtrudeMode},
    fillet::{Fillet, FilletTarget},
    sketch::{Line, Plane, Point2D, Sketch, SketchPrimitive},
    ModelingOperation,
};
use modeling_module::persistent_data::{Create, ModelingTransaction};
use modeling_module::ModelingModule;
use std::sync::Arc;
use uuid::Uuid;
use workspace::Workspace;

struct App {
    viewport: viewport::Viewport,
    project: project::Project,
    project_view: Arc<project::ProjectView>,
    reg: project::ModuleRegistry,
    data_uuid: project::DataId,
    extrude_id: Uuid,
    depth: f64,
    tick: u32,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Tick,
}

fn square_xy() -> Sketch {
    let mk = |from, to| (Uuid::new_v4(), SketchPrimitive::Line(Line { from, to }));
    Sketch {
        plane: Plane::XY,
        primitives: vec![
            mk(Point2D::new(0.0, 0.0), Point2D::new(1.0, 0.0)),
            mk(Point2D::new(1.0, 0.0), Point2D::new(1.0, 1.0)),
            mk(Point2D::new(1.0, 1.0), Point2D::new(0.0, 1.0)),
            mk(Point2D::new(0.0, 1.0), Point2D::new(0.0, 0.0)),
        ],
    }
}

impl App {
    fn new() -> Self {
        let mut reg = project::ModuleRegistry::new();
        reg.register::<ModelingModule>();
        let mut project = project::Project::new();
        let project_view = project.create_view(&reg).unwrap();
        let mut cb = project::ChangeBuilder::from(&project_view);

        let mut doc = project_view.create_document(&mut cb, "/doc".try_into().unwrap());
        let mut data = doc.create_data::<ModelingModule>();
        let data_uuid = *data;

        let sketch_id = Uuid::new_v4();
        data.apply_persistent(ModelingTransaction::Create(Create {
            id: sketch_id,
            before: None,
            name: "sketch".into(),
            operation: ModelingOperation::Sketch(square_xy()),
        }));
        let extrude_id = Uuid::new_v4();
        data.apply_persistent(ModelingTransaction::Create(Create {
            id: extrude_id,
            before: None,
            name: "extrude".into(),
            operation: ModelingOperation::Extrude(Extrude {
                sketch_id,
                depth: 0.3,
                direction: ExtrudeDirection::Normal,
                mode: ExtrudeMode::Add,
            }),
        }));
        data.apply_persistent(ModelingTransaction::Create(Create {
            id: Uuid::new_v4(),
            before: None,
            name: "fillet".into(),
            operation: ModelingOperation::Fillet(Fillet {
                radius: 0.1,
                target: FilletTarget::WholeBody,
            }),
        }));
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
            reg,
            data_uuid,
            extrude_id,
            depth: 0.3,
            tick: 0,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Tick => {
                self.tick += 1;
                // Periodically grow the extrude depth so the demo applies real
                // changes through the project, not just redraws.
                if self.tick >= 100 {
                    self.tick = 0;
                    self.depth = if self.depth >= 0.6 {
                        0.3
                    } else {
                        self.depth + 0.1
                    };
                    let data = self
                        .project_view
                        .open_data_by_id::<ModelingModule>(self.data_uuid)
                        .unwrap();
                    let mut cb = project::ChangeBuilder::from(&data);
                    data.apply_persistent(
                        ModelingTransaction::EditExtrude {
                            step_id: self.extrude_id,
                            change: ExtrudeChange::SetDepth(self.depth),
                        },
                        &mut cb,
                    );
                    self.project.apply_changes(cb, &self.reg).unwrap();
                    self.project_view = Arc::new(self.project.create_view(&self.reg).unwrap());
                }
                // Ticks also drive redraws: Program::update mutates camera state
                // through a Mutex but does not request a redraw on its own, so
                // without them rotation, panning and resize never reach the screen.
                self.viewport.update(self.project_view.clone());
            }
        }
    }

    fn view(&self) -> iced::Element<'_, Message> {
        let viewport_shader = iced::widget::shader(&self.viewport)
            .width(iced::Length::Fill)
            .height(iced::Length::Fill);

        iced::widget::column!(iced::widget::text("Viewport:"), viewport_shader).into()
    }

    #[expect(clippy::unused_self, reason = "iced subscription takes &self")]
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

    iced::application(App::new, App::update, App::view)
        .title("CADara")
        .subscription(App::subscription)
        .run()
        .unwrap();
}
