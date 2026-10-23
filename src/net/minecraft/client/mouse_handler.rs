//! net/minecraft/client/MouseHandler.java
//!
//! The mouse's turning of the player: movement accumulated between frames turns the player at
//! the default sensitivity (`options.sensitivity()`, 0.5) once a frame
//! (`handleAccumulatedMovement`), while the mouse is grabbed, which a click on the window does
//! and Escape undoes. Not ported yet: the buttons' bindings (attacking, using, picking), the
//! scroll wheel, the smooth camera and the spyglass's slower turning, inverted axes, and screens'
//! mouse handling.

use crate::net::minecraft::client::minecraft::Minecraft;

#[derive(Default)]
pub struct MouseHandler {
    accumulated_dx: f64,
    accumulated_dy: f64,
    mouse_grabbed: bool,
}

impl MouseHandler {
    const SENSITIVITY: f64 = 0.5;

    pub fn new() -> MouseHandler {
        MouseHandler::default()
    }

    /// `onMove`, with SDL's relative motion while grabbed.
    pub fn on_move(&mut self, x_relative: f64, y_relative: f64) {
        if self.mouse_grabbed {
            self.accumulated_dx += x_relative;
            self.accumulated_dy += y_relative;
        }
    }

    pub fn is_mouse_grabbed(&self) -> bool {
        self.mouse_grabbed
    }

    pub fn set_mouse_grabbed(&mut self, grabbed: bool) {
        self.mouse_grabbed = grabbed;
        self.accumulated_dx = 0.0;
        self.accumulated_dy = 0.0;
    }

    /// The movement since the last frame turns the player.
    pub fn handle_accumulated_movement(&mut self, minecraft: &Minecraft) {
        if self.mouse_grabbed {
            self.turn_player(minecraft);
        }
        self.accumulated_dx = 0.0;
        self.accumulated_dy = 0.0;
    }

    fn turn_player(&self, minecraft: &Minecraft) {
        let ss = MouseHandler::SENSITIVITY * f64::from(0.6f32) + f64::from(0.2f32);
        let sensitivity_mod = ss * ss * ss;
        let sens = sensitivity_mod * 8.0;
        let xo = self.accumulated_dx * sens;
        let yo = self.accumulated_dy * sens;
        if let Some(player) = minecraft.player.borrow_mut().as_mut() {
            player.turn(xo, yo);
        }
    }
}
