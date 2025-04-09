use crate::beacon::Event;
use crate::beacon::span;
use crate::chart;
use crate::timeline::{self, Timeline};
use crate::widget::card;

use iced::Element;
use iced::widget::{column, row};

#[derive(Debug, Default)]
pub struct Present {
    present: chart::Cache,
    quad: Cache,
    triangle: Option<Cache>,
    shader: Option<Cache>,
    image: Option<Cache>,
    text: Cache,
}

impl Present {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn invalidate(&mut self) {
        self.present.clear();
        self.quad.clear();
        self.text.clear();

        if let Some(triangle) = &mut self.triangle {
            triangle.clear();
        }

        if let Some(shader) = &mut self.shader {
            shader.clear();
        }

        if let Some(image) = &mut self.image {
            image.clear();
        }
    }

    pub fn invalidate_by(&mut self, event: &Event) {
        match event {
            Event::SpanFinished { span, .. } => match span.stage() {
                span::Stage::Present(_id) => {
                    self.present.clear();
                }
                span::Stage::Prepare(primitive) | span::Stage::Render(primitive) => {
                    let cache = match primitive {
                        span::Primitive::Quad => &mut self.quad,
                        span::Primitive::Triangle => {
                            if self.triangle.is_none() {
                                self.triangle = Some(Cache::default());
                            }

                            self.triangle.as_mut().unwrap()
                        }
                        span::Primitive::Shader => {
                            if self.shader.is_none() {
                                self.shader = Some(Cache::default());
                            }

                            self.shader.as_mut().unwrap()
                        }
                        span::Primitive::Image => {
                            if self.image.is_none() {
                                self.image = Some(Cache::default());
                            }

                            self.image.as_mut().unwrap()
                        }
                        span::Primitive::Text => &mut self.text,
                    };

                    if matches!(span.stage(), span::Stage::Prepare(_)) {
                        cache.prepare.clear();
                    } else {
                        cache.render.clear();
                    }
                }
                _ => {}
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
        let primitives = [
            Some((span::Primitive::Quad, &self.quad)),
            self.triangle
                .as_ref()
                .map(|cache| (span::Primitive::Triangle, cache)),
            self.shader
                .as_ref()
                .map(|cache| (span::Primitive::Shader, cache)),
            self.image
                .as_ref()
                .map(|cache| (span::Primitive::Image, cache)),
            Some((span::Primitive::Text, &self.text)),
        ]
        .into_iter()
        .flatten()
        .map(|(primitive, cache)| {
            let prepare_stage = chart::Stage::Prepare(primitive);
            let render_stage = chart::Stage::Render(primitive);

            row![
                card(
                    prepare_stage.to_string(),
                    chart::performance(timeline, playhead, &cache.prepare, &prepare_stage)
                ),
                card(
                    render_stage.to_string(),
                    chart::performance(timeline, playhead, &cache.render, &render_stage)
                ),
            ]
            .spacing(10)
            .into()
        });

        column(primitives).spacing(10).into()
    }
}

#[derive(Debug, Default)]
struct Cache {
    prepare: chart::Cache,
    render: chart::Cache,
}

impl Cache {
    fn clear(&mut self) {
        self.prepare.clear();
        self.render.clear();
    }
}
