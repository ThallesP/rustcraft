//! net/minecraft/client/gui/Gui.java
//!
//! The HUD and, later, the screen over it, the overlay and the toasts. The port's screens are
//! still `Minecraft`'s stand-ins (the connect screen, the level's loading, the disconnected
//! screen), which draw nothing; the ones that draw come with their ports (the pause menu, chat,
//! the inventory). `GuiRenderState` is the game renderer's (`GameRenderState.guiRenderState`), which
//! the extract borrows through its `GuiGraphicsExtractor`.
//!
//! Not ported yet: the chat listener, the toasts, the splash manager, the screens' narration and
//! the saving indicator's text (a singleplayer server's).

use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;
use crate::net::minecraft::client::gui::hud::{Hud, HudContext};

#[derive(Default)]
pub struct Gui {
    pub hud: Hud,
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
