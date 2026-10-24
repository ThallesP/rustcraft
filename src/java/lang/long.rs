//! java/lang/Long.java
//!
//! Only `parseLong(String)`, which Gson's numbers parse with (`LazilyParsedNumber.longValue`,
//! `JsonReader.nextLong`, `JsonPrimitive.getAsLong` of a string), and `parseLong(String, int)` and
//! `parseUnsignedLong(String, int)` in radixes 2, 10 and 16, which SNBT's numbers parse with
//! (`net.minecraft.nbt.TagParser`'s goldens check them). Like `Integer.parseInt`, they take any
//! Unicode digit `Character.digit` does.

use crate::java::lang::character::Character;
use crate::java::lang::number_format_exception::NumberFormatException;

pub struct Long;

impl Long {
    pub const MIN_VALUE: i64 = i64::MIN;
    pub const MAX_VALUE: i64 = i64::MAX;

    /// `parseLong(String)`: an optional sign, then decimal digits, accumulated negatively so
    /// that `MIN_VALUE` fits.
    pub fn parse_long(s: &str) -> Result<i64, NumberFormatException> {
        Long::parse_long_radix(s, 10)
    }

    /// `parseLong(String, int)`, in the radixes `Character::digit` takes.
    pub fn parse_long_radix(s: &str, radix: i32) -> Result<i64, NumberFormatException> {
        let chars: Vec<u16> = s.encode_utf16().collect();
        let len = chars.len();
        if len == 0 {
            return Err(NumberFormatException::for_input_string_radix("", radix));
        }
        let radix64 = i64::from(radix);
        let mut digit = !0xFF;
        let mut i = 0;
        let first_char = chars[i];
        i += 1;
        if first_char != u16::from(b'-') && first_char != u16::from(b'+') {
            digit = Character::digit(first_char, radix);
        }
        if digit >= 0 || (digit == !0xFF && len > 1) {
            let limit = if first_char != u16::from(b'-') {
                Long::MIN_VALUE + 1
            } else {
                Long::MIN_VALUE
            };
            let multmin = limit / radix64;
            let mut result = -i64::from(digit & 0xFF);
            let mut in_range = true;
            // Accumulating negatively avoids surprises near MAX_VALUE.
            while i < len {
                digit = Character::digit(chars[i], radix);
                i += 1;
                if digit < 0 {
                    break;
                }
                in_range = result > multmin
                    || (result == multmin && i64::from(digit) <= radix64 * multmin - limit);
                if !in_range {
                    break;
                }
                result = radix64 * result - i64::from(digit);
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

    /// `parseUnsignedLong(String, int)`: an optional `+`, then digits, accumulated as an
    /// unsigned long, which wraps to a negative one past `MAX_VALUE`. A `-` first, a character
    /// that isn't a digit and a value past 2^64 - 1 each have their own message.
    pub fn parse_unsigned_long_radix(s: &str, radix: i32) -> Result<i64, NumberFormatException> {
        let chars: Vec<u16> = s.encode_utf16().collect();
        let len = chars.len();
        if len == 0 {
            return Err(NumberFormatException::for_input_string_radix(s, radix));
        }
        let radix64 = i64::from(radix);
        let mut i = 0;
        let first_char = chars[i];
        i += 1;
        if first_char == u16::from(b'-') {
            return Err(NumberFormatException::new(format!(
                "Illegal leading minus sign on unsigned string {s}."
            )));
        }
        let mut digit = !0xFF;
        if first_char != u16::from(b'+') {
            digit = Character::digit(first_char, radix);
        }
        if digit >= 0 || (digit == !0xFF && len > 1) {
            // -1L, the largest unsigned long, divided by the radix.
            let multmax = (u64::MAX / radix as u64) as i64;
            let mut result = i64::from(digit & 0xFF);
            let mut in_range = true;
            while i < len {
                digit = Character::digit(chars[i], radix);
                i += 1;
                if digit < 0 {
                    break;
                }
                in_range = (result as u64) < (multmax as u64)
                    || (result == multmax
                        && digit < (radix64.wrapping_neg().wrapping_mul(multmax)) as i32);
                if !in_range {
                    break;
                }
                result = radix64.wrapping_mul(result).wrapping_add(i64::from(digit));
            }
            if in_range && i == len && digit >= 0 {
                return Ok(result);
            }
        }
        if digit < 0 {
            return Err(NumberFormatException::for_input_string_radix(s, radix));
        }
        Err(NumberFormatException::new(format!(
            "String value {s} exceeds range of unsigned long."
        )))
    }
}
