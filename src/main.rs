use iced_sentinel as sentinel;

mod module;
mod timeline;

pub use module::Module;
pub use timeline::Timeline;

use crate::sentinel::timing;

use iced::advanced::debug;
use iced::executor;
use iced::subscription::{self, Subscription};
use iced::theme::{self, Theme};
use iced::time::SystemTime;
use iced::widget::{button, column, horizontal_space, pane_grid, row, slider, text};
use iced::window;
use iced::{Alignment, Application, Command, Element, Settings};

pub fn main() -> iced::Result {
    Inspector::run(Settings::default())
}

#[derive(Debug)]
struct Inspector {
    state: State,
    theme: Theme,
    timeline: Timeline,
    playhead: timeline::Index,
    modules: pane_grid::State<Module>,
}

#[derive(Debug)]
enum State {
    Connected { version: sentinel::Version },
    Disconnected { at: Option<SystemTime> },
}

#[derive(Debug, Clone)]
enum Message {
    EventReported(sentinel::Event),
    PlayheadChanged(timeline::Index),
    GoLive,
}

impl Application for Inspector {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let (mut modules, update) =
            pane_grid::State::new(Module::performance_chart(timing::Stage::Update));

        let (draw, _) = modules
            .split(
                pane_grid::Axis::Vertical,
                update,
                Module::performance_chart(timing::Stage::Draw(window::Id::MAIN)),
            )
            .unwrap();

        let (_view, _) = modules
            .split(
                pane_grid::Axis::Horizontal,
                update,
                Module::performance_chart(timing::Stage::View(window::Id::MAIN)),
            )
            .unwrap();

        modules.split(
            pane_grid::Axis::Horizontal,
            draw,
            Module::performance_chart(timing::Stage::Render(window::Id::MAIN)),
        );

        (
            Inspector {
                state: State::Disconnected { at: None },
                theme: Theme::TokyoNight,
                timeline: Timeline::new(),
                playhead: timeline::Index::default(),
                modules,
            },
            Command::none(),
        )
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::EventReported(event) => {
                debug::skip_next_timing();

                for (_, module) in self.modules.iter_mut() {
                    module.invalidate_by(&event);
                }

                match event.clone() {
                    sentinel::Event::Connected { version, .. } => {
                        self.state = State::Connected { version };
                    }
                    sentinel::Event::Disconnected { at } => {
                        self.state = State::Disconnected { at: Some(at) };
                    }
                    sentinel::Event::TimingMeasured(_timing) => {}
                    sentinel::Event::ThemeChanged { palette, .. } => {
                        self.theme = Theme::custom(String::from("Custom"), palette);
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
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let header = {
            let status = match &self.state {
                State::Connected { version, .. } => text(format!("Connected! ({version})")),
                State::Disconnected { at: None } => text("Disconnected"),
                State::Disconnected { at: Some(at) } => text(format!("Disconnected ({at:?})")), // TODO: Proper time formatting
            }
            .size(12);

            let counter = text(self.timeline.len()).size(12);

            row![status, horizontal_space(), counter].spacing(10)
        };

        let modules = pane_grid(&self.modules, |_pane, module, _focused| {
            let content = module.view(&self.timeline, self.playhead);

            let title_bar = pane_grid::TitleBar::new(text(module.title()));

            pane_grid::Content::new(content).title_bar(title_bar)
        })
        .spacing(10);

        let timeline = row![
            slider(
                self.timeline.range(),
                self.playhead,
                Message::PlayheadChanged,
            ),
            button(text("→").size(14))
                .on_press_maybe((!self.timeline.is_live(self.playhead)).then_some(Message::GoLive))
                .padding([2, 5])
                .style(theme::Button::Secondary)
        ]
        .align_items(Alignment::Center)
        .spacing(10);

        column![header, modules, timeline]
            .spacing(10)
            .padding(10)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        subscription::run(sentinel::run).map(Message::EventReported)
    }

    fn title(&self) -> String {
        String::from("Inspector - Iced")
    }

    fn theme(&self) -> Self::Theme {
        self.theme.clone()
    }
}
