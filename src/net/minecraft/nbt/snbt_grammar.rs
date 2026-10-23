//! net/minecraft/nbt/SnbtGrammar.java
//!
//! Not ported yet: everything but `escapeControlCharacters`, which `StringTag`'s quoting writes
//! SNBT with: the grammar that parses SNBT (`TagParser`), which comes with commands.

/// The escape after a backslash for a control character: `b`, `t`, `n`, `f`, `r`, or `x` and its
/// two hex digits in upper case, or none for any other character.
pub fn escape_control_characters(c: char) -> Option<String> {
    match c {
        '\u{8}' => Some("b".to_owned()),
        '\t' => Some("t".to_owned()),
        '\n' => Some("n".to_owned()),
        '\u{c}' => Some("f".to_owned()),
        '\r' => Some("r".to_owned()),
        c if c < ' ' => Some(format!("x{:02X}", c as u32)),
        _ => None,
    }
}
