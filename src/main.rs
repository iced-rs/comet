use iced::executor;
use iced::subscription::{self, Subscription};
use iced::theme::Theme;
use iced::widget::{container, text};
use iced::{Application, Command, Element, Length, Settings};
use protocol::server;

pub fn main() -> iced::Result {
    Inspector::run(Settings::default())
}

#[derive(Debug)]
struct Inspector {
    state: State,
}

#[derive(Debug)]
enum State {
    Connected,
    Disconnected,
}

#[derive(Debug, Clone)]
enum Message {
    Server(server::Message),
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
            },
            Command::none(),
        )
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Server(message) => match message {
                server::Message::Connected => {
                    self.state = State::Connected;
                }
                server::Message::Disconnected => {
                    self.state = State::Disconnected;
                }
                server::Message::PerformanceReported(_performance) => {
                    // TODO
                }
            },
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let content = match &self.state {
            State::Connected => text(format!("Connected!")),
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
        subscription::run(server::run).map(Message::Server)
    }

    fn title(&self) -> String {
        String::from("Inspector - Iced")
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }
}
