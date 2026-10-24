//! net/minecraft/client/gui/screens/inventory/tooltip/ClientTooltipComponent.java
//!
//! A line of a tooltip. Only text lines are ported (`create(FormattedCharSequence)`); the image
//! components (bundles, maps) and `create(TooltipComponent)` come with items.

use std::rc::Rc;

use crate::net::minecraft::client::gui::font::Font;
use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;
use crate::net::minecraft::client::gui::screens::inventory::tooltip::client_text_tooltip::ClientTextTooltip;
use crate::net::minecraft::util::formatted_char_sequence::FormattedCharSequence;

pub trait ClientTooltipComponent {
    fn get_height(&self, font: &Font) -> i32;

    fn get_width(&self, font: &Font) -> i32;

    fn extract_text(
        &self,
        _graphics: &mut GuiGraphicsExtractor,
        _font: &Rc<Font>,
        _x: i32,
        _y: i32,
    ) {
    }

    #[allow(clippy::too_many_arguments)]
    fn extract_image(
        &self,
        _font: &Rc<Font>,
        _x: i32,
        _y: i32,
        _w: i32,
        _h: i32,
        _graphics: &mut GuiGraphicsExtractor,
    ) {
    }
}

/// `create(FormattedCharSequence)`.
pub fn create(text: FormattedCharSequence) -> Box<dyn ClientTooltipComponent> {
    Box::new(ClientTextTooltip::new(text))
}
