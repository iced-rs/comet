use iced::border;
use iced::padding;
use iced::theme::palette;
use iced::widget::{column, container, horizontal_space, text, tooltip};
use iced::{Background, Color, Element, Font, Theme};

pub use iced_palace::widget::diffused_text;

pub fn card<'a, Message: 'a>(
    title: impl text::IntoFragment<'a>,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(column![
        container(diffused_text(title).font(Font::MONOSPACE)).padding(padding::all(10).bottom(5)),
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

pub fn tip<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    tip: impl text::IntoFragment<'a>,
    position: tooltip::Position,
) -> Element<'a, Message> {
    tooltip(
        content,
        container(text(tip).font(Font::MONOSPACE).size(8))
            .padding(5)
            .style(container::rounded_box),
        position,
    )
    .into()
}
