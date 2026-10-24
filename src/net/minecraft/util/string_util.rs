//! net/minecraft/util/StringUtil.java
//!
//! `isAllowedChatCharacter`, which `ExtraCodecs.CHAT_STRING` checks a command's characters with,
//! `isValidPlayerName`, which `ExtraCodecs.PLAYER_NAME` checks a name with, and what the chat and
//! its edit box run text through. Strings are cut at UTF-16 units, like Java's.

pub struct StringUtil;

impl StringUtil {
    /// Not `§`, a control character or delete.
    pub fn is_allowed_chat_character(ch: i32) -> bool {
        ch != 167 && ch >= 32 && ch != 127
    }

    /// At most 16 UTF-16 units, none a space, a control character or past `~`.
    pub fn is_valid_player_name(name: &str) -> bool {
        let units: Vec<u16> = name.encode_utf16().collect();
        units.len() <= 16 && units.iter().all(|&c| c > 32 && c < 127)
    }

    /// `truncateStringIfNecessary`.
    pub fn truncate_string_if_necessary(
        s: &str,
        max_length: usize,
        add_dot_dot_dot_if_truncated: bool,
    ) -> String {
        let units: Vec<u16> = s.encode_utf16().collect();
        if units.len() <= max_length {
            s.to_string()
        } else if add_dot_dot_dot_if_truncated && max_length > 3 {
            String::from_utf16_lossy(&units[..max_length - 3]) + "..."
        } else {
            String::from_utf16_lossy(&units[..max_length])
        }
    }

    /// `trimChatMessage`: at most 256 units.
    pub fn trim_chat_message(message: &str) -> String {
        StringUtil::truncate_string_if_necessary(message, 256, false)
    }

    /// `filterText(String)`: the allowed characters.
    pub fn filter_text(input: &str) -> String {
        let units: Vec<u16> = input
            .encode_utf16()
            .filter(|&c| StringUtil::is_allowed_chat_character(i32::from(c)))
            .collect();
        String::from_utf16_lossy(&units)
    }
}
