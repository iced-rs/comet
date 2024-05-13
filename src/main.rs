use iced_beacon as beacon;
use iced_beacon::core;

mod board;
mod module;
mod timeline;
mod widget;

use crate::board::Board;
use crate::module::Module;
use crate::timeline::Timeline;
use crate::widget::animated_text;

use iced::advanced::debug;
use iced::keyboard;
use iced::program;
use iced::subscription::{self, Subscription};
use iced::time::SystemTime;
use iced::widget::{
    button, center, column, container, horizontal_space, pane_grid, pick_list, progress_bar, row,
    slider, svg, text, tooltip,
};
use iced::window;
use iced::{Alignment, Background, Border, Command, Element, Font, Point, Settings, Size, Theme};

pub fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    if beacon::is_running() {
        log::warn!("comet is already running. Exiting...");
        std::process::exit(0);
    }

    program(Comet::title, Comet::update, Comet::view)
        .subscription(Comet::subscription)
        .theme(Comet::theme)
        .settings(Settings {
            window: window::Settings {
                size: Size::new(800.0, 600.0),
                position: window::Position::SpecificWith(|window, monitor| {
                    Point::new(monitor.width - window.width - 5.0, 0.0)
                }),
                ..window::Settings::default()
            },
            ..Settings::default()
        })
        .run_with(Comet::new)
}

#[derive(Debug)]
struct Comet {
    state: State,
    theme: Theme,
    timeline: Timeline,
    playhead: timeline::Index,
    board: Board,
    modules: pane_grid::State<Module>,
    logo: svg::Handle,
}

#[derive(Debug)]
enum State {
    Waiting,
    Working {
        name: String,
        connection: Connection,
    },
}

#[derive(Debug)]
#[allow(dead_code)]
enum Connection {
    Connected { version: beacon::Version },
    Disconnected { at: SystemTime },
}

#[derive(Debug, Clone)]
enum Message {
    EventReported(beacon::Event),
    PlayheadChanged(timeline::Index),
    GoLive,
    Quit,
    BoardChanged(Board),
}

impl Comet {
    fn new() -> Self {
        let logo = svg::Handle::from_memory(include_bytes!("../assets/logo.svg"));
        let board = Board::Overview;
        let modules = pane_grid::State::with_configuration(board.modules());

        Self {
            state: State::Waiting,
            theme: Theme::TokyoNight,
            timeline: Timeline::new(),
            playhead: timeline::Index::default(),
            board,
            modules,
            logo,
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::EventReported(event) => {
                debug::skip_next_timing();

                for (_, module) in self.modules.iter_mut() {
                    module.invalidate_by(&event);
                }

                match event.clone() {
                    beacon::Event::Connected { name, version, .. } => {
                        let current_name = match &self.state {
                            State::Working { name, .. } => Some(name),
                            State::Waiting => None,
                        };

                        if Some(&name) != current_name {
                            self.playhead = timeline::Index::default();
                            self.timeline.clear();
                        }

                        self.state = State::Working {
                            name,
                            connection: Connection::Connected { version },
                        };
                    }
                    beacon::Event::Disconnected { at } => {
                        if let State::Working { connection, .. } = &mut self.state {
                            *connection = Connection::Disconnected { at };
                        }
                    }
                    beacon::Event::ThemeChanged { palette, .. } => {
                        if let State::Working { name, .. } = &self.state {
                            self.theme = Theme::custom(name.clone(), palette);
                        }
                    }
                    beacon::Event::SpanFinished { .. }
                    | beacon::Event::SubscriptionsTracked { .. } => {}
                    beacon::Event::QuitRequested { .. } | beacon::Event::AlreadyRunning { .. } => {
                        return window::close(window::Id::MAIN);
                    }
                }

                let is_live = self.timeline.is_live(self.playhead);
                let latest = self.timeline.push(event);

                if is_live {
                    self.playhead = latest;
                }
            }
            Message::PlayheadChanged(playhead) => {
                for (_, module) in self.modules.iter_mut() {
                    module.invalidate();
                }

                self.playhead = playhead;
            }
            Message::GoLive => {
                self.playhead = *self.timeline.range().end();
            }
            Message::Quit => {
                return window::close(window::Id::MAIN);
            }
            Message::BoardChanged(board) => {
                self.board = board;
                self.modules = pane_grid::State::with_configuration(board.modules());
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        match &self.state {
            State::Waiting => center(
                row![
                    svg(self.logo.clone()).width(100).height(100),
                    animated_text("comet").font(Font::MONOSPACE).size(70),
                ]
                .spacing(30)
                .align_items(Alignment::Center),
            )
            .into(),
            State::Working { name, connection } => {
                let header = {
                    let logo = row![
                        svg(self.logo.clone()).width(24).height(24),
                        animated_text(name).font(Font::MONOSPACE).size(18),
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center);

                    let status = container(horizontal_space()).width(8).height(8).style(
                        move |theme: &Theme| {
                            let palette = theme.palette();

                            let color = match connection {
                                Connection::Connected { .. } => palette.success,
                                Connection::Disconnected { .. } => palette.danger,
                            };

                            container::Style {
                                background: Some(Background::from(color)),
                                border: Border::rounded(4),
                                ..container::Style::default()
                            }
                        },
                    );

                    let board_selector =
                        pick_list(Board::ALL, Some(self.board), Message::BoardChanged);

                    row![logo, status, horizontal_space(), board_selector]
                        .spacing(10)
                        .align_items(Alignment::Center)
                };

                let modules = pane_grid(&self.modules, |_pane, module, _focused| {
                    let content = module.view(&self.timeline, self.playhead);

                    let title_bar = pane_grid::TitleBar::new(
                        animated_text(module.title()).font(Font::MONOSPACE),
                    );

                    pane_grid::Content::new(content).title_bar(title_bar)
                })
                .spacing(10);

                let timeline = {
                    let counter = tooltip(
                        progress_bar(
                            0.0..=self.timeline.capacity() as f32,
                            self.timeline.len() as f32,
                        )
                        .height(10)
                        .width(20),
                        container(
                            text(format!(
                                "Buffer capacity: {} / {}",
                                self.timeline.len(),
                                self.timeline.capacity(),
                            ))
                            .font(Font::MONOSPACE)
                            .size(8),
                        )
                        .padding(5)
                        .style(container::rounded_box),
                        tooltip::Position::Top,
                    );

                    row![
                        counter,
                        slider(
                            self.timeline.range(),
                            self.playhead,
                            Message::PlayheadChanged,
                        ),
                        button(text("→").size(14))
                            .on_press_maybe(
                                (!self.timeline.is_live(self.playhead)).then_some(Message::GoLive)
                            )
                            .padding([2, 5])
                            .style(button::secondary),
                    ]
                    .align_items(Alignment::Center)
                    .spacing(10)
                };

                column![header, modules, timeline]
                    .spacing(10)
                    .padding(10)
                    .into()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let beacon = subscription::run(beacon::run).map(Message::EventReported);

        let hotkeys = keyboard::on_key_press(|key, _| match key {
            keyboard::Key::Named(keyboard::key::Named::F12) => Some(Message::Quit),
            _ => None,
        });

        Subscription::batch([beacon, hotkeys])
    }

    fn title(&self) -> String {
        match &self.state {
            State::Waiting => String::from("comet"),
            State::Working { name, .. } => format!("{name} - comet"),
        }
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
