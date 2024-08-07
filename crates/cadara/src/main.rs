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
        let tab = icara::Tab::new(iced::widget::text("Hello"))
            .trailing(
                iced::widget::button(
                    iced::widget::text("X")
                        .horizontal_alignment(iced::alignment::Horizontal::Right),
                )
                .on_press(()),
            )
            .on_press(());
        let tabs = iced::widget::row![
            iced::widget::text("<Projects tab>"),
            iced::widget::text("<file 1 tab>"),
            iced::widget::text("<file 2 tab>"),
            tab
        ];

        iced::widget::column![
            tabs,
            iced::widget::row![
                // toolbar
                iced::widget::column![
                    // document tools
                    iced::widget::text("Document"),
                    iced::widget::container(
                        // here border
                        iced::widget::row![
                            iced::widget::text("<icon>"),
                            iced::widget::text("<icon>"),
                            iced::widget::Button::new(iced::widget::text("test"))
                                .padding(iced::Padding::new(10.0)),
                        ]
                    )
                ],
                iced::widget::column![
                    // create tools
                    iced::widget::text("Create"),
                    iced::widget::container(
                        // here border
                        iced::widget::row![
                            iced::widget::text("<icon>"),
                            iced::widget::text("<icon>"),
                        ]
                    )
                ]
            ],
        ]
        .into()
    }
}

fn main() -> iced::Result {
    App::run(iced::Settings {
        antialiasing: true,
        ..iced::Settings::default()
    })
}
