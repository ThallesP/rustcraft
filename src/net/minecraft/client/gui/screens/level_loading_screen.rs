//! net/minecraft/client/gui/screens/LevelLoadingScreen.java
//!
//! The screen while the level loads, which closes once the level is ready (`LevelLoadTracker`'s,
//! which the game's listener ticks: the client loaded). Not ported yet: what it draws (the
//! progress, the chunks' statuses, the nether portal's or the dirt's background).

use std::any::Any;
use std::rc::Rc;

use crate::net::minecraft::client::gui::gui_graphics_extractor::GuiGraphicsExtractor;
use crate::net::minecraft::client::gui::screens::screen::{Screen, ScreenClass};
use crate::net::minecraft::client::minecraft::Minecraft;
use crate::net::minecraft::network::chat::component::Component;

pub struct LevelLoadingScreen {
    screen: Screen,
}

impl LevelLoadingScreen {
    pub fn new(minecraft: &Rc<Minecraft>) -> LevelLoadingScreen {
        LevelLoadingScreen {
            screen: Screen::new(minecraft, Component::empty().into()),
        }
    }
}

impl ScreenClass for LevelLoadingScreen {
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

    /// `tick`: closed once the level is ready.
    fn tick(&mut self) {
        let minecraft = self.screen.minecraft();
        let ready = minecraft
            .get_connection()
            .is_some_and(|connection| connection.borrow().has_client_loaded());
        if ready {
            self.on_close();
        }
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
