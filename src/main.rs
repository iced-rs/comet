use iced_beacon as beacon;
use iced_beacon::core;

mod chart;
mod screen;
mod timeline;
mod widget;

use crate::screen::Screen;
use crate::timeline::Timeline;
use crate::widget::{circle, diffused_text};

use iced::border;
use iced::debug;
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
    playhead: timeline::Playhead,
    screen: Screen,
    bar_width: chart::BarWidth,
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
    TogglePause,
    Previous,
    Next,
    GoLive,
    ShowOverview,
    ShowUpdate,
    ShowPresent,
    ShowCustom,
    Custom(screen::custom::Message),
    IncrementBarWidth,
    DecrementBarWidth,
    Quit,
}

impl Comet {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                logo: svg::Handle::from_memory(include_bytes!("../assets/logo.svg")),
                state: State::Waiting,
                theme: Theme::CatppuccinMocha,
                timeline: Timeline::new(),
                playhead: timeline::Playhead::Live,
                screen: Screen::Overview(screen::Overview::new()),
                bar_width: chart::BarWidth::default(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::EventReported(event) => {
                debug::skip_next_timing();

                match event.clone() {
                    beacon::Event::Connected { name, version, .. } => {
                        let current_name = match &self.state {
                            State::Working { name, .. } => Some(name),
                            State::Waiting => None,
                        };

                        if Some(&name) != current_name {
                            self.playhead = timeline::Playhead::Live;
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

                self.screen.invalidate_by(&event);
                self.timeline.push(event);

                Task::none()
            }
            Message::PlayheadChanged(index) => {
                self.playhead = timeline::Playhead::Paused(index);
                self.screen.invalidate();

                Task::none()
            }
            Message::TogglePause => {
                self.playhead = if self.playhead.is_live() {
                    timeline::Playhead::Paused(self.timeline.end())
                } else {
                    timeline::Playhead::Live
                };

                self.screen.invalidate();

                Task::none()
            }
            Message::Previous => {
                self.playhead = match self.playhead {
                    timeline::Playhead::Live => timeline::Playhead::Paused(self.timeline.end()),
                    timeline::Playhead::Paused(index) => timeline::Playhead::Paused(index - 1),
                };

                self.screen.invalidate();

                Task::none()
            }
            Message::Next => {
                self.playhead = match self.playhead {
                    timeline::Playhead::Live => timeline::Playhead::Live,
                    timeline::Playhead::Paused(index) => {
                        if index + 1 >= self.timeline.end() {
                            timeline::Playhead::Live
                        } else {
                            timeline::Playhead::Paused(index + 1)
                        }
                    }
                };

                self.screen.invalidate();

                Task::none()
            }
            Message::GoLive => {
                self.playhead = timeline::Playhead::Live;
                self.screen.invalidate();

                Task::none()
            }
            Message::ShowOverview => {
                self.screen = Screen::Overview(screen::Overview::new());

                Task::none()
            }
            Message::ShowUpdate => {
                self.screen = Screen::Update(screen::Update::new());

                Task::none()
            }
            Message::ShowPresent => {
                self.screen = Screen::Present(screen::Present::new());

                Task::none()
            }
            Message::ShowCustom => {
                self.screen = Screen::Custom(screen::Custom::new(&self.timeline, self.playhead));

                Task::none()
            }
            Message::Custom(message) => {
                if let Screen::Custom(custom) = &mut self.screen {
                    custom.update(message);
                }

                Task::none()
            }
            Message::IncrementBarWidth => {
                self.bar_width = self.bar_width.increment();
                self.screen.invalidate();

                Task::none()
            }
            Message::DecrementBarWidth => {
                self.bar_width = self.bar_width.decrement();
                self.screen.invalidate();

                Task::none()
            }
            Message::Quit => iced::exit(),
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
                            tab(
                                "Present",
                                Message::ShowPresent,
                                matches!(self.screen, Screen::Present(_))
                            ),
                            tab(
                                "Custom",
                                Message::ShowCustom,
                                matches!(self.screen, Screen::Custom(_))
                            )
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
                    Screen::Overview(overview) => {
                        overview.view(&self.timeline, self.playhead, self.bar_width)
                    }
                    Screen::Update(update) => {
                        update.view(&self.timeline, self.playhead, self.bar_width)
                    }
                    Screen::Present(present) => {
                        present.view(&self.timeline, self.playhead, self.bar_width)
                    }
                    Screen::Custom(custom) => custom
                        .view(&self.timeline, self.playhead, self.bar_width)
                        .map(Message::Custom),
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
                        self.timeline.index(self.playhead),
                        Message::PlayheadChanged,
                    );

                    let live: Element<_> = {
                        let is_live = self.playhead.is_live();

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

        let hotkeys = keyboard::on_key_press(|key, _| match key.as_ref() {
            keyboard::Key::Named(keyboard::key::Named::F12) => Some(Message::Quit),
            keyboard::Key::Named(keyboard::key::Named::Space) => Some(Message::TogglePause),
            keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => Some(Message::Previous),
            keyboard::Key::Named(keyboard::key::Named::ArrowRight) => Some(Message::Next),
            keyboard::Key::Character("o") => Some(Message::ShowOverview),
            keyboard::Key::Character("u") => Some(Message::ShowUpdate),
            keyboard::Key::Character("p") => Some(Message::ShowPresent),
            keyboard::Key::Character("c") => Some(Message::ShowCustom),
            keyboard::Key::Named(keyboard::key::Named::ArrowUp) => Some(Message::IncrementBarWidth),
            keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                Some(Message::DecrementBarWidth)
            }
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
