//! net/minecraft/client/input/MouseButtonInfo.java

use crate::net::minecraft::client::input::input_with_modifiers::InputWithModifiers;

/// SDL's button (1 the left one) and the modifiers held.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MouseButtonInfo {
    pub button: i32,
    pub modifiers: i32,
}

impl MouseButtonInfo {
    pub fn new(button: i32, modifiers: i32) -> MouseButtonInfo {
        MouseButtonInfo { button, modifiers }
    }
}

impl InputWithModifiers for MouseButtonInfo {
    fn input(&self) -> i32 {
        self.button
    }

    fn modifiers(&self) -> i32 {
        self.modifiers
    }
}
