//! net/minecraft/client/gui/Gui.java
//!
//! The HUD, the screen over it, and later the overlay and the toasts. The screen is the game's
//! (`Minecraft::set_screen`, Java's `setScreen`), which ticks and draws it after the HUD, outside
//! this cell, since a screen reaches the game (the chat screen the HUD's chat). `GuiRenderState` is
//! the game renderer's (`GameRenderState.guiRenderState`), which the extract borrows through its
//! `GuiGraphicsExtractor`.
//!
//! Not ported yet: the chat listener's delay, the toasts, the splash manager, the screens'
//! narration and the saving indicator's text (a singleplayer server's).

use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;
use crate::net::minecraft::client::gui::hud::{Hud, HudContext};
use crate::net::minecraft::client::gui::screens::screen::ScreenRef;

#[derive(Default)]
pub struct Gui {
    pub hud: Hud,
    /// `screen`.
    pub screen: Option<ScreenRef>,
    /// Whether the screen is the chat screen (`ChatComponent.isChatFocused`), kept as it's set.
    pub chat_screen_shown: bool,
}

impl Gui {
    pub fn new() -> Gui {
        Gui::default()
    }

    /// `tick()`: the HUD's tick (the chat listener and the screen's tick come with them).
    pub fn tick(&mut self, pause: bool) {
        self.hud.tick(pause);
    }

    /// `extractRenderState`: the HUD when the level renders, under the screen. The render state was
    /// reset by whoever made `graphics`.
    pub fn extract_render_state(
        &mut self,
        graphics: &mut GuiGraphicsExtractor,
        hud_context: Option<&HudContext>,
    ) {
        if let Some(hud_context) = hud_context {
            self.hud.extract_render_state(graphics, hud_context);
        }
    }
}
