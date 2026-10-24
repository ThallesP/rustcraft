//! net/minecraft/client/input/CharacterEvent.java

use crate::net::minecraft::util::string_util::StringUtil;

/// A character typed, as its codepoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CharacterEvent {
    pub codepoint: i32,
}

impl CharacterEvent {
    pub fn new(codepoint: i32) -> CharacterEvent {
        CharacterEvent { codepoint }
    }

    /// `Character.toString(codepoint)`: U+FFFD for a lone surrogate, which a Rust string can't
    /// hold.
    pub fn codepoint_as_string(&self) -> String {
        char::from_u32(self.codepoint as u32)
            .unwrap_or('\u{FFFD}')
            .to_string()
    }

    pub fn is_allowed_chat_character(&self) -> bool {
        StringUtil::is_allowed_chat_character(self.codepoint)
    }
}
