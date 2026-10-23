//! com/google/gson/JsonPrimitive.java (Gson 2.14.0)
//!
//! A JSON primitive: Java's `Object value`, a `Boolean`, a `Number` of any class (a parsed number
//! is a `LazilyParsedNumber`, its text) or a `String`, as an enum. `JsonPrimitive(Character)`, a
//! one-character string, isn't ported, and neither are `getAsBigInteger` and the deprecated
//! `getAsCharacter`. A getter that throws in Java (`NumberFormatException` for a string that
//! doesn't parse, `UnsupportedOperationException` for a boolean's number) panics, with Java's
//! `toString()` of the exception.
//!
//! `equals` is `PartialEq`, and can panic too: two numbers that aren't both integral compare as
//! doubles, and a `LazilyParsedNumber` whose text isn't a number throws there. Java returns true
//! first when both are the same object, which a clone stands for here (Java's shared reference),
//! so two `LazilyParsedNumber`s of the same text are equal without parsing it.

/*
 * Copyright (C) 2008 Google Inc.
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

use std::cmp::Ordering;

use crate::com::google::gson::internal::lazily_parsed_number::LazilyParsedNumber;
use crate::com::google::gson::internal::number_limits::NumberLimits;
use crate::java::lang::byte::Byte;
use crate::java::lang::double::Double;
use crate::java::lang::float::Float;
use crate::java::lang::integer::Integer;
use crate::java::lang::long::Long;
use crate::java::lang::number::Number;
use crate::java::lang::short::Short;
use crate::java::lang::string::JavaString;
use crate::java::math::big_decimal::BigDecimal;

#[derive(Clone, Debug)]
pub enum JsonPrimitive {
    Boolean(bool),
    Number(Number),
    String(String),
}

impl JsonPrimitive {
    pub fn is_boolean(&self) -> bool {
        matches!(self, JsonPrimitive::Boolean(_))
    }

    /// A boolean, or `Boolean.parseBoolean` of the text: whether it's "true" in any case.
    pub fn get_as_boolean(&self) -> bool {
        match self {
            JsonPrimitive::Boolean(value) => *value,
            _ => self.get_as_string().eq_ignore_ascii_case("true"),
        }
    }

    pub fn is_number(&self) -> bool {
        matches!(self, JsonPrimitive::Number(_))
    }

    /// The number, or a string's text as a `LazilyParsedNumber`. Java throws
    /// `UnsupportedOperationException` for a boolean.
    pub fn get_as_number(&self) -> Number {
        match self {
            JsonPrimitive::Number(number) => number.clone(),
            JsonPrimitive::String(string) => {
                Number::LazilyParsedNumber(LazilyParsedNumber::new(string.clone()))
            }
            JsonPrimitive::Boolean(_) => panic!(
                "java.lang.UnsupportedOperationException: Primitive is neither a number nor a string"
            ),
        }
    }

    pub fn is_string(&self) -> bool {
        matches!(self, JsonPrimitive::String(_))
    }

    /// The string, a number's `toString()`, or a boolean's.
    pub fn get_as_string(&self) -> String {
        match self {
            JsonPrimitive::String(string) => string.clone(),
            JsonPrimitive::Number(number) => number.to_string(),
            JsonPrimitive::Boolean(value) => value.to_string(),
        }
    }

    /// A number's `doubleValue()`, or `Double.parseDouble` of the text.
    pub fn get_as_double(&self) -> f64 {
        match self {
            JsonPrimitive::Number(number) => number.double_value(),
            _ => Double::parse_double(&self.get_as_string())
                .unwrap_or_else(|error| panic!("{error}")),
        }
    }

    /// A `BigDecimal` itself, or `NumberLimits.parseBigDecimal` of the text: a number's
    /// `toString()`, so a double's shortest digits, and NaN, infinities and booleans don't parse.
    pub fn get_as_big_decimal(&self) -> BigDecimal {
        match self {
            JsonPrimitive::Number(Number::BigDecimal(decimal)) => decimal.clone(),
            _ => NumberLimits::parse_big_decimal(&self.get_as_string())
                .unwrap_or_else(|error| panic!("{error}")),
        }
    }

    /// A number's `floatValue()`, or `Float.parseFloat` of the text.
    pub fn get_as_float(&self) -> f32 {
        match self {
            JsonPrimitive::Number(number) => number.float_value(),
            _ => {
                Float::parse_float(&self.get_as_string()).unwrap_or_else(|error| panic!("{error}"))
            }
        }
    }

    /// A number's `longValue()`, or `Long.parseLong` of the text.
    pub fn get_as_long(&self) -> i64 {
        match self {
            JsonPrimitive::Number(number) => number.long_value(),
            _ => Long::parse_long(&self.get_as_string()).unwrap_or_else(|error| panic!("{error}")),
        }
    }

    /// A number's `shortValue()`, or `Short.parseShort` of the text.
    pub fn get_as_short(&self) -> i16 {
        match self {
            JsonPrimitive::Number(number) => number.short_value(),
            _ => {
                Short::parse_short(&self.get_as_string()).unwrap_or_else(|error| panic!("{error}"))
            }
        }
    }

    /// A number's `intValue()`, or `Integer.parseInt` of the text.
    pub fn get_as_int(&self) -> i32 {
        match self {
            JsonPrimitive::Number(number) => number.int_value(),
            _ => {
                Integer::parse_int(&self.get_as_string()).unwrap_or_else(|error| panic!("{error}"))
            }
        }
    }

    /// A number's `byteValue()`, or `Byte.parseByte` of the text.
    pub fn get_as_byte(&self) -> i8 {
        match self {
            JsonPrimitive::Number(number) => number.byte_value(),
            _ => Byte::parse_byte(&self.get_as_string()).unwrap_or_else(|error| panic!("{error}")),
        }
    }

    /// `hashCode()`, Effective Java's for longs and doubles: an integral number's `longValue()`,
    /// any other number's `doubleValue()` bits (NaNs made one), a string's or a boolean's own.
    pub fn hash_code(&self) -> i32 {
        match self {
            JsonPrimitive::Number(number) if is_integral(number) => {
                let value = number.long_value();
                (value ^ ((value as u64) >> 32) as i64) as i32
            }
            JsonPrimitive::Number(number) => {
                let value = Double::double_to_long_bits(number.double_value());
                (value ^ ((value as u64) >> 32) as i64) as i32
            }
            JsonPrimitive::String(string) => string.hash_code(),
            // `Boolean.hashCode`.
            JsonPrimitive::Boolean(value) => {
                if *value {
                    1231
                } else {
                    1237
                }
            }
        }
    }
}

/// `equals`: integral numbers (`Long`, `Integer`, `Short`, `Byte`) by `longValue()`, two
/// `BigDecimal`s by `compareTo` (so `1.0` is `1.00`), other numbers by `doubleValue()` with
/// `-0.0 == 0.0` and NaN equal to NaN; strings and booleans by value, and nothing else across
/// kinds. Two `LazilyParsedNumber`s of the same text are equal first, Java's identity check.
impl PartialEq for JsonPrimitive {
    fn eq(&self, other: &JsonPrimitive) -> bool {
        if same_value(self, other) {
            return true;
        }
        match (self, other) {
            (JsonPrimitive::Number(this), JsonPrimitive::Number(other)) => {
                if is_integral(this) && is_integral(other) {
                    return this.long_value() == other.long_value();
                }
                if let (Number::BigDecimal(this), Number::BigDecimal(other)) = (this, other) {
                    return this.compare_to(other) == Ordering::Equal;
                }
                let this_as_double = this.double_value();
                let other_as_double = other.double_value();
                // Not `Double.compare`, which tells -0.0 and 0.0 apart.
                this_as_double == other_as_double
                    || (this_as_double.is_nan() && other_as_double.is_nan())
            }
            (JsonPrimitive::String(this), JsonPrimitive::String(other)) => this == other,
            (JsonPrimitive::Boolean(this), JsonPrimitive::Boolean(other)) => this == other,
            _ => false,
        }
    }
}

/// Two `LazilyParsedNumber`s of the same text, which a clone of one Java reference holds: Java
/// returns true for the same object before it converts a text that may not be a number. Any other
/// value compares the same whether it's one object or two.
fn same_value(this: &JsonPrimitive, other: &JsonPrimitive) -> bool {
    match (this, other) {
        (
            JsonPrimitive::Number(Number::LazilyParsedNumber(this)),
            JsonPrimitive::Number(Number::LazilyParsedNumber(other)),
        ) => this == other,
        _ => false,
    }
}

/// Whether the number is a `Long`, `Integer`, `Short` or `Byte` (or Java's `BigInteger`, which
/// isn't ported).
fn is_integral(number: &Number) -> bool {
    matches!(
        number,
        Number::Long(_) | Number::Integer(_) | Number::Short(_) | Number::Byte(_)
    )
}

#[cfg(test)]
mod tests {
    use super::JsonPrimitive;
    use crate::com::google::gson::json_element::JsonElement;
    use crate::com::google::gson::json_element::tests::{simple_name, thrown};
    use crate::golden::{Encode, Golden};

    fn primitive(element: JsonElement) -> JsonPrimitive {
        element.get_as_json_primitive().clone()
    }

    /// Checks a getter, and its exceptions' classes and messages where vanilla threw.
    fn check<R: Encode + 'static>(
        golden: &Golden,
        method: &str,
        getter: impl Fn(&JsonPrimitive) -> R + Copy,
    ) {
        golden.check(method, move |element: JsonElement| {
            getter(&primitive(element))
        });
        golden.check(&format!("{method} message"), move |element: JsonElement| {
            let primitive = primitive(element);
            thrown(move || {
                getter(&primitive);
            })
        });
    }

    /// The getters, `hashCode` and `equals` on primitives of every box (parsed numbers among them,
    /// valid and not) and numeric and other strings, and what `getAsBigDecimal()` hands out.
    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("com.google.gson.JsonPrimitive");
        golden.check("isBoolean()Z", |element: JsonElement| {
            primitive(element).is_boolean()
        });
        golden.check("isNumber()Z", |element: JsonElement| {
            primitive(element).is_number()
        });
        golden.check("isString()Z", |element: JsonElement| {
            primitive(element).is_string()
        });
        golden.check("getAsBoolean()Z", |element: JsonElement| {
            primitive(element).get_as_boolean()
        });
        golden.check("getAsString()Ljava/lang/String;", |element: JsonElement| {
            primitive(element).get_as_string()
        });
        check(&golden, "getAsNumber()Ljava/lang/Number;", |primitive| {
            let number = primitive.get_as_number();
            (simple_name(&number).to_owned(), number.to_string())
        });
        check(&golden, "getAsDouble()D", JsonPrimitive::get_as_double);
        check(&golden, "getAsFloat()F", JsonPrimitive::get_as_float);
        check(&golden, "getAsLong()J", JsonPrimitive::get_as_long);
        check(&golden, "getAsInt()I", JsonPrimitive::get_as_int);
        check(&golden, "getAsShort()S", JsonPrimitive::get_as_short);
        check(&golden, "getAsByte()B", JsonPrimitive::get_as_byte);
        check(
            &golden,
            "getAsBigDecimal()Ljava/math/BigDecimal;",
            |primitive| primitive.get_as_big_decimal().to_string(),
        );
        check(&golden, "getAsBigDecimal().intValue()I", |primitive| {
            primitive.get_as_big_decimal().int_value()
        });
        check(&golden, "getAsBigDecimal().longValue()J", |primitive| {
            primitive.get_as_big_decimal().long_value()
        });
        check(
            &golden,
            "getAsBigDecimal().longValueExact()J",
            |primitive| {
                primitive
                    .get_as_big_decimal()
                    .long_value_exact()
                    .unwrap_or_else(|error| panic!("{error}"))
            },
        );
        check(&golden, "getAsBigDecimal().doubleValue()D", |primitive| {
            primitive.get_as_big_decimal().double_value()
        });
        check(&golden, "getAsBigDecimal().floatValue()F", |primitive| {
            primitive.get_as_big_decimal().float_value()
        });
        check(&golden, "getAsBigDecimal().scale()I", |primitive| {
            primitive.get_as_big_decimal().scale()
        });
        check(&golden, "hashCode()I", JsonPrimitive::hash_code);
        golden.check(
            "equals(Ljava/lang/Object;)Z",
            |first: JsonElement, second: JsonElement| primitive(first) == primitive(second),
        );
        golden.check(
            "equals(Ljava/lang/Object;)Z message",
            |first: JsonElement, second: JsonElement| {
                let (first, second) = (primitive(first), primitive(second));
                thrown(move || {
                    let _ = first == second;
                })
            },
        );
        golden.finish();
    }
}
