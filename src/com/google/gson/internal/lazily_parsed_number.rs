//! com/google/gson/internal/LazilyParsedNumber.java (Gson 2.14.0)
//!
//! A JSON number as its text, which Gson parses into each type when it's asked for one. Written
//! from Gson's behaviour: its sources aren't among the game's.

use std::fmt;

/// The text of a JSON number, as `JsonReader` read it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LazilyParsedNumber {
    value: String,
}

impl LazilyParsedNumber {
    pub fn new(value: String) -> LazilyParsedNumber {
        LazilyParsedNumber { value }
    }

    /// `Float.parseFloat` of the text, correctly rounded straight to a float like Java's (not
    /// through a double). A JSON number's text is all decimal, which Rust's parser reads the same;
    /// Java's grammar adds a type suffix, hex and surrounding whitespace, which no JSON number has.
    /// Java throws `NumberFormatException` where this panics.
    pub fn float_value(&self) -> f32 {
        self.value.parse().unwrap_or_else(|_| {
            panic!(
                "NumberFormatException: For input string: \"{}\"",
                self.value
            )
        })
    }

    pub fn double_value(&self) -> f64 {
        self.value.parse().unwrap_or_else(|_| {
            panic!(
                "NumberFormatException: For input string: \"{}\"",
                self.value
            )
        })
    }

    /// `Integer.parseInt`, else `(int) Long.parseLong`, else the decimal's `intValue()`: its
    /// integer part, wrapped to an int.
    pub fn int_value(&self) -> i32 {
        if let Ok(value) = self.value.parse::<i32>() {
            return value;
        }
        if let Ok(value) = self.value.parse::<i64>() {
            return value as i32;
        }
        self.big_decimal_int_value()
    }

    // `new BigDecimal(value).intValue()`: the integer part, its low 32 bits. Through a double,
    // which only differs past 2^53, where no model's number is.
    fn big_decimal_int_value(&self) -> i32 {
        let value = self.double_value();
        if value.abs() < 9.007_199_254_740_992e15 {
            (value.trunc() as i64) as i32
        } else {
            panic!(
                "LazilyParsedNumber: {} is past what the port reads",
                self.value
            )
        }
    }
}

/// `toString()`: the text.
impl fmt::Display for LazilyParsedNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::LazilyParsedNumber;
    use crate::golden::Golden;

    /// Every number in the vanilla pack's models and block states, read the way Gson reads it.
    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("com.google.gson.internal.LazilyParsedNumber");
        golden.check("floatValue()F", |text: String| {
            LazilyParsedNumber::new(text).float_value()
        });
        golden.check("intValue()I", |text: String| {
            LazilyParsedNumber::new(text).int_value()
        });
        golden.finish();
    }
}
