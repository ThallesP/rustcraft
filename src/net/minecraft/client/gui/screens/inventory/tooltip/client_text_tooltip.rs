//! net/minecraft/client/gui/screens/inventory/tooltip/ClientTextTooltip.java

use std::rc::Rc;

use crate::net::minecraft::client::gui::font::Font;
use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;
use crate::net::minecraft::client::gui::screens::inventory::tooltip::client_tooltip_component::ClientTooltipComponent;
use crate::net::minecraft::util::formatted_char_sequence::FormattedCharSequence;

pub struct ClientTextTooltip {
    text: FormattedCharSequence,
}

impl ClientTextTooltip {
    pub fn new(text: FormattedCharSequence) -> ClientTextTooltip {
        ClientTextTooltip { text }
    }
}

impl ClientTooltipComponent for ClientTextTooltip {
    fn get_width(&self, font: &Font) -> i32 {
        font.width_sequence(&self.text)
    }

    fn get_height(&self, _font: &Font) -> i32 {
        10
    }

    fn extract_text(&self, graphics: &mut GuiGraphicsExtractor, font: &Rc<Font>, x: i32, y: i32) {
        graphics.text_sequence(font, &self.text, x, y, -1, true);
    }
}
