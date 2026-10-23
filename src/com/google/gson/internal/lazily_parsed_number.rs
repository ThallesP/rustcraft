//! com/google/gson/internal/LazilyParsedNumber.java (Gson 2.14.0)
//!
//! A JSON number as its text, which Gson parses into each type when it's asked for one: what
//! `JsonReader` reads a number as, held in a `JsonPrimitive` as a `java::lang::number::Number`.
//! A text that doesn't parse throws `NumberFormatException` in Java, which panics here, with
//! Java's `toString()` of the exception. `writeReplace` and `readObject` (Java serialization)
//! aren't ported.

/*
 * Copyright (C) 2011 Google Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * Translated to Rust from Gson 2.14.0, and changed: see above.
 */

use std::fmt;

use crate::com::google::gson::internal::number_limits::NumberLimits;
use crate::java::lang::double::Double;
use crate::java::lang::float::Float;
use crate::java::lang::integer::Integer;
use crate::java::lang::long::Long;
use crate::java::lang::string::JavaString;
use crate::java::math::big_decimal::BigDecimal;

/// The text of a JSON number. `equals` and `hashCode` are the text's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LazilyParsedNumber {
    value: String,
}

impl LazilyParsedNumber {
    pub fn new(value: String) -> LazilyParsedNumber {
        LazilyParsedNumber { value }
    }

    fn as_big_decimal(&self) -> BigDecimal {
        NumberLimits::parse_big_decimal(&self.value).unwrap_or_else(|error| panic!("{error}"))
    }

    /// `Integer.parseInt`, else `(int) Long.parseLong`, else the decimal's `intValue()`: its
    /// integer part's low 32 bits.
    pub fn int_value(&self) -> i32 {
        Integer::parse_int(&self.value)
            .or_else(|_| Long::parse_long(&self.value).map(|value| value as i32))
            .unwrap_or_else(|_| self.as_big_decimal().int_value())
    }

    /// `Long.parseLong`, else the decimal's `longValue()`: its integer part's low 64 bits.
    pub fn long_value(&self) -> i64 {
        Long::parse_long(&self.value).unwrap_or_else(|_| self.as_big_decimal().long_value())
    }

    /// `Float.parseFloat`, rounded straight to a float, not through a double.
    pub fn float_value(&self) -> f32 {
        Float::parse_float(&self.value).unwrap_or_else(|error| panic!("{error}"))
    }

    /// `Double.parseDouble`.
    pub fn double_value(&self) -> f64 {
        Double::parse_double(&self.value).unwrap_or_else(|error| panic!("{error}"))
    }

    /// `hashCode()`: the text's.
    pub fn hash_code(&self) -> i32 {
        self.value.hash_code()
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
