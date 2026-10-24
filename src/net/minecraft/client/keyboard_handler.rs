//! net/minecraft/client/KeyboardHandler.java
//!
//! Keys go to the screen first (`keyPressed`, `keyReleased`), and to the game's key mappings when
//! there's no screen: W, A, S and D, space, left shift (sneak) and left control (sprint) hold their
//! mappings down while pressed, T and / are clicked to open the chat, F1 hides the HUD, and Escape
//! opens the pause menu. Not ported yet: the rest of the bindings and their options, the global
//! keys (fullscreen, screenshots), the debug keys (F3 and its combinations), the narrator's hotkey,
//! and the clipboard.

use crate::net::minecraft::client::gui::screens::pause_screen::PauseScreen;
use crate::net::minecraft::client::input::input_with_modifiers::InputWithModifiers;
use crate::net::minecraft::client::input::key_event::KeyEvent;
use crate::net::minecraft::client::input_type::{self, InputType};
use crate::net::minecraft::client::minecraft::Minecraft;

pub struct KeyboardHandler;

/// `KeyEvent.Action`: GLFW's and SDL's release, press and repeat.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyAction {
    RELEASE,
    PRESS,
    REPEAT,
}

impl KeyboardHandler {
    /// SDL's scancodes for the default bindings (`InputConstants`' keys).
    const KEY_A: i32 = 4;
    const KEY_D: i32 = 7;
    const KEY_S: i32 = 22;
    const KEY_W: i32 = 26;
    const KEY_SPACE: i32 = 44;
    const KEY_F1: i32 = 58;
    const KEY_LCONTROL: i32 = 224;
    const KEY_LSHIFT: i32 = 225;

    /// `KeyMapping.set(key, down)`, and a click when it goes down (`KeyMapping.click`).
    fn set_mappings(minecraft: &Minecraft, key: i32, down: bool, click: bool) {
        let mut options = minecraft.options.borrow_mut();
        let mapping = match key {
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
            if click {
                mapping.click();
            }
        }
        for mapping in options.key_mappings_mut() {
            if mapping.matches(key) {
                mapping.set_down(down);
                if click {
                    mapping.click();
                }
            }
        }
    }

    /// `keyPress`: a key going down, repeating while held, or up.
    pub fn key_press(minecraft: &Minecraft, event: &KeyEvent, action: KeyAction) {
        let screen = minecraft.screen();
        if screen.is_some() {
            match event.shortcut_key() {
                9 => input_type::set_last_input_type(InputType::KEYBOARD_TAB),
                1073741903..=1073741906 => {
                    input_type::set_last_input_type(InputType::KEYBOARD_ARROW)
                }
                _ => {}
            }
            let handled = match action {
                KeyAction::PRESS | KeyAction::REPEAT => {
                    minecraft.with_screen(|screen| screen.key_pressed(event)) == Some(true)
                }
                KeyAction::RELEASE => {
                    minecraft.with_screen(|screen| screen.key_released(event)) == Some(true)
                }
            };
            if handled {
                if action != KeyAction::RELEASE && minecraft.screen().is_none() {
                    KeyboardHandler::set_mappings(minecraft, event.key, false, false);
                }
                return;
            }
        }

        let handles_game_input = minecraft.screen().is_none();
        let handles_global_input = handles_game_input
            || minecraft.screen().is_some_and(|screen| {
                screen.try_borrow().is_ok_and(|screen| {
                    screen
                        .as_any()
                        .downcast_ref::<PauseScreen>()
                        .is_some_and(|pause| !pause.shows_pause_menu())
                })
            });
        if action == KeyAction::RELEASE {
            KeyboardHandler::set_mappings(minecraft, event.key, false, false);
        } else {
            if handles_global_input && event.is_escape() {
                minecraft.pause_game(false);
            }
            if handles_game_input {
                KeyboardHandler::set_mappings(minecraft, event.key, true, true);
                // F1 is `keyToggleGui`, which `Gui.handleKeybinds` toggles the HUD with.
                if event.key == KeyboardHandler::KEY_F1 && action == KeyAction::PRESS {
                    minecraft.hud_hidden.set(!minecraft.hud_hidden.get());
                }
            }
        }
    }
}
