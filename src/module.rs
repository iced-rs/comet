use crate::sentinel;
use crate::sentinel::timing;
use crate::Timeline;

use iced::alignment;
use iced::mouse;
use iced::time::Duration;
use iced::widget::{canvas, container};
use iced::{Element, Font, Length, Pixels, Point, Rectangle, Renderer, Size, Theme};

#[derive(Debug)]
pub enum Module {
    PerformanceChart {
        stage: timing::Stage,
        cache: canvas::Cache,
    },
}

impl Module {
    pub fn performance_chart(stage: timing::Stage) -> Self {
        Self::PerformanceChart {
            stage,
            cache: canvas::Cache::new(),
        }
    }

    pub fn title(&self) -> String {
        match self {
            Self::PerformanceChart { stage, .. } => format!("Performance - {stage}"),
        }
    }

    pub fn invalidate(&mut self, event: &sentinel::Event) {
        match (self, event) {
            (Self::PerformanceChart { stage, cache }, sentinel::Event::TimingMeasured(timing))
                if &timing.stage == stage =>
            {
                cache.clear();
            }
            _ => {}
        }
    }

    pub fn view<'a, Message: 'a>(&'a self, timeline: &'a Timeline) -> Element<'a, Message> {
        match self {
            Self::PerformanceChart { stage, cache } => container(
                canvas(PerformanceChart {
                    timeline,
                    cache,
                    stage: stage.clone(),
                })
                .width(Length::Fill)
                .height(Length::Fill),
            )
            .padding(10)
            .into(),
        }
    }
}

struct PerformanceChart<'a> {
    timeline: &'a Timeline,
    cache: &'a canvas::Cache,
    stage: timing::Stage,
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
            let timings = self.timeline.timings(&self.stage).rev().take(amount);

            let Some(max) = timings.clone().map(|timing| timing.duration).max() else {
                return;
            };

            let average: Duration = timings
                .clone()
                .map(|timing| timing.duration)
                .sum::<Duration>()
                / amount as u32;

            let average_pixels = f64::from(bounds.height) / (2.0 * average.as_nanos() as f64);
            let max_pixels = f64::from(bounds.height) / max.as_nanos() as f64;

            let pixels_per_nanosecond = average_pixels.min(max_pixels);

            for (i, timing) in timings.enumerate() {
                let timing_nanos = timing.duration.as_nanos() as f64;
                let bar_height = (timing_nanos * pixels_per_nanosecond) as f32;

                frame.fill_rectangle(
                    Point::new(
                        bounds.width - BAR_WIDTH * i as f32,
                        bounds.height - bar_height,
                    ),
                    Size::new(BAR_WIDTH, bar_height),
                    if timing_nanos < average.as_nanos() as f64 * 0.75 {
                        palette.success.base.color
                    } else if timing_nanos > average.as_nanos() as f64 * 1.5 {
                        palette.danger.base.color
                    } else {
                        palette.background.strong.color
                    },
                )
            }

            frame.fill_text(canvas::Text {
                content: format!("Average: {average:?}"),
                position: Point::ORIGIN,
                color: palette.background.strong.text,
                size: Pixels(16.0),
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Top,
                font: Font::MONOSPACE,
                ..canvas::Text::default()
            });

            frame.fill_text(canvas::Text {
                content: format!("Maximum: {max:?}"),
                position: Point::new(0.0, 20.0),
                color: palette.background.strong.text,
                size: Pixels(16.0),
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Top,
                font: Font::MONOSPACE,
                ..canvas::Text::default()
            });
        });

        vec![geometry]
    }
}
