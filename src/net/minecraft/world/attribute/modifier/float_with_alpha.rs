//! net/minecraft/world/attribute/modifier/FloatWithAlpha.java

use crate::com::google::gson::json_element::JsonElement;
use crate::net::minecraft::world::attribute::attribute_type::decode_float;

/// A float to blend toward, and how far.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloatWithAlpha {
    pub value: f32,
    pub alpha: f32,
}

impl FloatWithAlpha {
    pub fn new(value: f32, alpha: f32) -> FloatWithAlpha {
        FloatWithAlpha { value, alpha }
    }

    /// `FloatWithAlpha(float)`: all the way.
    pub fn of(value: f32) -> FloatWithAlpha {
        FloatWithAlpha::new(value, 1.0)
    }

    /// `CODEC`: a float, or `{"value", "alpha"}` with the alpha in 0..1, 1 when it's missing.
    pub fn from_json(json: &JsonElement) -> Result<FloatWithAlpha, String> {
        if let Ok(value) = decode_float(json) {
            return Ok(FloatWithAlpha::of(value));
        }
        let object = json
            .as_json_object()
            .ok_or_else(|| format!("Not a float or an object: {}", json.to_json_string()))?;
        let value = decode_float(object.get("value").ok_or("No key value in MapLike")?)?;
        let alpha = match object.get("alpha") {
            Some(alpha) => decode_float(alpha)?,
            None => 1.0,
        };
        if !(0.0..=1.0).contains(&alpha) {
            return Err(format!("Value {alpha} outside of range [0.0:1.0]"));
        }
        Ok(FloatWithAlpha::new(value, alpha))
    }
}
