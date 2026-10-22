//! java/lang/Integer.java
//!
//! Only `parseInt(String)`, which `IntegerProperty.getValue` parses a value's name with. It takes
//! any Unicode decimal digit, as `Character.digit` does, not only ASCII ones: `"٣"` is 3.

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

        let multmin = limit / 10;
        let mut result = 0i32;
        while i < chars.len() {
            let digit = Character::digit(chars[i], 10);
            i += 1;
            if digit < 0 || result < multmin {
                return Err(NumberFormatException);
            }
            result *= 10;
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
