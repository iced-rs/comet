use iced::advanced;
use iced::border;
use iced::padding;
use iced::theme::palette;
use iced::widget::{column, container, horizontal_space, text};
use iced::{Background, Color, Element, Font, Theme};

mod diffused_text;

use diffused_text::DiffusedText;

pub fn diffused_text<'a, Theme, Renderer>(
    fragment: impl text::IntoFragment<'a>,
) -> DiffusedText<'a, Theme, Renderer>
where
    Theme: text::Catalog,
    Renderer: advanced::text::Renderer,
{
    DiffusedText::new(fragment)
}

pub fn card<'a, Message: 'a>(
    title: impl text::IntoFragment<'a>,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(column![
        container(text(title).font(Font::MONOSPACE)).padding(padding::all(10).bottom(5)),
        content.into()
    ])
    .style(|theme| {
        let style = container::bordered_box(theme);

        container::Style {
            border: border::rounded(border::top(5))
                .width(1)
                .color(theme.extended_palette().background.weak.color),
            ..style
        }
    })
    .into()
}

pub fn circle<'a, Message: 'a>(
    color: impl Fn(&palette::Extended) -> Color + 'a,
) -> Element<'a, Message> {
    container(horizontal_space())
        .width(8)
        .height(8)
        .style(move |theme: &Theme| container::Style {
            background: Some(Background::from(color(theme.extended_palette()))),
            border: border::rounded(4),
            ..container::Style::default()
        })
        .into()
}
