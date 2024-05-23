#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::cast_precision_loss)]

mod workspace;

use iced::Sandbox;
use viewport::ViewportPlugin;

struct App {
    viewport: viewport::Viewport,
}

impl iced::Sandbox for App {
    type Message = ();

    fn new() -> Self {
        let mut viewport = viewport::Viewport::default();
        viewport
            .pipeline
            .add_plugin(ViewportPlugin::new(workspace::DemoViewportPlugin::default()).unwrap())
            .unwrap();
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
