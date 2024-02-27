use iced_sentinel as sentinel;

use crate::sentinel::Version;

use iced::executor;
use iced::subscription::{self, Subscription};
use iced::theme::Theme;
use iced::widget::{container, text};
use iced::{Application, Command, Element, Length, Settings};

pub fn main() -> iced::Result {
    Inspector::run(Settings::default())
}

#[derive(Debug)]
struct Inspector {
    state: State,
    theme: Theme,
}

#[derive(Debug)]
enum State {
    Connected(Version),
    Disconnected,
}

#[derive(Debug, Clone)]
enum Message {
    Server(sentinel::Event),
}

impl Application for Inspector {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Inspector {
                state: State::Disconnected,
                theme: Theme::CatppuccinMocha,
            },
            Command::none(),
        )
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Server(message) => match message {
                sentinel::Event::Connected(version) => {
                    self.state = State::Connected(version);
                }
                sentinel::Event::Disconnected => {
                    self.state = State::Disconnected;
                }
                sentinel::Event::TimingMeasured(_timing) => {
                    // TODO
                }
                sentinel::Event::ThemeChanged(palette) => {
                    self.theme = Theme::custom(String::from("Custom"), palette);
                }
            },
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let content = match &self.state {
            State::Connected(version) => text(format!("Connected! ({version})")),
            State::Disconnected => text("Waiting for incoming connection..."),
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        subscription::run(sentinel::run).map(Message::Server)
    }

    fn title(&self) -> String {
        String::from("Inspector - Iced")
    }

    fn theme(&self) -> Self::Theme {
        self.theme.clone()
    }
}
