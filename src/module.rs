use crate::beacon;
use crate::beacon::span::{self, Span};
use crate::timeline::{self, Timeline};

use iced::mouse;
use iced::time::SystemTime;
use iced::widget::{canvas, column, scrollable, text};
use iced::{Element, Font, Length, Pixels, Point, Rectangle, Renderer, Size, Theme};

use std::fmt;

#[derive(Debug)]
pub enum Module {
    PerformanceChart { stage: Stage, cache: canvas::Cache },
    CommandsSpawned { cache: canvas::Cache },
    SubscriptionsAlive { cache: canvas::Cache },
    MessageRate { cache: canvas::Cache },
    MessageLog,
}

impl Module {
    pub fn performance_chart(stage: Stage) -> Self {
        Self::PerformanceChart {
            stage,
            cache: canvas::Cache::new(),
        }
    }

    pub fn commands_spawned() -> Self {
        Self::CommandsSpawned {
            cache: canvas::Cache::new(),
        }
    }

    pub fn subscriptions_alive() -> Self {
        Self::SubscriptionsAlive {
            cache: canvas::Cache::new(),
        }
    }

    pub fn message_rate() -> Self {
        Self::MessageRate {
            cache: canvas::Cache::new(),
        }
    }

    pub fn message_log() -> Self {
        Self::MessageLog
    }

    pub fn title(&self) -> String {
        match self {
            Self::PerformanceChart { stage, .. } => stage.to_string(),
            Module::CommandsSpawned { .. } => String::from("Commands Spawned"),
            Module::SubscriptionsAlive { .. } => String::from("Subscriptions Alive"),
            Module::MessageRate { .. } => String::from("Message Rate"),
            Module::MessageLog => String::from("Message Log"),
        }
    }

    pub fn invalidate(&mut self) {
        match self {
            Self::PerformanceChart { cache, .. }
            | Self::CommandsSpawned { cache }
            | Self::SubscriptionsAlive { cache }
            | Self::MessageRate { cache } => {
                cache.clear();
            }
            Self::MessageLog => {}
        }
    }

    pub fn invalidate_by(&mut self, event: &beacon::Event) {
        let should_invalidate = match (&self, event) {
            (Self::PerformanceChart { stage, .. }, beacon::Event::SpanFinished { span, .. }) => {
                &Stage::from(span.stage()) == stage
            }
            (Self::PerformanceChart { .. }, beacon::Event::ThemeChanged { .. }) => true,
            (
                Self::CommandsSpawned { .. } | Self::MessageRate { .. },
                beacon::Event::SpanFinished { span, .. },
            ) => span.stage() == span::Stage::Update,
            (Self::SubscriptionsAlive { .. }, beacon::Event::SubscriptionsTracked { .. }) => true,
            _ => false,
        };

        if should_invalidate {
            self.invalidate();
        }
    }

    pub fn view<'a, Message: 'a>(
        &'a self,
        timeline: &'a Timeline,
        playhead: timeline::Index,
    ) -> Element<'a, Message> {
        match self {
            Self::PerformanceChart { stage, cache } => canvas(PerformanceChart {
                timeline,
                playhead,
                cache,
                stage: stage.clone(),
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
            Self::CommandsSpawned { cache } => canvas(CommandsSpawned {
                timeline,
                playhead,
                cache,
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
            Self::SubscriptionsAlive { cache } => canvas(SubscriptionsAlive {
                timeline,
                playhead,
                cache,
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
            Self::MessageRate { cache } => canvas(MessageRate {
                timeline,
                playhead,
                cache,
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
            Module::MessageLog => {
                let messages = timeline
                    .seek(playhead)
                    .filter_map(|event| match event {
                        beacon::Event::SpanFinished {
                            span: Span::Update { message, .. },
                            ..
                        } => Some(message),
                        _ => None,
                    })
                    .map(|message| text(message).size(10).font(Font::MONOSPACE).into())
                    .take(10)
                    .collect::<Vec<_>>();

                scrollable(column(messages.into_iter().rev()).spacing(5).padding(5))
                    .anchor_bottom()
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stage {
    Boot,
    Update,
    View,
    Layout,
    Interact,
    Draw,
    Present,
    Custom(String),
}

impl From<span::Stage> for Stage {
    fn from(stage: span::Stage) -> Self {
        match stage {
            span::Stage::Boot => Stage::Boot,
            span::Stage::Update => Stage::Update,
            span::Stage::View(_id) => Stage::View,
            span::Stage::Layout(_id) => Stage::Layout,
            span::Stage::Interact(_id) => Stage::Interact,
            span::Stage::Draw(_id) => Stage::Draw,
            span::Stage::Present(_id) => Stage::Present,
            span::Stage::Custom(_id, name) => Stage::Custom(name),
        }
    }
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Stage::Boot => "Boot",
            Stage::Update => "Update",
            Stage::View => "View",
            Stage::Layout => "Layout",
            Stage::Interact => "Interact",
            Stage::Draw => "Draw",
            Stage::Present => "Present",
            Stage::Custom(name) => &name,
        })
    }
}

struct PerformanceChart<'a> {
    timeline: &'a Timeline,
    playhead: timeline::Index,
    cache: &'a canvas::Cache,
    stage: Stage,
}

impl<'a, Message> canvas::Program<Message> for PerformanceChart<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        // TODO: Configurable zoom
        const BAR_WIDTH: f32 = 10.0;

        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            draw_bar_chart(
                BarChartConfig {
                    bar_width: BAR_WIDTH,
                },
                frame,
                theme,
                self.timeline
                    .timeframes(self.playhead, &self.stage)
                    .map(|timeframe| timeframe.duration),
                |duration| duration.as_secs_f64(),
                |duration| format!("{duration:?}"),
                |duration, n| duration / n,
                |duration| duration.as_secs_f64(),
                |duration| format!("{duration:?}"),
            );
        });

        vec![geometry]
    }
}

struct CommandsSpawned<'a> {
    timeline: &'a Timeline,
    playhead: timeline::Index,
    cache: &'a canvas::Cache,
}

impl<'a, Message> canvas::Program<Message> for CommandsSpawned<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        // TODO: Configurable zoom
        const BAR_WIDTH: f32 = 10.0;

        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            draw_bar_chart(
                BarChartConfig {
                    bar_width: BAR_WIDTH,
                },
                frame,
                theme,
                self.timeline
                    .seek(self.playhead)
                    .filter_map(|event| match event {
                        beacon::Event::SpanFinished {
                            span:
                                Span::Update {
                                    commands_spawned, ..
                                },
                            ..
                        } => Some(*commands_spawned),
                        _ => None,
                    }),
                |amount| amount as f64,
                |amount| amount.to_string(),
                |amount, n| amount as f64 / n as f64,
                std::convert::identity,
                |average| format!("{:.1}", average),
            );
        });

        vec![geometry]
    }
}

struct SubscriptionsAlive<'a> {
    timeline: &'a Timeline,
    playhead: timeline::Index,
    cache: &'a canvas::Cache,
}

impl<'a, Message> canvas::Program<Message> for SubscriptionsAlive<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        // TODO: Configurable zoom
        const BAR_WIDTH: f32 = 10.0;

        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            draw_bar_chart(
                BarChartConfig {
                    bar_width: BAR_WIDTH,
                },
                frame,
                theme,
                self.timeline
                    .seek(self.playhead)
                    .filter_map(|event| match event {
                        beacon::Event::SubscriptionsTracked { amount_alive, .. } => {
                            Some(*amount_alive)
                        }
                        _ => None,
                    }),
                |amount| amount as f64,
                |amount| amount.to_string(),
                |amount, n| amount as f64 / n as f64,
                std::convert::identity,
                |average| format!("{:.1}", average),
            );
        });

        vec![geometry]
    }
}

struct MessageRate<'a> {
    timeline: &'a Timeline,
    playhead: timeline::Index,
    cache: &'a canvas::Cache,
}

impl<'a, Message> canvas::Program<Message> for MessageRate<'a> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        // TODO: Configurable zoom
        const BAR_WIDTH: f32 = 10.0;

        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let updates_per_second = {
                let mut updates =
                    self.timeline
                        .seek(self.playhead)
                        .filter_map(|event| match event {
                            beacon::Event::SpanFinished { at, span, .. }
                                if span.stage() == span::Stage::Update =>
                            {
                                Some(*at)
                            }
                            _ => None,
                        });

                let mut current_bucket = 1;
                let mut current_second = updates.next().map(|time| {
                    time.duration_since(SystemTime::UNIX_EPOCH)
                        .ok()
                        .unwrap_or_default()
                        .as_secs()
                });

                std::iter::from_fn(move || {
                    while let Some(time) = updates.next() {
                        let second = time
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();

                        if Some(second) != current_second {
                            let bucket = current_bucket;

                            current_second = Some(second);
                            current_bucket = 1;

                            return Some(bucket);
                        }

                        current_bucket += 1;
                    }

                    current_second.take().is_some().then_some(current_bucket)
                })
            };

            draw_bar_chart(
                BarChartConfig {
                    bar_width: BAR_WIDTH,
                },
                frame,
                theme,
                updates_per_second,
                |amount| amount as f64,
                |amount| format!("{amount} msg/s"),
                |amount, n| amount as f64 / n as f64,
                std::convert::identity,
                |average| format!("{:.1} msg/s", average),
            );
        });

        vec![geometry]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct BarChartConfig {
    bar_width: f32,
}

fn draw_bar_chart<T, A>(
    config: BarChartConfig,
    frame: &mut canvas::Frame,
    theme: &Theme,
    datapoints: impl Iterator<Item = T> + Clone,
    to_float: impl Fn(T) -> f64,
    to_string: impl Fn(T) -> String,
    average: impl Fn(T, u32) -> A,
    average_to_float: impl Fn(A) -> f64,
    average_to_string: impl Fn(A) -> String,
) where
    T: Copy + Ord + std::iter::Sum,
    A: Copy,
{
    let bounds = frame.size();
    let palette = theme.extended_palette();

    let amount = (bounds.width / config.bar_width).ceil() as usize;

    let Some(max) = datapoints.clone().take(amount).max() else {
        return;
    };

    let average = {
        let mut n = 0;

        let sum = datapoints
            .clone()
            .take(amount * 3)
            .map(|datapoint| {
                n += 1;
                datapoint
            })
            .sum::<T>();

        average(sum, n)
    };

    let average_value = average_to_float(average);
    let average_pixels = f64::from(bounds.height) / (2.0 * average_value);
    let max_pixels = f64::from(bounds.height) / to_float(max);

    let pixels_per_unit = average_pixels.min(max_pixels);

    for (i, datapoint) in datapoints.take(amount).enumerate() {
        let value = to_float(datapoint);
        let bar_height = (value * pixels_per_unit) as f32;

        frame.fill_rectangle(
            Point::new(
                bounds.width - config.bar_width * (i + 1) as f32,
                bounds.height - bar_height,
            ),
            Size::new(config.bar_width, bar_height),
            if value < average_value as f64 / 2.0 {
                palette.success.base.color
            } else if value > average_value as f64 * 3.0 {
                palette.danger.base.color
            } else {
                palette.background.strong.color
            },
        )
    }

    frame.fill_text(canvas::Text {
        content: format!("Average: {}", average_to_string(average)),
        position: Point::new(4.0, 4.0),
        color: palette.background.base.text,
        size: Pixels(14.0),
        font: Font::MONOSPACE,
        ..canvas::Text::default()
    });

    frame.fill_text(canvas::Text {
        content: format!("Maximum: {}", to_string(max)),
        position: Point::new(4.0, 22.0),
        color: palette.background.base.text,
        size: Pixels(14.0),
        font: Font::MONOSPACE,
        ..canvas::Text::default()
    });
}
