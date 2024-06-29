#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::cognitive_complexity)]

use iced::Sandbox;

struct App {}

impl iced::Sandbox for App {
    type Message = ();

    fn new() -> Self {
        Self {}
    }

    fn title(&self) -> String {
        "Hello".to_string()
    }

    fn update(&mut self, _message: Self::Message) {}

    fn view(&self) -> iced::Element<'_, Self::Message> {
        iced::widget::button("Hello").into()
    }
}

fn main() -> iced::Result {
    App::run(iced::Settings::default())
}
