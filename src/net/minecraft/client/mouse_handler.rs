//! net/minecraft/client/MouseHandler.java
//!
//! The mouse's place in the window (`xpos`, `ypos`), which screens read in the GUI's scaled
//! coordinates, and its turning of the player: movement accumulated between frames turns the
//! player at the default sensitivity (`options.sensitivity()`, 0.5) once a frame
//! (`handleAccumulatedMovement`) while the mouse is grabbed, which a click on the window does and
//! a screen undoes, putting the mouse in the window's middle. Not ported yet: the buttons'
//! bindings (attacking, using, picking), the scroll wheel, dragging, double clicks, the smooth
//! camera and the spyglass's slower turning, inverted axes. `resyncMousePosition` asks SDL where
//! the mouse is in Java; here the place it was last moved to stays, which is what a mouse that
//! didn't move gives.

use crate::net::minecraft::client::minecraft::Minecraft;

#[derive(Default)]
pub struct MouseHandler {
    xpos: f64,
    ypos: f64,
    accumulated_dx: f64,
    accumulated_dy: f64,
    mouse_grabbed: bool,
    ignore_first_move: bool,
}

impl MouseHandler {
    const SENSITIVITY: f64 = 0.5;

    pub fn new() -> MouseHandler {
        MouseHandler::default()
    }

    /// `onMove`: the mouse at a place in the window, which moved by the relative amounts.
    pub fn on_move(&mut self, xpos: f64, ypos: f64, x_relative: f64, y_relative: f64) {
        if self.ignore_first_move {
            self.xpos = xpos;
            self.ypos = ypos;
            self.ignore_first_move = false;
        } else {
            if self.mouse_grabbed {
                self.accumulated_dx += x_relative;
                self.accumulated_dy += y_relative;
            } else {
                self.accumulated_dx += xpos - self.xpos;
                self.accumulated_dy += ypos - self.ypos;
            }
            self.xpos = xpos;
            self.ypos = ypos;
        }
    }

    /// `onMove` to a place, its movement relative to the last one, the way a scenario's test input
    /// moves it (`TestInput.setCursorPos`).
    pub fn on_move_to(&mut self, xpos: f64, ypos: f64) {
        let (dx, dy) = (xpos - self.xpos, ypos - self.ypos);
        self.on_move(xpos, ypos, dx, dy);
    }

    pub fn xpos(&self) -> f64 {
        self.xpos
    }

    pub fn ypos(&self) -> f64 {
        self.ypos
    }

    /// `getScaledXPos`: the mouse's place in the GUI's coordinates.
    pub fn get_scaled_x_pos(
        &self,
        (screen_width, _): (i32, i32),
        (gui_width, _): (i32, i32),
    ) -> f64 {
        self.xpos * f64::from(gui_width) / f64::from(screen_width)
    }

    pub fn get_scaled_y_pos(
        &self,
        (_, screen_height): (i32, i32),
        (_, gui_height): (i32, i32),
    ) -> f64 {
        self.ypos * f64::from(gui_height) / f64::from(screen_height)
    }

    pub fn is_mouse_grabbed(&self) -> bool {
        self.mouse_grabbed
    }

    pub fn has_accumulated_movement(&self) -> bool {
        self.accumulated_dx != 0.0 || self.accumulated_dy != 0.0
    }

    /// `grabMouse`'s own part: grabbed, in the window's middle, ignoring the next move.
    pub fn grab_mouse(&mut self, (width, height): (i32, i32)) {
        self.mouse_grabbed = true;
        self.xpos = f64::from(width) / 2.0;
        self.ypos = f64::from(height) / 2.0;
        self.ignore_first_move = true;
    }

    /// `releaseMouse`'s own part: let go, in the window's middle.
    pub fn release_mouse(&mut self, (width, height): (i32, i32)) {
        self.mouse_grabbed = false;
        self.xpos = f64::from(width) / 2.0;
        self.ypos = f64::from(height) / 2.0;
    }

    /// `resyncMousePosition`: where the mouse is stays where it was last put (Java asks SDL, which
    /// answers the same for a mouse that didn't move since).
    pub fn resync_mouse_position(&mut self) {}

    /// The movement since the last frame turns the player, while grabbed.
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
