//! net/minecraft/client/input/MouseButtonEvent.java

use crate::net::minecraft::client::input::input_with_modifiers::InputWithModifiers;
use crate::net::minecraft::client::input::mouse_button_info::MouseButtonInfo;

/// A button at a place in the GUI's scaled coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MouseButtonEvent {
    pub x: f64,
    pub y: f64,
    pub button_info: MouseButtonInfo,
}

impl MouseButtonEvent {
    pub fn new(x: f64, y: f64, button_info: MouseButtonInfo) -> MouseButtonEvent {
        MouseButtonEvent { x, y, button_info }
    }

    pub fn button(&self) -> i32 {
        self.button_info.button
    }
}

impl InputWithModifiers for MouseButtonEvent {
    fn input(&self) -> i32 {
        self.button()
    }

    fn modifiers(&self) -> i32 {
        self.button_info.modifiers
    }
}
