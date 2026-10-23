//! java/lang/Number.java
//!
//! A boxed number, which NBT's numeric tags hand out (`NumericTag.box`, `Tag.asNumber`),
//! `DynamicOps.getNumberValue` reads, and Gson's `JsonPrimitive` holds: the box is the value's own
//! class, a `Byte` for a `ByteTag`, a `LazilyParsedNumber` for a number Gson read (its text), or a
//! `BigDecimal`. Its conversions are each class's: the primitive boxes' are Java's narrowing
//! primitive conversions (a float or double goes to an int or long first, saturating, NaN to 0,
//! then wraps to a byte or short); `LazilyParsedNumber` parses its text, panicking where Java
//! throws `NumberFormatException`; and `byteValue` and `shortValue` are `Number`'s defaults,
//! `(byte) intValue()`, for the classes that don't override them.

use std::fmt;

use crate::com::google::gson::internal::lazily_parsed_number::LazilyParsedNumber;
use crate::java::lang::double::Double;
use crate::java::lang::float::Float;
use crate::java::math::big_decimal::BigDecimal;

/// One of the `Number` classes the game and Gson hand out. Not `Copy`, since a
/// `LazilyParsedNumber` holds its text and a `BigDecimal` its digits.
#[derive(Clone, Debug)]
pub enum Number {
    Byte(i8),
    Short(i16),
    Integer(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    LazilyParsedNumber(LazilyParsedNumber),
    BigDecimal(BigDecimal),
}

impl Number {
    pub fn byte_value(&self) -> i8 {
        match *self {
            Number::Byte(value) => value,
            _ => self.int_value() as i8,
        }
    }

    pub fn short_value(&self) -> i16 {
        match *self {
            Number::Short(value) => value,
            _ => self.int_value() as i16,
        }
    }

    pub fn int_value(&self) -> i32 {
        match self {
            Number::Byte(value) => i32::from(*value),
            Number::Short(value) => i32::from(*value),
            Number::Integer(value) => *value,
            Number::Long(value) => *value as i32,
            Number::Float(value) => *value as i32,
            Number::Double(value) => *value as i32,
            Number::LazilyParsedNumber(value) => value.int_value(),
            Number::BigDecimal(value) => value.int_value(),
        }
    }

    pub fn long_value(&self) -> i64 {
        match self {
            Number::Byte(value) => i64::from(*value),
            Number::Short(value) => i64::from(*value),
            Number::Integer(value) => i64::from(*value),
            Number::Long(value) => *value,
            Number::Float(value) => *value as i64,
            Number::Double(value) => *value as i64,
            Number::LazilyParsedNumber(value) => value.long_value(),
            Number::BigDecimal(value) => value.long_value(),
        }
    }

    pub fn float_value(&self) -> f32 {
        match self {
            Number::Byte(value) => f32::from(*value),
            Number::Short(value) => f32::from(*value),
            Number::Integer(value) => *value as f32,
            Number::Long(value) => *value as f32,
            Number::Float(value) => *value,
            Number::Double(value) => *value as f32,
            Number::LazilyParsedNumber(value) => value.float_value(),
            Number::BigDecimal(value) => value.float_value(),
        }
    }

    pub fn double_value(&self) -> f64 {
        match self {
            Number::Byte(value) => f64::from(*value),
            Number::Short(value) => f64::from(*value),
            Number::Integer(value) => f64::from(*value),
            Number::Long(value) => *value as f64,
            Number::Float(value) => f64::from(*value),
            Number::Double(value) => *value,
            Number::LazilyParsedNumber(value) => value.double_value(),
            Number::BigDecimal(value) => value.double_value(),
        }
    }

    /// `getClass().getName()`, which `JsonWriter` names in its message about a number that
    /// doesn't print as JSON.
    pub fn class_name(&self) -> &'static str {
        match self {
            Number::Byte(_) => "java.lang.Byte",
            Number::Short(_) => "java.lang.Short",
            Number::Integer(_) => "java.lang.Integer",
            Number::Long(_) => "java.lang.Long",
            Number::Float(_) => "java.lang.Float",
            Number::Double(_) => "java.lang.Double",
            Number::LazilyParsedNumber(_) => "com.google.gson.internal.LazilyParsedNumber",
            Number::BigDecimal(_) => "java.math.BigDecimal",
        }
    }
}

/// Each class's `equals`: the same class and value, floats and doubles by their bits with NaNs
/// made one (so `-0.0` isn't `0.0`, and NaN is NaN), a `LazilyParsedNumber` by its text, and a
/// `BigDecimal` by its value and scale.
impl PartialEq for Number {
    fn eq(&self, other: &Number) -> bool {
        match (self, other) {
            (Number::Byte(a), Number::Byte(b)) => a == b,
            (Number::Short(a), Number::Short(b)) => a == b,
            (Number::Integer(a), Number::Integer(b)) => a == b,
            (Number::Long(a), Number::Long(b)) => a == b,
            (Number::Float(a), Number::Float(b)) => {
                Float::float_to_int_bits(*a) == Float::float_to_int_bits(*b)
            }
            (Number::Double(a), Number::Double(b)) => {
                Double::double_to_long_bits(*a) == Double::double_to_long_bits(*b)
            }
            (Number::LazilyParsedNumber(a), Number::LazilyParsedNumber(b)) => a == b,
            (Number::BigDecimal(a), Number::BigDecimal(b)) => a == b,
            _ => false,
        }
    }
}

/// Each class's `toString`: the integer, `Float.toString` and `Double.toString`, a
/// `LazilyParsedNumber`'s text, a `BigDecimal`'s scientific form.
impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Number::Byte(value) => write!(f, "{value}"),
            Number::Short(value) => write!(f, "{value}"),
            Number::Integer(value) => write!(f, "{value}"),
            Number::Long(value) => write!(f, "{value}"),
            Number::Float(value) => f.write_str(&Float::to_string(*value)),
            Number::Double(value) => f.write_str(&Double::to_string(*value)),
            Number::LazilyParsedNumber(value) => write!(f, "{value}"),
            Number::BigDecimal(value) => write!(f, "{value}"),
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use serde_json::Value;

    use super::Number;
    use crate::golden::{Decode, Encode};

    /// `[box, value]`, the box's simple name and the number by its class, as the codec suites
    /// record a number whose box matters (`DfuValues.number`).
    pub(crate) fn encode_boxed(number: &Number) -> Box<dyn Encode> {
        let name = match number {
            Number::Byte(_) => "Byte",
            Number::Short(_) => "Short",
            Number::Integer(_) => "Integer",
            Number::Long(_) => "Long",
            Number::Float(_) => "Float",
            Number::Double(_) => "Double",
            Number::LazilyParsedNumber(_) => "LazilyParsedNumber",
            Number::BigDecimal(_) => "BigDecimal",
        };
        Box::new((Value::from(name), number.clone()))
    }

    pub(crate) fn decode_boxed(spec: &Value) -> Number {
        let spec = spec.as_array().unwrap();
        match spec[0].as_str().unwrap() {
            "Byte" => Number::Byte(i8::decode(&spec[1])),
            "Short" => Number::Short(i16::decode(&spec[1])),
            "Integer" => Number::Integer(i32::decode(&spec[1])),
            "Long" => Number::Long(i64::decode(&spec[1])),
            "Float" => Number::Float(f32::decode(&spec[1])),
            _ => Number::Double(f64::decode(&spec[1])),
        }
    }

    // By its own class, like vanilla: an integer, or a float's or double's bits. Gson's numbers,
    // which no codec suite records, are their text.
    impl Encode for Number {
        fn encode(&self) -> Value {
            match self {
                Number::Byte(value) => value.encode(),
                Number::Short(value) => value.encode(),
                Number::Integer(value) => value.encode(),
                Number::Long(value) => value.encode(),
                Number::Float(value) => value.encode(),
                Number::Double(value) => value.encode(),
                Number::LazilyParsedNumber(_) | Number::BigDecimal(_) => {
                    Value::from(self.to_string())
                }
            }
        }

        fn matches(&self, expected: &Value) -> bool {
            match self {
                Number::Byte(value) => value.matches(expected),
                Number::Short(value) => value.matches(expected),
                Number::Integer(value) => value.matches(expected),
                Number::Long(value) => value.matches(expected),
                Number::Float(value) => value.matches(expected),
                Number::Double(value) => value.matches(expected),
                Number::LazilyParsedNumber(_) | Number::BigDecimal(_) => self.encode() == *expected,
            }
        }
    }
}
