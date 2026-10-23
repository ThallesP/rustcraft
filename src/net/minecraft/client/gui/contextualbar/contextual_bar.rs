//! net/minecraft/client/gui/contextualbar/ContextualBar.java
//!
//! The bar above the hotbar. Java's interface is implemented by `EMPTY`'s anonymous class and the
//! three bars, so it's an enum of them; the locator's and the jumpable vehicle's come with
//! waypoints and vehicles. `extractExperienceLevel` comes with the font.

use crate::net::minecraft::client::gui::contextualbar::experience_bar::ExperienceBar;
use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;
use crate::net::minecraft::client::player::local_player::LocalPlayer;

pub enum ContextualBar {
    Empty,
    Experience(ExperienceBar),
}

impl ContextualBar {
    pub const WIDTH: i32 = 182;
    pub const HEIGHT: i32 = 5;
    pub const MARGIN_BOTTOM: i32 = 24;

    /// `left(Window)`.
    pub fn left(gui_scaled_width: i32) -> i32 {
        (gui_scaled_width - 182) / 2
    }

    /// `top(Window)`.
    pub fn top(gui_scaled_height: i32) -> i32 {
        gui_scaled_height - 24 - 5
    }

    pub fn extract_background(&self, graphics: &mut GuiGraphicsExtractor, player: &LocalPlayer) {
        match self {
            ContextualBar::Empty => {}
            ContextualBar::Experience(bar) => bar.extract_background(graphics, player),
        }
    }

    pub fn extract_render_state(&self, graphics: &mut GuiGraphicsExtractor, player: &LocalPlayer) {
        match self {
            ContextualBar::Empty => {}
            ContextualBar::Experience(bar) => bar.extract_render_state(graphics, player),
        }
    }
}
