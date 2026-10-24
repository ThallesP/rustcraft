//! net/minecraft/client/gui/components/AbstractStringWidget.java
//!
//! A widget that's text in the font: its lines drawn through the widget's text collector, with
//! tooltips of their hover events when hovered. A click on a style with a click event
//! (`componentClickHandler`, `ClickableStyleFinder`) isn't ported yet. The class's state is
//! `StringWidget`'s, the only one ported; this is its shared code.

use std::rc::Rc;

use crate::net::minecraft::client::gui::font::Font;
use crate::net::minecraft::client::gui::gui_graphics_extractor::HoveredTextEffects;

/// `extractWidgetRenderState`'s effects: tooltips while hovered.
pub fn hovered_text_effects(
    is_hovered: bool,
    has_component_click_handler: bool,
) -> HoveredTextEffects {
    if is_hovered {
        if has_component_click_handler {
            HoveredTextEffects::TOOLTIP_AND_CURSOR
        } else {
            HoveredTextEffects::TOOLTIP_ONLY
        }
    } else {
        HoveredTextEffects::NONE
    }
}

/// The font a string widget draws in (`getFont`).
pub type WidgetFont = Rc<Font>;
