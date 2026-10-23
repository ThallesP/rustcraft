//! net/minecraft/ChatFormatting.java
//!
//! The legacy `§` formatting codes: the sixteen colors, the five flags, and reset. A code is read
//! case-insensitively (`getByCode`). `stripFormatting` comes with its first caller.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatFormatting {
    BLACK,
    DARK_BLUE,
    DARK_GREEN,
    DARK_AQUA,
    DARK_RED,
    DARK_PURPLE,
    GOLD,
    GRAY,
    DARK_GRAY,
    BLUE,
    GREEN,
    AQUA,
    RED,
    LIGHT_PURPLE,
    YELLOW,
    WHITE,
    OBFUSCATED,
    BOLD,
    STRIKETHROUGH,
    UNDERLINE,
    ITALIC,
    RESET,
}

impl ChatFormatting {
    pub const PREFIX_CODE: char = '§';
    pub const VALUES: [ChatFormatting; 22] = [
        ChatFormatting::BLACK,
        ChatFormatting::DARK_BLUE,
        ChatFormatting::DARK_GREEN,
        ChatFormatting::DARK_AQUA,
        ChatFormatting::DARK_RED,
        ChatFormatting::DARK_PURPLE,
        ChatFormatting::GOLD,
        ChatFormatting::GRAY,
        ChatFormatting::DARK_GRAY,
        ChatFormatting::BLUE,
        ChatFormatting::GREEN,
        ChatFormatting::AQUA,
        ChatFormatting::RED,
        ChatFormatting::LIGHT_PURPLE,
        ChatFormatting::YELLOW,
        ChatFormatting::WHITE,
        ChatFormatting::OBFUSCATED,
        ChatFormatting::BOLD,
        ChatFormatting::STRIKETHROUGH,
        ChatFormatting::UNDERLINE,
        ChatFormatting::ITALIC,
        ChatFormatting::RESET,
    ];

    pub fn code(self) -> char {
        b"0123456789abcdefklmnor"[self as usize] as char
    }

    /// `getByCode`: the formatting of a code in either case (`Character.toLowerCase`), or none.
    pub fn get_by_code(code: u16) -> Option<ChatFormatting> {
        let code = char::from_u32(u32::from(code))?;
        let sanitized = code.to_lowercase().next().unwrap_or(code);
        ChatFormatting::VALUES
            .into_iter()
            .find(|format| format.code() == sanitized)
    }
}

impl std::fmt::Display for ChatFormatting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "§{}", self.code())
    }
}
