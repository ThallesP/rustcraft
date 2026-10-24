//! net/minecraft/client/InputType.java, with `Minecraft`'s `lastInputType`
//!
//! Which input the player used last, which decides whether a focused widget shows its tooltip and
//! where a screen puts its first focus. Java keeps it on the game, which widgets reach through
//! `Minecraft.getInstance()`; here it's the game thread's (`last_input_type`), the only thread
//! that reads or sets it.

use std::cell::Cell;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputType {
    NONE,
    MOUSE,
    KEYBOARD_ARROW,
    KEYBOARD_TAB,
}

impl InputType {
    pub fn is_mouse(self) -> bool {
        self == InputType::MOUSE
    }

    pub fn is_keyboard(self) -> bool {
        self == InputType::KEYBOARD_ARROW || self == InputType::KEYBOARD_TAB
    }
}

thread_local! {
    static LAST_INPUT_TYPE: Cell<InputType> = const { Cell::new(InputType::NONE) };
}

/// `Minecraft.getLastInputType()`.
pub fn last_input_type() -> InputType {
    LAST_INPUT_TYPE.with(Cell::get)
}

/// `Minecraft.setLastInputType`.
pub fn set_last_input_type(input_type: InputType) {
    LAST_INPUT_TYPE.with(|last| last.set(input_type));
}
