//! net/minecraft/client/gui/TextAlignment.java

use crate::net::minecraft::client::gui::font::Font;
use crate::net::minecraft::util::formatted_char_sequence::FormattedCharSequence;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAlignment {
    LEFT,
    CENTER,
    RIGHT,
}

impl TextAlignment {
    pub fn calculate_left(self, anchor: i32, width: i32) -> i32 {
        match self {
            TextAlignment::LEFT => anchor,
            TextAlignment::CENTER => anchor - width / 2,
            TextAlignment::RIGHT => anchor - width,
        }
    }

    /// `calculateLeft(int, Font, FormattedCharSequence)`: left alignment doesn't measure.
    pub fn calculate_left_text(
        self,
        anchor: i32,
        font: &Font,
        text: &FormattedCharSequence,
    ) -> i32 {
        match self {
            TextAlignment::LEFT => anchor,
            _ => self.calculate_left(anchor, font.width_sequence(text)),
        }
    }
}
