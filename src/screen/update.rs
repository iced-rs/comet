use crate::beacon::{Event, Span};
use crate::chart;
use crate::timeline::{self, Timeline};
use crate::widget::card;

use iced::padding;
use iced::widget::{column, container, row, scrollable, text};
use iced::{Element, Fill, FillPortion};

#[derive(Debug, Default)]
pub struct Update {
    update: chart::Cache,
    tasks_spawned: chart::Cache,
    subscriptions_alive: chart::Cache,
    message_rate: chart::Cache,
}

impl Update {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn invalidate(&mut self) {
        self.update.clear();
        self.tasks_spawned.clear();
        self.subscriptions_alive.clear();
        self.message_rate.clear();
    }

    pub fn invalidate_by(&mut self, event: &Event) {
        match event {
            Event::SpanFinished {
                span: Span::Update { .. },
                ..
            } => {
                self.update.clear();
                self.tasks_spawned.clear();
                self.message_rate.clear();
                self.subscriptions_alive.clear();
            }
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
        let update = chart::updates(timeline, playhead, &self.update, zoom);
        let tasks_spawned = chart::tasks_spawned(timeline, playhead, &self.tasks_spawned, zoom);
        let subscriptions_alive =
            chart::subscriptions_alive(timeline, playhead, &self.subscriptions_alive, zoom);
        let message_rate = chart::message_rate(timeline, playhead, &self.message_rate, zoom);

        let last_message = container(
            scrollable({
                let message = timeline
                    .updates(playhead)
                    .next()
                    .map(|update| update.message)
                    .unwrap_or_default();

                text(message).size(10)
            })
            .width(Fill)
            .height(Fill)
            .spacing(10),
        )
        .padding(padding::all(10).top(0));

        row![
            column![
                container(card("Update", update)).height(FillPortion(2)),
                card("Tasks Spawned", tasks_spawned),
                card("Subscriptions Alive", subscriptions_alive),
            ]
            .width(FillPortion(2))
            .spacing(10),
            column![
                container(card("Last Message", last_message)).height(FillPortion(2)),
                card("Message Rate", message_rate),
            ]
            .spacing(10)
        ]
        .spacing(10)
        .into()
    }
}
