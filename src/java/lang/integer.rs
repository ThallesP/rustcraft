//! java/lang/Integer.java
//!
//! Only `parseInt(String)`, which `IntegerProperty.getValue` parses a value's name with, and
//! `parseInt(String, int)` in radix 16, which `TextColor.parseColor` reads a hex color with. They
//! take any digit `Character.digit` does, not only ASCII ones: `"٣"` is 3.

use crate::java::lang::character::Character;

pub struct Integer;

/// Java's `NumberFormatException`.
#[derive(Debug, PartialEq, Eq)]
pub struct NumberFormatException;

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
        let Some(&first) = chars.first() else {
            return Err(NumberFormatException);
        };
        let mut negative = false;
        let mut limit = -Integer::MAX_VALUE;
        let mut i = 0;
        if first < u16::from(b'0') {
            if first == u16::from(b'-') {
                negative = true;
                limit = Integer::MIN_VALUE;
            } else if first != u16::from(b'+') {
                return Err(NumberFormatException);
            }
            if chars.len() == 1 {
                return Err(NumberFormatException);
            }
            i += 1;
        }

        let multmin = limit / radix;
        let mut result = 0i32;
        while i < chars.len() {
            let digit = Character::digit(chars[i], radix);
            i += 1;
            if digit < 0 || result < multmin {
                return Err(NumberFormatException);
            }
            result *= radix;
            if result < limit + digit {
                return Err(NumberFormatException);
            }
            result -= digit;
        }
        Ok(if negative { result } else { -result })
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
            Integer::parse_int(&s).unwrap_or_else(|_| panic!("NumberFormatException: {s:?}"))
        });
        golden.finish();
    }
}
