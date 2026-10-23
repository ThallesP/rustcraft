//! java/lang/Character.java
//!
//! Only `digit(char, int)` in radixes 10 and 16, which `Integer.parseInt` reads digits with (a
//! text color's hex). The golden suite keeps to those two (`where()`).

pub struct Character;

// The zero of each run of ten decimal digits in the Basic Multilingual Plane, which
// `Character.digit(char, 10)` reads as 0 to 9: Unicode's Nd characters as the JDK 25 knows them,
// listed by running it over every char. A supplementary digit is two surrogates, neither a digit.
const DIGIT_ZEROS: [u16; 37] = [
    0x0030, 0x0660, 0x06F0, 0x07C0, 0x0966, 0x09E6, 0x0A66, 0x0AE6, 0x0B66, 0x0BE6, 0x0C66, 0x0CE6,
    0x0D66, 0x0DE6, 0x0E50, 0x0ED0, 0x0F20, 0x1040, 0x1090, 0x17E0, 0x1810, 0x1946, 0x19D0, 0x1A80,
    0x1A90, 0x1B50, 0x1BB0, 0x1C40, 0x1C50, 0xA620, 0xA8D0, 0xA900, 0xA9D0, 0xA9F0, 0xAA50, 0xABF0,
    0xFF10,
];

impl Character {
    /// `digit(char, radix)` for radix 10 or 16: a decimal digit's value, or a Latin letter's
    /// (`A` to `Z`, either case, ASCII or fullwidth) from 10 up, if it's less than the radix; else
    /// -1. Panics on another radix, which isn't ported.
    pub fn digit(ch: u16, radix: i32) -> i32 {
        assert!(
            radix == 10 || radix == 16,
            "Character.digit is only ported for radixes 10 and 16"
        );
        let value = DIGIT_ZEROS
            .iter()
            .find(|&&zero| ch >= zero && ch - zero < 10)
            .map(|zero| (ch - zero) as i32)
            .or_else(|| {
                [0x41u16, 0x61, 0xFF21, 0xFF41]
                    .iter()
                    .find(|&&a| ch >= a && ch - a < 26)
                    .map(|a| (ch - a) as i32 + 10)
            });
        match value {
            Some(value) if value < radix => value,
            _ => -1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Character;
    use crate::golden::Golden;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("java.lang.Character");
        golden.check("digit(CI)I", Character::digit);
        golden.finish();
    }
}
