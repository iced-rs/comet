use crate::beacon;
use crate::beacon::span::{self, Span};
use crate::timeline::{self, Timeline};

use iced::mouse;
use iced::time::SystemTime;
use iced::widget::canvas;
use iced::{Element, Fill, Font, Pixels, Point, Rectangle, Renderer, Size, Theme};

use std::fmt;

pub use canvas::Cache;

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

pub fn performance<'a, Message>(
    timeline: &'a Timeline,
    playhead: timeline::Index,
    cache: &'a canvas::Cache,
    stage: &'a Stage,
) -> Element<'a, Message>
where
    Message: 'a,
{
    canvas(BarChart {
        datapoints: timeline
            .timeframes(playhead, &stage)
            .map(|timeframe| timeframe.duration),
        cache,
        to_float: |duration| duration.as_secs_f64(),
        to_string: |duration| format!("{duration:?}"),
        average: |duration, n| duration / n,
        average_to_float: |duration| duration.as_secs_f64(),
        average_to_string: |duration| format!("{duration:?}"),
    })
    .width(Fill)
    .height(Fill)
    .into()
}

pub fn tasks_spawned<'a, Message>(
    timeline: &'a Timeline,
    playhead: timeline::Index,
    cache: &'a canvas::Cache,
) -> Element<'a, Message>
where
    Message: 'a,
{
    canvas(BarChart {
        datapoints: timeline.seek(playhead).filter_map(|event| match event {
            beacon::Event::SpanFinished {
                span: Span::Update {
                    commands_spawned, ..
                },
                ..
            } => Some(*commands_spawned),
            _ => None,
        }),
        cache,
        to_float: |amount| amount as f64,
        to_string: |amount| amount.to_string(),
        average: |amount, n| amount as f64 / n as f64,
        average_to_float: std::convert::identity,
        average_to_string: |average| format!("{:.1}", average),
    })
    .width(Fill)
    .height(Fill)
    .into()
}

pub fn subscriptions_alive<'a, Message>(
    timeline: &'a Timeline,
    playhead: timeline::Index,
    cache: &'a canvas::Cache,
) -> Element<'a, Message>
where
    Message: 'a,
{
    canvas(BarChart {
        datapoints: timeline.seek(playhead).filter_map(|event| match event {
            beacon::Event::SubscriptionsTracked { amount_alive, .. } => Some(*amount_alive),
            _ => None,
        }),
        cache,
        to_float: |amount| amount as f64,
        to_string: |amount| amount.to_string(),
        average: |amount, n| amount as f64 / n as f64,
        average_to_float: std::convert::identity,
        average_to_string: |average| format!("{:.1}", average),
    })
    .width(Fill)
    .height(Fill)
    .into()
}

pub fn message_rate<'a, Message>(
    timeline: &'a Timeline,
    playhead: timeline::Index,
    cache: &'a canvas::Cache,
) -> Element<'a, Message>
where
    Message: 'a,
{
    let updates_per_second = {
        let mut updates = timeline.seek(playhead).filter_map(|event| match event {
            beacon::Event::SpanFinished { at, span, .. } if span.stage() == span::Stage::Update => {
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

    canvas(BarChart {
        datapoints: updates_per_second,
        cache,
        to_float: |amount| amount as f64,
        to_string: |amount| format!("{amount} msg/s"),
        average: |amount, n| amount as f64 / n as f64,
        average_to_float: std::convert::identity,
        average_to_string: |average| format!("{:.1} msg/s", average),
    })
    .width(Fill)
    .height(Fill)
    .into()
}

struct BarChart<'a, I, T, A>
where
    I: Iterator<Item = T>,
{
    datapoints: I,
    cache: &'a canvas::Cache,
    to_float: fn(T) -> f64,
    to_string: fn(T) -> String,
    average: fn(T, u32) -> A,
    average_to_float: fn(A) -> f64,
    average_to_string: fn(A) -> String,
}

impl<'a, Message, I, T, A> canvas::Program<Message> for BarChart<'a, I, T, A>
where
    I: Iterator<Item = T> + Clone + 'a,
    T: Ord + Copy + std::iter::Sum,
    A: Copy,
{
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        // TODO: Zoom
        const BAR_WIDTH: f32 = 10.0;

        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let bounds = frame.size();
            let palette = theme.extended_palette();

            let amount = (bounds.width / BAR_WIDTH).ceil() as usize;

            let Some(max) = self.datapoints.clone().take(amount).max() else {
                return;
            };

            let average = {
                let mut n = 0;

                let sum = self
                    .datapoints
                    .clone()
                    .take(amount * 3)
                    .map(|datapoint| {
                        n += 1;
                        datapoint
                    })
                    .sum::<T>();

                (self.average)(sum, n)
            };

            let average_value = (self.average_to_float)(average);
            let average_pixels = f64::from(bounds.height) / (2.0 * average_value);
            let max_pixels = f64::from(bounds.height) / (self.to_float)(max);

            let pixels_per_unit = average_pixels.min(max_pixels);

            for (i, datapoint) in self.datapoints.clone().take(amount).enumerate() {
                let value = (self.to_float)(datapoint);
                let bar_height = (value * pixels_per_unit) as f32;

                frame.fill_rectangle(
                    Point::new(
                        bounds.width - BAR_WIDTH * (i + 1) as f32,
                        bounds.height - bar_height,
                    ),
                    Size::new(BAR_WIDTH, bar_height),
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
                content: format!("Average: {}", (self.average_to_string)(average)),
                position: Point::new(10.0, 0.0),
                color: palette.background.base.text,
                size: Pixels(14.0),
                font: Font::MONOSPACE,
                ..canvas::Text::default()
            });

            frame.fill_text(canvas::Text {
                content: format!("Maximum: {}", (self.to_string)(max)),
                position: Point::new(10.0, 18.0),
                color: palette.background.base.text,
                size: Pixels(14.0),
                font: Font::MONOSPACE,
                ..canvas::Text::default()
            });
        });

        vec![geometry]
    }
}
