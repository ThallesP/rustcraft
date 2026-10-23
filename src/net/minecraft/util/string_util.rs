//! net/minecraft/util/StringUtil.java
//!
//! Only `isAllowedChatCharacter`, which `ExtraCodecs.CHAT_STRING` checks a command's characters
//! with. The rest comes with chat.

pub struct StringUtil;

impl StringUtil {
    /// Not `§`, a control character or delete.
    pub fn is_allowed_chat_character(ch: i32) -> bool {
        ch != 167 && ch >= 32 && ch != 127
    }
}
