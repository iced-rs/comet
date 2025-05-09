use crate::beacon::{Event, Span};
use crate::chart;
use crate::timeline::{self, Timeline};
use crate::widget::card;

use iced::Element;
use iced::widget::{column, row};

#[derive(Debug, Default)]
pub struct Overview {
    update: chart::Cache,
    view: chart::Cache,
    layout: chart::Cache,
    interact: chart::Cache,
    draw: chart::Cache,
    present: chart::Cache,
}

impl Overview {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn invalidate(&mut self) {
        self.update.clear();
        self.view.clear();
        self.layout.clear();
        self.interact.clear();
        self.draw.clear();
        self.present.clear();
    }

    pub fn invalidate_by(&mut self, event: &Event) {
        match event {
            Event::SpanFinished { span, .. } => match span {
                Span::Update { .. } => {
                    self.update.clear();
                }
                Span::View { .. } => {
                    self.view.clear();
                }
                Span::Layout { .. } => {
                    self.layout.clear();
                }
                Span::Interact { .. } => {
                    self.interact.clear();
                }
                Span::Draw { .. } => {
                    self.draw.clear();
                }
                Span::Present { .. } => {
                    self.present.clear();
                }
                _ => {}
            },
            Event::ThemeChanged { .. } => {
                self.invalidate();
            }
            _ => {}
        }
    }

    pub fn view<'a>(
        &'a self,
        timeline: &'a Timeline,
        playhead: timeline::Playhead,
        zoom: chart::Zoom,
    ) -> Element<'a, chart::Interaction> {
        let update = (chart::Stage::Update, &self.update);
        let view = (chart::Stage::View, &self.view);
        let layout = (chart::Stage::Layout, &self.layout);
        let interact = (chart::Stage::Interact, &self.interact);
        let draw = (chart::Stage::Draw, &self.draw);
        let present = (chart::Stage::Present, &self.present);

        column(
            [[update, view], [layout, interact], [draw, present]].map(|charts| {
                row(charts.into_iter().map(|(stage, cache)| {
                    card(
                        stage.to_string(),
                        chart::performance(timeline, playhead, cache, stage, zoom),
                    )
                }))
                .spacing(10)
                .into()
            }),
        )
        .spacing(10)
        .into()
    }
}
