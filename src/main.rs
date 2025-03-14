use iced_beacon as beacon;
use iced_beacon::core;

mod chart;
mod screen;
mod timeline;
mod widget;

use crate::screen::Screen;
use crate::timeline::Timeline;
use crate::widget::{circle, diffused_text};

use iced::advanced::debug;
use iced::border;
use iced::keyboard;
use iced::time::SystemTime;
use iced::widget::{
    bottom, button, center, column, container, horizontal_rule, horizontal_space, progress_bar,
    row, rule, slider, stack, svg, text, tooltip,
};
use iced::window;
use iced::{Center, Element, Font, Point, Shrink, Size, Subscription, Task, Theme};

pub fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    if beacon::is_running() {
        log::warn!("comet is already running. Exiting...");
        std::process::exit(0);
    }

    iced::application(Comet::new, Comet::update, Comet::view)
        .title(Comet::title)
        .subscription(Comet::subscription)
        .theme(Comet::theme)
        .window(window::Settings {
            size: Size::new(800.0, 600.0),
            position: window::Position::SpecificWith(|window, monitor| {
                Point::new(monitor.width - window.width - 5.0, 0.0)
            }),
            ..window::Settings::default()
        })
        .run()
}

#[derive(Debug)]
struct Comet {
    logo: svg::Handle,
    state: State,
    theme: Theme,
    timeline: Timeline,
    playhead: timeline::Index,
    screen: Screen,
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
    ShowOverview,
    ShowUpdate,
}

impl Comet {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                logo: svg::Handle::from_memory(include_bytes!("../assets/logo.svg")),
                state: State::Waiting,
                theme: Theme::CatppuccinMocha,
                timeline: Timeline::new(),
                playhead: timeline::Index::default(),
                screen: Screen::Overview(screen::Overview::new()),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::EventReported(event) => {
                debug::skip_next_timing();

                match &mut self.screen {
                    Screen::Overview(overview) => {
                        overview.invalidate_by(&event);
                    }
                    Screen::Update(update) => {
                        update.invalidate_by(&event);
                    }
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
                        return iced::exit();
                    }
                }

                let is_live = self.timeline.is_live(self.playhead);
                let latest = self.timeline.push(event);

                if is_live {
                    self.playhead = latest;
                }

                Task::none()
            }
            Message::PlayheadChanged(playhead) => {
                match &mut self.screen {
                    Screen::Overview(overview) => {
                        overview.invalidate();
                    }
                    Screen::Update(update) => {
                        update.invalidate();
                    }
                }

                self.playhead = playhead;

                Task::none()
            }
            Message::GoLive => {
                match &mut self.screen {
                    Screen::Overview(overview) => {
                        overview.invalidate();
                    }
                    Screen::Update(update) => {
                        update.invalidate();
                    }
                }

                self.playhead = *self.timeline.range().end();

                Task::none()
            }
            Message::Quit => iced::exit(),
            Message::ShowOverview => {
                self.screen = Screen::Overview(screen::Overview::new());

                Task::none()
            }
            Message::ShowUpdate => {
                self.screen = Screen::Update(screen::Update::new());

                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        match &self.state {
            State::Waiting => center(
                row![
                    svg(self.logo.clone()).width(100).height(100),
                    diffused_text("comet")
                        .font(Font::MONOSPACE)
                        .size(70)
                        .height(105)
                ]
                .spacing(30)
                .align_y(Center),
            )
            .into(),
            State::Working { name, connection } => {
                let header = {
                    let logo = row![
                        svg(self.logo.clone()).width(24).height(24),
                        diffused_text(name).font(Font::MONOSPACE).size(18),
                    ]
                    .spacing(10)
                    .align_y(Center);

                    let status = circle(move |palette| match connection {
                        Connection::Connected { .. } => palette.success.base.color,
                        Connection::Disconnected { .. } => palette.danger.base.color,
                    });

                    let time = if let Some(time) = self.timeline.time_at(self.playhead) {
                        let datetime: chrono::DateTime<chrono::Local> = time.into();

                        text(datetime.format("%d/%m/%Y %H:%M:%S%.3f").to_string())
                            .font(Font::MONOSPACE)
                            .size(10)
                            .into()
                    } else {
                        Element::from(horizontal_space())
                    };

                    let tabs = {
                        fn tab<'a>(
                            label: &'static str,
                            on_press: Message,
                            is_active: bool,
                        ) -> Element<'a, Message> {
                            let label = text(label).font(Font::MONOSPACE);

                            if is_active {
                                stack![
                                    container(label).padding([5, 10]),
                                    bottom(horizontal_rule(2).style(|theme: &Theme| rule::Style {
                                        color: theme.palette().text,
                                        width: 2,
                                        radius: border::Radius::default(),
                                        fill_mode: rule::FillMode::Full,
                                    }))
                                ]
                                .into()
                            } else {
                                button(label).on_press(on_press).style(button::text).into()
                            }
                        }

                        row![
                            tab(
                                "Overview",
                                Message::ShowOverview,
                                matches!(self.screen, Screen::Overview(_))
                            ),
                            tab(
                                "Update",
                                Message::ShowUpdate,
                                matches!(self.screen, Screen::Update(_))
                            ),
                        ]
                        .spacing(10)
                        .align_y(Center)
                    };

                    row![logo, status, time, horizontal_space(), tabs]
                        .spacing(10)
                        .align_y(Center)
                        .height(Shrink)
                };

                let screen = match &self.screen {
                    Screen::Overview(overview) => overview.view(&self.timeline, self.playhead),
                    Screen::Update(update) => update.view(&self.timeline, self.playhead),
                };

                let timeline = {
                    let counter = tooltip(
                        progress_bar(
                            0.0..=self.timeline.capacity() as f32,
                            self.timeline.len() as f32,
                        )
                        .girth(10)
                        .length(20),
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

                    let timeline = slider(
                        self.timeline.range(),
                        self.playhead,
                        Message::PlayheadChanged,
                    );

                    let live: Element<_> = {
                        let is_live = self.timeline.is_live(self.playhead);

                        let indicator = circle(move |palette| {
                            if is_live {
                                palette.danger.strong.color
                            } else {
                                palette.background.weak.color
                            }
                        });

                        let live = row![indicator, text("LIVE").size(12).font(Font::MONOSPACE)]
                            .spacing(5)
                            .align_y(Center);

                        if is_live {
                            live.into()
                        } else {
                            button(live)
                                .padding(0)
                                .on_press(Message::GoLive)
                                .style(button::text)
                                .into()
                        }
                    };

                    row![counter, timeline, live].align_y(Center).spacing(10)
                };

                column![header, screen, timeline]
                    .spacing(10)
                    .padding(10)
                    .into()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let beacon = Subscription::run(beacon::run).map(Message::EventReported);

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
