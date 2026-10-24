//! net/minecraft/client/gui/components/Tooltip.java
//!
//! A widget's tooltip text, split at 170 pixels the first time it's shown (and again for another
//! language, which the port's one language never is). Not ported yet: its image component, and
//! narration.

use crate::net::minecraft::client::gui::font::Font;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::formatted_char_sequence::FormattedCharSequence;

#[derive(Clone, Debug)]
pub struct Tooltip {
    message: Component,
    cached_tooltip: Option<Vec<FormattedCharSequence>>,
    style: Option<Identifier>,
    narration: Option<Component>,
}

impl Tooltip {
    const MAX_WIDTH: i32 = 170;

    pub fn create(message: Component) -> Tooltip {
        Tooltip {
            narration: Some(message.clone()),
            message,
            cached_tooltip: None,
            style: None,
        }
    }

    /// `create(Component, Component)`.
    pub fn create_with_narration(message: Component, narration: Option<Component>) -> Tooltip {
        Tooltip {
            message,
            cached_tooltip: None,
            style: None,
            narration,
        }
    }

    pub fn style(&self) -> Option<&Identifier> {
        self.style.as_ref()
    }

    pub fn message(&self) -> &Component {
        &self.message
    }

    pub fn narration(&self) -> Option<&Component> {
        self.narration.as_ref()
    }

    /// `toCharSequence`: the lines, split once.
    pub fn to_char_sequence(&mut self, font: &Font) -> Vec<FormattedCharSequence> {
        self.cached_tooltip
            .get_or_insert_with(|| Tooltip::split_tooltip(font, &self.message))
            .clone()
    }

    pub fn split_tooltip(font: &Font, message: &Component) -> Vec<FormattedCharSequence> {
        font.split(message, Tooltip::MAX_WIDTH)
    }
}
