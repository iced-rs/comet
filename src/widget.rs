use iced::advanced;
use iced::widget::text;

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
