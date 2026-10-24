//! net/minecraft/client/input/KeyEvent.java
//!
//! A key's SDL scancode (`key`, what bindings and `InputWithModifiers` test), its keycode in the
//! keyboard's layout (`keycode`, what shortcuts test) and the modifiers. `keycode_of` is SDL's
//! default US layout (`SDL_GetKeyFromScancode` without a keyboard's own layout, which is what SDL
//! answers before its video is up), for the keys a scenario presses and a headless client has no
//! window to ask.

use crate::net::minecraft::client::input::input_with_modifiers::InputWithModifiers;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    pub key: i32,
    pub keycode: i32,
    pub modifiers: i32,
}

impl KeyEvent {
    pub fn new(key: i32, keycode: i32, modifiers: i32) -> KeyEvent {
        KeyEvent {
            key,
            keycode,
            modifiers,
        }
    }

    /// SDL's default keymap (`SDL_default_keymap`) for the scancodes of letters, digits, the
    /// controls and the arrows: a letter's lower case, a digit's, ASCII's controls, and
    /// `SDLK_SCANCODE_MASK` with the scancode for the rest.
    pub fn keycode_of(scancode: i32) -> i32 {
        const SCANCODE_MASK: i32 = 1 << 30;
        match scancode {
            4..=29 => i32::from(b'a') + scancode - 4,
            30..=38 => i32::from(b'1') + scancode - 30,
            39 => i32::from(b'0'),
            40 => 13,
            41 => 27,
            42 => 8,
            43 => 9,
            44 => 32,
            45 => i32::from(b'-'),
            46 => i32::from(b'='),
            47 => i32::from(b'['),
            48 => i32::from(b']'),
            49 => i32::from(b'\\'),
            51 => i32::from(b';'),
            52 => i32::from(b'\''),
            53 => i32::from(b'`'),
            54 => i32::from(b','),
            55 => i32::from(b'.'),
            56 => i32::from(b'/'),
            76 => 127,
            _ => SCANCODE_MASK | scancode,
        }
    }

    /// A key pressed with no modifiers, its keycode SDL's default.
    pub fn of_scancode(scancode: i32) -> KeyEvent {
        KeyEvent::new(scancode, KeyEvent::keycode_of(scancode), 0)
    }
}

impl InputWithModifiers for KeyEvent {
    fn input(&self) -> i32 {
        self.key
    }

    fn shortcut_key(&self) -> i32 {
        self.keycode
    }

    fn modifiers(&self) -> i32 {
        self.modifiers
    }
}
