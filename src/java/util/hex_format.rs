//! java/util/HexFormat.java
//!
//! Only `fromHexDigits(CharSequence)`, which SNBT's `\x`, `\u` and `\U` escapes read their digits
//! with. Checked through `net.minecraft.nbt.TagParser`'s goldens.

use crate::java::lang::number_format_exception::NumberFormatException;

pub struct HexFormat;

impl HexFormat {
    /// `fromHexDigits(CharSequence)`: up to eight ASCII hex digits, either case, as the int with
    /// those bits (eight of them can make a negative one). Panics where Java throws: more than
    /// eight digits (`IllegalArgumentException`), or a character that isn't a hex digit
    /// (`NumberFormatException`).
    pub fn from_hex_digits(string: &[u16]) -> i32 {
        assert!(
            string.len() <= 8,
            "java.lang.IllegalArgumentException: string length greater than 8: {}",
            string.len()
        );
        let mut value: i32 = 0;
        for &ch in string {
            value = (value << 4)
                + HexFormat::from_hex_digit(ch).unwrap_or_else(|error| panic!("{error}"));
        }
        value
    }

    /// `fromHexDigit(int)`: an ASCII hex digit's value.
    pub fn from_hex_digit(ch: u16) -> Result<i32, NumberFormatException> {
        match char::from_u32(u32::from(ch))
            .filter(char::is_ascii)
            .and_then(|c| c.to_digit(16))
        {
            Some(value) => Ok(value as i32),
            None => Err(NumberFormatException::new(format!(
                "not a hexadecimal digit: \"{}\" = {ch}",
                String::from_utf16_lossy(&[ch])
            ))),
        }
    }
}
