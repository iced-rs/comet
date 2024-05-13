use iced::advanced;
use iced::widget::text;

mod animated_text;

use animated_text::AnimatedText;

pub fn animated_text<'a, Theme, Renderer>(
    fragment: impl text::IntoFragment<'a>,
) -> AnimatedText<'a, Theme, Renderer>
where
    Theme: text::Catalog,
    Renderer: advanced::text::Renderer,
{
    AnimatedText::new(fragment)
}
