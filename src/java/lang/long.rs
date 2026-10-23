//! java/lang/Long.java
//!
//! Only `parseLong(String)`, which Gson's numbers parse with (`LazilyParsedNumber.longValue`,
//! `JsonReader.nextLong`, `JsonPrimitive.getAsLong` of a string). Like `Integer.parseInt`, it takes
//! any Unicode decimal digit.

use crate::java::lang::character::Character;
use crate::java::lang::number_format_exception::NumberFormatException;

pub struct Long;

impl Long {
    pub const MIN_VALUE: i64 = i64::MIN;
    pub const MAX_VALUE: i64 = i64::MAX;

    /// `parseLong(String)`: an optional sign, then decimal digits, accumulated negatively so
    /// that `MIN_VALUE` fits.
    pub fn parse_long(s: &str) -> Result<i64, NumberFormatException> {
        let chars: Vec<u16> = s.encode_utf16().collect();
        let len = chars.len();
        if len == 0 {
            return Err(NumberFormatException::for_input_string(""));
        }
        let mut digit = !0xFF;
        let mut i = 0;
        let first_char = chars[i];
        i += 1;
        if first_char != u16::from(b'-') && first_char != u16::from(b'+') {
            digit = Character::digit(first_char, 10);
        }
        if digit >= 0 || (digit == !0xFF && len > 1) {
            let limit = if first_char != u16::from(b'-') {
                Long::MIN_VALUE + 1
            } else {
                Long::MIN_VALUE
            };
            let multmin = limit / 10;
            let mut result = -i64::from(digit & 0xFF);
            let mut in_range = true;
            // Accumulating negatively avoids surprises near MAX_VALUE.
            while i < len {
                digit = Character::digit(chars[i], 10);
                i += 1;
                if digit < 0 {
                    break;
                }
                in_range = result > multmin
                    || (result == multmin && i64::from(digit) <= 10 * multmin - limit);
                if !in_range {
                    break;
                }
                result = 10 * result - i64::from(digit);
            }
            if in_range && i == len && digit >= 0 {
                return Ok(if first_char != u16::from(b'-') {
                    -result
                } else {
                    result
                });
            }
        }
        Err(NumberFormatException::for_input_string(s))
    }
}
