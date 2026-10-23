//! java/lang/Number.java
//!
//! A boxed number, which NBT's numeric tags hand out (`NumericTag.box`, `Tag.asNumber`) and
//! `DynamicOps.getNumberValue` reads: the box is the tag's own kind, a `Byte` for a `ByteTag`.
//! Its conversions are each box's, Java's narrowing primitive conversions: a float or double
//! goes to an int or long first (saturating, NaN to 0), then wraps to a byte or short.

use std::fmt;

use crate::java::lang::double::Double;
use crate::java::lang::float::Float;

/// One of the boxes of Java's primitive numbers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Number {
    Byte(i8),
    Short(i16),
    Integer(i32),
    Long(i64),
    Float(f32),
    Double(f64),
}

impl Number {
    pub fn byte_value(&self) -> i8 {
        self.int_value() as i8
    }

    pub fn short_value(&self) -> i16 {
        self.int_value() as i16
    }

    pub fn int_value(&self) -> i32 {
        match *self {
            Number::Byte(value) => i32::from(value),
            Number::Short(value) => i32::from(value),
            Number::Integer(value) => value,
            Number::Long(value) => value as i32,
            Number::Float(value) => value as i32,
            Number::Double(value) => value as i32,
        }
    }

    pub fn long_value(&self) -> i64 {
        match *self {
            Number::Byte(value) => i64::from(value),
            Number::Short(value) => i64::from(value),
            Number::Integer(value) => i64::from(value),
            Number::Long(value) => value,
            Number::Float(value) => value as i64,
            Number::Double(value) => value as i64,
        }
    }

    pub fn float_value(&self) -> f32 {
        match *self {
            Number::Byte(value) => f32::from(value),
            Number::Short(value) => f32::from(value),
            Number::Integer(value) => value as f32,
            Number::Long(value) => value as f32,
            Number::Float(value) => value,
            Number::Double(value) => value as f32,
        }
    }

    pub fn double_value(&self) -> f64 {
        match *self {
            Number::Byte(value) => f64::from(value),
            Number::Short(value) => f64::from(value),
            Number::Integer(value) => f64::from(value),
            Number::Long(value) => value as f64,
            Number::Float(value) => f64::from(value),
            Number::Double(value) => value,
        }
    }
}

/// Each box's `toString`: the integer, or `Float.toString` and `Double.toString`.
impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Number::Byte(value) => write!(f, "{value}"),
            Number::Short(value) => write!(f, "{value}"),
            Number::Integer(value) => write!(f, "{value}"),
            Number::Long(value) => write!(f, "{value}"),
            Number::Float(value) => f.write_str(&Float::to_string(value)),
            Number::Double(value) => f.write_str(&Double::to_string(value)),
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
        };
        Box::new((Value::from(name), *number))
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

    // By its own class, like vanilla: an integer, or a float's or double's bits.
    impl Encode for Number {
        fn encode(&self) -> Value {
            match *self {
                Number::Byte(value) => value.encode(),
                Number::Short(value) => value.encode(),
                Number::Integer(value) => value.encode(),
                Number::Long(value) => value.encode(),
                Number::Float(value) => value.encode(),
                Number::Double(value) => value.encode(),
            }
        }

        fn matches(&self, expected: &Value) -> bool {
            match *self {
                Number::Byte(value) => value.matches(expected),
                Number::Short(value) => value.matches(expected),
                Number::Integer(value) => value.matches(expected),
                Number::Long(value) => value.matches(expected),
                Number::Float(value) => value.matches(expected),
                Number::Double(value) => value.matches(expected),
            }
        }
    }
}
