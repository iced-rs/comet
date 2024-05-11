use iced_beacon as beacon;
use iced_beacon::core;

mod module;
mod timeline;

pub use module::Module;
pub use timeline::Timeline;

use crate::beacon::span;

use iced::advanced::debug;
use iced::keyboard;
use iced::program;
use iced::subscription::{self, Subscription};
use iced::time::SystemTime;
use iced::widget::{
    button, center, column, container, horizontal_space, pane_grid, progress_bar, row, slider, svg,
    text,
};
use iced::window;
use iced::{Alignment, Background, Border, Command, Element, Font, Point, Settings, Size, Theme};

pub fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    if beacon::is_running() {
        log::warn!("Comet is already running. Exiting...");
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
}

impl Comet {
    fn new() -> Self {
        let logo = svg::Handle::from_memory(include_bytes!("../assets/logo.svg"));

        Self {
            state: State::Waiting,
            theme: Theme::TokyoNight,
            timeline: Timeline::new(),
            playhead: timeline::Index::default(),
            modules: pane_grid::State::with_configuration(performance_board()),
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
                    beacon::Event::SpanFinished { .. } => {}
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
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        match &self.state {
            State::Waiting => center(
                row![
                    svg(self.logo.clone()).width(100).height(100),
                    text("Comet").font(Font::MONOSPACE).size(70),
                ]
                .spacing(30)
                .align_items(Alignment::Center),
            )
            .into(),
            State::Working { name, connection } => {
                let header = {
                    let logo = row![
                        svg(self.logo.clone()).width(24).height(24),
                        text(name).font(Font::MONOSPACE).size(18),
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

                    let counter = column![
                        text(format!(
                            "{} / {}",
                            self.timeline.len(),
                            self.timeline.capacity(),
                        ))
                        .font(Font::MONOSPACE)
                        .size(8),
                        progress_bar(
                            0.0..=self.timeline.capacity() as f32,
                            self.timeline.len() as f32
                        )
                        .height(3),
                    ]
                    .width(100)
                    .spacing(2)
                    .align_items(Alignment::End);

                    row![logo, status, horizontal_space(), counter]
                        .spacing(10)
                        .align_items(Alignment::Center)
                };

                let modules = pane_grid(&self.modules, |_pane, module, _focused| {
                    let content = module.view(&self.timeline, self.playhead);

                    let title_bar =
                        pane_grid::TitleBar::new(text(module.title()).font(Font::MONOSPACE));

                    pane_grid::Content::new(content).title_bar(title_bar)
                })
                .spacing(10);

                let timeline = {
                    row![
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
            State::Waiting => String::from("Comet"),
            State::Working { name, .. } => format!("{name} - Comet"),
        }
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}

fn performance_board() -> pane_grid::Configuration<Module> {
    let update_and_view = pane_grid::Configuration::Split {
        axis: pane_grid::Axis::Vertical,
        ratio: 0.5,
        a: Box::new(pane_grid::Configuration::Pane(Module::performance_chart(
            span::Stage::Update,
        ))),
        b: Box::new(pane_grid::Configuration::Pane(Module::performance_chart(
            span::Stage::View(window::Id::MAIN),
        ))),
    };

    let layout_and_interact = pane_grid::Configuration::Split {
        axis: pane_grid::Axis::Vertical,
        ratio: 0.5,
        a: Box::new(pane_grid::Configuration::Pane(Module::performance_chart(
            span::Stage::Layout(window::Id::MAIN),
        ))),
        b: Box::new(pane_grid::Configuration::Pane(Module::performance_chart(
            span::Stage::Interact(window::Id::MAIN),
        ))),
    };

    let draw_and_present = pane_grid::Configuration::Split {
        axis: pane_grid::Axis::Vertical,
        ratio: 0.5,
        a: Box::new(pane_grid::Configuration::Pane(Module::performance_chart(
            span::Stage::Draw(window::Id::MAIN),
        ))),
        b: Box::new(pane_grid::Configuration::Pane(Module::performance_chart(
            span::Stage::Present(window::Id::MAIN),
        ))),
    };

    pane_grid::Configuration::Split {
        axis: pane_grid::Axis::Horizontal,
        ratio: 1.0 / 3.0,
        a: Box::new(update_and_view),
        b: Box::new(pane_grid::Configuration::Split {
            axis: pane_grid::Axis::Horizontal,
            ratio: 0.5,
            a: Box::new(layout_and_interact),
            b: Box::new(draw_and_present),
        }),
    }
}
