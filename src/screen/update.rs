use crate::beacon::{Event, Span};
use crate::chart;
use crate::timeline::{self, Timeline};
use crate::widget::card;

use iced::padding;
use iced::widget::{column, container, row, scrollable, text};
use iced::{Element, Fill, Font};

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
            Event::SubscriptionsTracked { .. } => {
                self.subscriptions_alive.clear();
            }
            Event::SpanFinished {
                span: Span::Update { .. },
                ..
            } => {
                self.update.clear();
                self.tasks_spawned.clear();
                self.message_rate.clear();
            }
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
        let update = chart::performance(timeline, playhead, &self.update, &chart::Stage::Update);
        let tasks_spawned = chart::tasks_spawned(timeline, playhead, &self.tasks_spawned);
        let subscriptions_alive =
            chart::subscriptions_alive(timeline, playhead, &self.subscriptions_alive);
        let message_rate = chart::message_rate(timeline, playhead, &self.message_rate);

        let message_log = container(
            scrollable(
                column({
                    let messages: Vec<_> = timeline
                        .seek(playhead)
                        .filter_map(|event| match event {
                            Event::SpanFinished {
                                span: Span::Update { message, .. },
                                ..
                            } => Some(message),
                            _ => None,
                        })
                        .take(20)
                        .map(|message| text(message).font(Font::MONOSPACE).size(10).into())
                        .collect();

                    messages.into_iter().rev()
                })
                .spacing(5),
            )
            .width(Fill)
            .height(Fill)
            .anchor_bottom(),
        )
        .padding(padding::all(10).top(0));

        column![
            card("Update", update),
            row![
                card("Tasks Spawned", tasks_spawned),
                card("Subscriptions Alive", subscriptions_alive)
            ]
            .spacing(10),
            row![
                card("Message Rate", message_rate),
                card("Message Log", message_log)
            ]
            .spacing(10)
        ]
        .spacing(10)
        .into()
    }
}
