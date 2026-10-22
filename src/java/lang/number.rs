//! java/lang/Number.java
//!
//! A boxed number, which NBT's numeric tags hand out (`NumericTag.box`, `Tag.asNumber`): the box
//! is the tag's own kind, a `Byte` for a `ByteTag`. Only the boxes are here, not `Number`'s
//! conversions, which nothing ported calls.

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

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::Number;
    use crate::golden::Encode;

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
