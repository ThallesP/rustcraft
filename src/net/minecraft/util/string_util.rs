//! net/minecraft/util/StringUtil.java
//!
//! `isAllowedChatCharacter`, which `ExtraCodecs.CHAT_STRING` checks a command's characters with,
//! `isValidPlayerName`, which `ExtraCodecs.PLAYER_NAME` checks a name with, and `filterText`, which
//! an edit box runs typed text through. The rest comes with chat.

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

    /// `filterText(String)`: the allowed characters.
    pub fn filter_text(input: &str) -> String {
        let units: Vec<u16> = input
            .encode_utf16()
            .filter(|&c| StringUtil::is_allowed_chat_character(i32::from(c)))
            .collect();
        String::from_utf16_lossy(&units)
    }
}
