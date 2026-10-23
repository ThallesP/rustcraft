//! java/lang/Integer.java
//!
//! Only `parseInt(String)`, which `IntegerProperty.getValue` parses a value's name with, and Gson's
//! numbers (`LazilyParsedNumber.intValue`, `JsonPrimitive.getAsInt` of a string), and
//! `parseInt(String, int)` in radix 16, which `TextColor.parseColor` reads a hex color with. They
//! take any digit `Character.digit` does, not only ASCII ones: `"٣"` is 3.

use crate::java::lang::character::Character;
use crate::java::lang::number_format_exception::NumberFormatException;

pub struct Integer;

impl Integer {
    pub const MIN_VALUE: i32 = i32::MIN;
    pub const MAX_VALUE: i32 = i32::MAX;

    /// `parseInt(String)`: an optional sign, then decimal digits, accumulated negatively so
    /// that `MIN_VALUE` fits, like Java's.
    pub fn parse_int(s: &str) -> Result<i32, NumberFormatException> {
        Integer::parse_int_radix(s, 10)
    }

    /// `parseInt(String, int)`, for the radixes `Character.digit` is ported for.
    pub fn parse_int_radix(s: &str, radix: i32) -> Result<i32, NumberFormatException> {
        let chars: Vec<u16> = s.encode_utf16().collect();
        let len = chars.len();
        if len == 0 {
            return Err(NumberFormatException::for_input_string_radix("", radix));
        }
        let mut digit = !0xFF;
        let mut i = 0;
        let first_char = chars[i];
        i += 1;
        if first_char != u16::from(b'-') && first_char != u16::from(b'+') {
            digit = Character::digit(first_char, radix);
        }
        if digit >= 0 || (digit == !0xFF && len > 1) {
            let limit = if first_char != u16::from(b'-') {
                Integer::MIN_VALUE + 1
            } else {
                Integer::MIN_VALUE
            };
            let multmin = limit / radix;
            let mut result = -(digit & 0xFF);
            let mut in_range = true;
            // Accumulating negatively avoids surprises near MAX_VALUE.
            while i < len {
                digit = Character::digit(chars[i], radix);
                i += 1;
                if digit < 0 {
                    break;
                }
                in_range =
                    result > multmin || (result == multmin && digit <= radix * multmin - limit);
                if !in_range {
                    break;
                }
                result = radix * result - digit;
            }
            if in_range && i == len && digit >= 0 {
                return Ok(if first_char != u16::from(b'-') {
                    -result
                } else {
                    result
                });
            }
        }
        Err(NumberFormatException::for_input_string_radix(s, radix))
    }
}

#[cfg(test)]
mod tests {
    use super::Integer;
    use crate::golden::Golden;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("java.lang.Integer");
        golden.check("parseInt(Ljava/lang/String;)I", |s: String| {
            Integer::parse_int(&s).unwrap_or_else(|error| panic!("{error}"))
        });
        golden.finish();
    }
}
