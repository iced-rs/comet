use crate::beacon::Event;
use crate::beacon::span;
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
            Event::SpanFinished { span, .. } => match span.stage() {
                span::Stage::Update => {
                    self.update.clear();
                }
                span::Stage::View(_id) => {
                    self.view.clear();
                }
                span::Stage::Layout(_id) => {
                    self.layout.clear();
                }
                span::Stage::Interact(_id) => {
                    self.interact.clear();
                }
                span::Stage::Draw(_id) => {
                    self.draw.clear();
                }
                span::Stage::Present(_id) => {
                    self.present.clear();
                }
                span::Stage::Boot | span::Stage::Custom(_, _) => {}
            },
            Event::ThemeChanged { .. } => {
                self.invalidate();
            }
            _ => {}
        }
    }

    pub fn view<'a, Message: 'a>(
        &'a self,
        timeline: &'a Timeline,
        playhead: timeline::Playhead,
    ) -> Element<'a, Message> {
        let update = (&chart::Stage::Update, &self.update);
        let view = (&chart::Stage::View, &self.view);
        let layout = (&chart::Stage::Layout, &self.layout);
        let interact = (&chart::Stage::Interact, &self.interact);
        let draw = (&chart::Stage::Draw, &self.draw);
        let present = (&chart::Stage::Present, &self.present);

        column(
            [[update, view], [layout, interact], [draw, present]].map(|charts| {
                row(charts.iter().map(|(stage, cache)| {
                    card(
                        stage.to_string(),
                        chart::performance(timeline, playhead, cache, stage),
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
