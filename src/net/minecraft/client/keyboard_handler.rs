//! net/minecraft/client/KeyboardHandler.java
//!
//! Only the keys a person walks around with, by their default bindings: W, A, S and D, space,
//! left shift (sneak) and left control (sprint) hold their `KeyMapping`s down while pressed, F1
//! hides the HUD (`keyToggleGui`), and Escape lets go of the mouse, where vanilla opens the pause
//! screen. Not ported yet: the rest of the bindings and their options, the debug keys (F3 and its
//! combinations), screens' key handling, the narrator, and character input.

use crate::net::minecraft::client::minecraft::Minecraft;

pub struct KeyboardHandler;

impl KeyboardHandler {
    /// SDL's scancodes for the default bindings (`InputConstants`' keys).
    const KEY_A: i32 = 4;
    const KEY_D: i32 = 7;
    const KEY_S: i32 = 22;
    const KEY_W: i32 = 26;
    const KEY_ESCAPE: i32 = 41;
    const KEY_SPACE: i32 = 44;
    const KEY_F1: i32 = 58;
    const KEY_LCONTROL: i32 = 224;
    const KEY_LSHIFT: i32 = 225;

    /// `keyPress`: a key going down (`down`) or up, or repeating while held.
    pub fn key_press(minecraft: &Minecraft, scancode: i32, down: bool, repeat: bool) -> KeyAction {
        let mut options = minecraft.options.borrow_mut();
        let mapping = match scancode {
            KeyboardHandler::KEY_W => Some(&mut options.key_up),
            KeyboardHandler::KEY_A => Some(&mut options.key_left),
            KeyboardHandler::KEY_S => Some(&mut options.key_down),
            KeyboardHandler::KEY_D => Some(&mut options.key_right),
            KeyboardHandler::KEY_SPACE => Some(&mut options.key_jump),
            KeyboardHandler::KEY_LSHIFT => Some(&mut options.key_shift),
            KeyboardHandler::KEY_LCONTROL => Some(&mut options.key_sprint),
            _ => None,
        };
        if let Some(mapping) = mapping {
            mapping.set_down(down);
            return KeyAction::None;
        }

        match scancode {
            KeyboardHandler::KEY_F1 if down && !repeat => {
                minecraft.hud_hidden.set(!minecraft.hud_hidden.get());
                KeyAction::None
            }
            KeyboardHandler::KEY_ESCAPE if down && !repeat => KeyAction::ReleaseMouse,
            _ => KeyAction::None,
        }
    }
}

/// What a key asks of the window, which the handler doesn't hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyAction {
    None,
    ReleaseMouse,
}
