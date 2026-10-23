//! net/minecraft/world/attribute/AttributeRange.java
//!
//! The values an attribute takes: any, or floats from a minimum to a maximum. Java's interface
//! is implemented only by `any()` and `ofFloat`'s anonymous classes, so it's an enum of them.

use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::attribute::attribute_type::AttributeValue;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AttributeRange {
    Any,
    Float { min_value: f32, max_value: f32 },
}

impl AttributeRange {
    pub const UNIT_FLOAT: AttributeRange = AttributeRange::of_float(0.0, 1.0);
    pub const UNIT_FLOAT_EPSILON: AttributeRange = AttributeRange::of_float(0.0, 0.9999999);
    pub const NON_NEGATIVE_FLOAT: AttributeRange = AttributeRange::of_float(0.0, f32::INFINITY);

    pub const fn of_float(min_value: f32, max_value: f32) -> AttributeRange {
        AttributeRange::Float {
            min_value,
            max_value,
        }
    }

    /// Whether data may hold the value, `Err` with Java's message where its codec fails.
    pub fn validate(&self, value: &AttributeValue) -> Result<(), String> {
        match self {
            AttributeRange::Any => Ok(()),
            AttributeRange::Float {
                min_value,
                max_value,
            } => {
                let value = value.as_float();
                if value >= *min_value && value <= *max_value {
                    Ok(())
                } else {
                    Err(format!(
                        "{value} is not in range [{min_value}; {max_value}]"
                    ))
                }
            }
        }
    }

    /// The value clamped into the range: `Mth.clamp`, which keeps a NaN.
    pub fn sanitize(&self, value: AttributeValue) -> AttributeValue {
        match self {
            AttributeRange::Any => value,
            AttributeRange::Float {
                min_value,
                max_value,
            } => {
                let float = value.as_float();
                if float >= *min_value && float <= *max_value {
                    value
                } else {
                    AttributeValue::Float(Mth::clamp_f32(float, *min_value, *max_value))
                }
            }
        }
    }
}
