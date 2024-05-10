use crate::beacon;
use crate::beacon::span;
use crate::timeline::{self, Timeline};

use iced::alignment;
use iced::mouse;
use iced::time::Duration;
use iced::widget::canvas;
use iced::{Element, Font, Length, Pixels, Point, Rectangle, Renderer, Size, Theme};

#[derive(Debug)]
pub enum Module {
    PerformanceChart {
        stage: span::Stage,
        cache: canvas::Cache,
    },
}

impl Module {
    pub fn performance_chart(stage: span::Stage) -> Self {
        Self::PerformanceChart {
            stage,
            cache: canvas::Cache::new(),
        }
    }

    pub fn title(&self) -> String {
        match self {
            Self::PerformanceChart { stage, .. } => stage.to_string(),
        }
    }

    pub fn invalidate(&mut self) {
        match self {
            Self::PerformanceChart { cache, .. } => {
                cache.clear();
            }
        }
    }

    pub fn invalidate_by(&mut self, event: &beacon::Event) {
        let should_invalidate = match (&self, event) {
            (Self::PerformanceChart { stage, .. }, beacon::Event::SpanFinished { span, .. }) => {
                &span.stage() == stage
            }
            (Self::PerformanceChart { .. }, beacon::Event::ThemeChanged { .. }) => true,
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
        }
    }
}

struct PerformanceChart<'a> {
    timeline: &'a Timeline,
    playhead: timeline::Index,
    cache: &'a canvas::Cache,
    stage: span::Stage,
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
            let bounds = frame.size();
            let palette = theme.extended_palette();

            let amount = (bounds.width / BAR_WIDTH).ceil() as usize + 1;
            let timeframes = self.timeline.timeframes(self.playhead, &self.stage);

            let Some(max) = timeframes
                .clone()
                .take(amount)
                .map(|timeframe| timeframe.duration)
                .max()
            else {
                return;
            };

            let average: Duration = {
                let mut n = 0;

                timeframes
                    .clone()
                    .take(amount * 3)
                    .map(|timeframe| {
                        n += 1;
                        timeframe.duration
                    })
                    .sum::<Duration>()
                    / n as u32
            };

            let average_pixels = f64::from(bounds.height) / (2.0 * average.as_nanos() as f64);
            let max_pixels = f64::from(bounds.height) / max.as_nanos() as f64;

            let pixels_per_nanosecond = average_pixels.min(max_pixels);

            for (i, timeframe) in timeframes.take(amount).enumerate() {
                let timeframe_nanos = timeframe.duration.as_nanos() as f64;
                let bar_height = (timeframe_nanos * pixels_per_nanosecond) as f32;

                frame.fill_rectangle(
                    Point::new(
                        bounds.width - BAR_WIDTH * i as f32,
                        bounds.height - bar_height,
                    ),
                    Size::new(BAR_WIDTH, bar_height),
                    if timeframe_nanos < average.as_nanos() as f64 * 0.75 {
                        palette.success.base.color
                    } else if timeframe_nanos > average.as_nanos() as f64 * 1.5 {
                        palette.danger.base.color
                    } else {
                        palette.background.strong.color
                    },
                )
            }

            frame.fill_text(canvas::Text {
                content: format!("Average: {average:?}"),
                position: Point::new(4.0, 4.0),
                color: palette.background.base.text,
                size: Pixels(14.0),
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Top,
                font: Font::MONOSPACE,
                ..canvas::Text::default()
            });

            frame.fill_text(canvas::Text {
                content: format!("Maximum: {max:?}"),
                position: Point::new(4.0, 22.0),
                color: palette.background.base.text,
                size: Pixels(14.0),
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Top,
                font: Font::MONOSPACE,
                ..canvas::Text::default()
            });
        });

        vec![geometry]
    }
}
