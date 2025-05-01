// Generated automatically by iced_fontello at build time.
// Do not edit manually. Source: ../fonts/comet-icons.toml
// 335bfeb1f3396725729a7457fc51d717a4a0ca0cecd15611b613d87858810299
use iced::widget::{text, Text};
use iced::Font;

pub const FONT: &[u8] = include_bytes!("../fonts/comet-icons.ttf");

pub fn time_travel<'a>() -> Text<'a> {
    icon("\u{E771}")
}

fn icon(codepoint: &str) -> Text<'_> {
    text(codepoint).font(Font::with_name("comet-icons"))
}
