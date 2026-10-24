//! net/minecraft/util/StringUtil.java
//!
//! Only `isAllowedChatCharacter`, which `ExtraCodecs.CHAT_STRING` checks a command's characters
//! with, and `isValidPlayerName`, which `ExtraCodecs.PLAYER_NAME` checks a name with. The rest comes
//! with chat.

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
}
