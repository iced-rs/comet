use crate::beacon;
use crate::beacon::span;
use crate::chart;
use crate::timeline::{self, Timeline};
use crate::widget::card;

use iced::widget::{center, column, container, rich_text, span};
use iced::{Color, Element, Fill, Font};

use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Custom {
    timings: BTreeMap<String, chart::Cache>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Browse(Link),
    Chart(chart::Interaction),
}

#[derive(Debug, Clone)]
pub enum Event {
    ChartInteracted(chart::Interaction),
}

#[derive(Debug, Clone)]
pub enum Link {
    Time,
    TimeWith,
    Debug,
}

impl Custom {
    pub fn new(timeline: &Timeline, playhead: timeline::Playhead) -> Self {
        let timings = timeline
            .seek(playhead)
            .filter_map(|event| {
                if let beacon::Event::SpanFinished {
                    span: span::Span::Custom { name },
                    ..
                } = event
                {
                    Some((name.to_owned(), chart::Cache::default()))
                } else {
                    None
                }
            })
            .collect();

        Self { timings }
    }

    pub fn invalidate(&mut self) {
        for cache in self.timings.values_mut() {
            cache.clear();
        }
    }

    pub fn invalidate_by(&mut self, event: &beacon::Event) {
        match event {
            beacon::Event::SpanFinished {
                span: span::Span::Custom { name },
                ..
            } => {
                self.timings.entry(name.to_owned()).or_default().clear();
            }
            beacon::Event::ThemeChanged { .. } => {
                self.invalidate();
            }
            _ => {}
        }
    }

    pub fn update(&mut self, message: Message) -> Option<Event> {
        match message {
            Message::Browse(link) => {
                let path = match link {
                    Link::Time => "debug/fn.time.html",
                    Link::TimeWith => "debug/fn.time_with.html",
                    Link::Debug => "debug/index.html",
                };

                let is_prerelease = !env!("CARGO_PKG_VERSION_PRE").is_empty();

                let docs_host = if is_prerelease {
                    "https://docs.iced.rs/iced/"
                } else {
                    concat!("https://docs.rs/iced/", env!("CARGO_PKG_VERSION"), "/iced/")
                };

                let _ = open::that_in_background(format!("{docs_host}{path}"));

                None
            }
            Message::Chart(interaction) => Some(Event::ChartInteracted(interaction)),
        }
    }

    pub fn view<'a>(
        &'a self,
        timeline: &'a Timeline,
        playhead: timeline::Playhead,
        zoom: chart::Zoom,
    ) -> Element<'a, Message> {
        if self.timings.is_empty() {
            let code = |text| {
                span(text)
                    .font(Font::MONOSPACE)
                    .color(Color::WHITE)
                    .background(Color::BLACK)
                    .padding([0, 2])
            };

            return center(
                container(card(
                    "No custom timings have been received!",
                    container(
                        rich_text![
                            "You can use the ",
                            code("time").link(Link::Time),
                            " and ",
                            code("time_with").link(Link::TimeWith),
                            " functions in ",
                            code("iced::debug").link(Link::Debug),
                            " to see performance metrics here."
                        ]
                        .on_link_click(Message::Browse)
                        .size(14)
                        .width(Fill),
                    )
                    .padding(10),
                ))
                .max_width(600),
            )
            .into();
        }

        let charts = self.timings.iter().map(|(name, cache)| {
            card(
                name,
                chart::performance(
                    timeline,
                    playhead,
                    cache,
                    &chart::Stage::Custom(name.to_owned()), // TODO: Avoid allocation (?)
                    zoom,
                )
                .map(Message::Chart),
            )
        });

        column(charts).spacing(10).into()
    }
}
