//! net/minecraft/client/gui/screens/DisconnectedScreen.java
//!
//! Where the game goes when a connection ends: its title and why. Not ported yet: what it draws
//! (the reason's lines, the button back to the server list or the title screen), which the title
//! screen comes with; a client with no screen to go back to stops here.

use std::any::Any;
use std::rc::Rc;

use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;
use crate::net::minecraft::client::gui::screens::screen::{Screen, ScreenClass};
use crate::net::minecraft::client::minecraft::Minecraft;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::disconnection_details::DisconnectionDetails;

pub struct DisconnectedScreen {
    screen: Screen,
    /// The title's translation key.
    pub title_key: String,
    pub details: DisconnectionDetails,
}

impl DisconnectedScreen {
    pub fn new(
        minecraft: &Rc<Minecraft>,
        title_key: &str,
        details: DisconnectionDetails,
    ) -> DisconnectedScreen {
        DisconnectedScreen {
            screen: Screen::new(minecraft, Component::translatable(title_key).into()),
            title_key: title_key.to_string(),
            details,
        }
    }
}

impl ScreenClass for DisconnectedScreen {
    fn screen(&self) -> &Screen {
        &self.screen
    }

    fn screen_mut(&mut self) -> &mut Screen {
        &mut self.screen
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn should_close_on_esc(&self) -> bool {
        false
    }

    fn extract_background(
        &mut self,
        _graphics: &mut GuiGraphicsExtractor,
        _x: i32,
        _y: i32,
        _a: f32,
    ) {
    }
}
