use crate::beacon::span::present;
use crate::beacon::{Event, Span};
use crate::chart;
use crate::timeline::{self, Timeline};
use crate::widget::card;

use iced::Element;
use iced::widget::{column, row};

#[derive(Debug, Default)]
pub struct Present {
    present: chart::Cache,
    layers: chart::Cache,
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
        self.layers.clear();
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
            Event::SpanFinished {
                span: Span::Present { prepare, .. },
                ..
            } => {
                self.present.clear();
                self.layers.clear();

                if self.triangle.is_none() && !prepare.triangles.is_zero() {
                    self.triangle = Some(Cache::default());
                }

                if self.shader.is_none() && !prepare.shaders.is_zero() {
                    self.shader = Some(Cache::default());
                }

                if self.image.is_none() && !prepare.images.is_zero() {
                    self.image = Some(Cache::default());
                }

                self.quad.clear();
                self.triangle.as_ref().map(Cache::clear);
                self.shader.as_ref().map(Cache::clear);
                self.image.as_ref().map(Cache::clear);
                self.text.clear();
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
        let primitives = [
            Some((present::Primitive::Quad, &self.quad)),
            self.triangle
                .as_ref()
                .map(|cache| (present::Primitive::Triangle, cache)),
            self.shader
                .as_ref()
                .map(|cache| (present::Primitive::Shader, cache)),
            self.image
                .as_ref()
                .map(|cache| (present::Primitive::Image, cache)),
            Some((present::Primitive::Text, &self.text)),
        ]
        .into_iter()
        .flatten()
        .map(|(primitive, cache)| {
            let prepare_stage = chart::Stage::Prepare(primitive);
            let render_stage = chart::Stage::Render(primitive);

            row![
                card(
                    prepare_stage.to_string(),
                    chart::performance(timeline, playhead, &cache.prepare, prepare_stage, zoom)
                ),
                card(
                    render_stage.to_string(),
                    chart::performance(timeline, playhead, &cache.render, render_stage, zoom)
                ),
            ]
            .spacing(10)
            .into()
        });

        let charts = [row![
            card(
                "Present",
                chart::performance(
                    timeline,
                    playhead,
                    &self.present,
                    chart::Stage::Present,
                    zoom,
                ),
            ),
            card(
                "Layers",
                chart::layers_rendered(timeline, playhead, &self.layers, zoom),
            ),
        ]
        .spacing(10)
        .into()]
        .into_iter()
        .chain(primitives);

        column(charts).spacing(10).into()
    }
}

#[derive(Debug, Default)]
struct Cache {
    prepare: chart::Cache,
    render: chart::Cache,
}

impl Cache {
    fn clear(&self) {
        self.prepare.clear();
        self.render.clear();
    }
}
