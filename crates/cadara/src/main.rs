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
        let mut viewport = viewport::Viewport::default();
        let workspace = demo_workspace::DemoWorkspace::default();
        // TODO: this should dynamically select the first fitting plugin
        let plugin = workspace.viewport_plugins()[0].clone();
        viewport.pipeline.add_dynamic_plugin(plugin).unwrap();
        Self { viewport }
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
