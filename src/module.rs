use crate::sentinel;
use crate::sentinel::timing;
use crate::Timeline;

use iced::mouse;
use iced::widget::{canvas, container};
use iced::{Element, Length, Point, Rectangle, Renderer, Size, Theme};

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
        const BAR_WIDTH: f32 = 22.0;

        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let bounds = frame.size();
            let palette = theme.extended_palette();

            let amount = (bounds.width / BAR_WIDTH).ceil() as usize + 1;
            let timings = self.timeline.timings(&self.stage).rev().take(amount);

            let Some(max_duration) = timings.clone().map(|timing| timing.duration).max() else {
                return;
            };

            let pixels_per_nanosecond = f64::from(bounds.height) / max_duration.as_nanos() as f64;

            for (i, timing) in timings.enumerate() {
                let bar_height = (timing.duration.as_nanos() as f64 * pixels_per_nanosecond) as f32;

                frame.fill_rectangle(
                    Point::new(
                        bounds.width - BAR_WIDTH * i as f32 + 1.0,
                        bounds.height - bar_height,
                    ),
                    Size::new(BAR_WIDTH - 2.0, bar_height),
                    palette.background.base.text,
                )
            }
        });

        vec![geometry]
    }
}
