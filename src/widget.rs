use iced::advanced;
use iced::border;
use iced::widget::{column, container, text};
use iced::{Element, Font};

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
        container(text(title).font(Font::MONOSPACE)).padding(10),
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
