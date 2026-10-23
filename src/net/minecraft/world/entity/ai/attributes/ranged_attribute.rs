//! net/minecraft/world/entity/ai/attributes/RangedAttribute.java
//!
//! The range an `Attribute` holds. The constructor's checks (a minimum past the maximum, a
//! default outside the range) only fail for a programming mistake, so the constant attributes
//! are checked by a test.

use crate::net::minecraft::util::mth::Mth;

#[derive(Clone, Copy, Debug)]
pub struct RangedAttribute {
    min_value: f64,
    max_value: f64,
}

impl RangedAttribute {
    pub(crate) const fn new(default_value: f64, min_value: f64, max_value: f64) -> RangedAttribute {
        let _ = default_value;
        RangedAttribute {
            min_value,
            max_value,
        }
    }

    pub fn get_min_value(&self) -> f64 {
        self.min_value
    }

    pub fn get_max_value(&self) -> f64 {
        self.max_value
    }

    /// NaN becomes the minimum, and anything else is clamped to the range.
    pub fn sanitize_value(&self, value: f64) -> f64 {
        if value.is_nan() {
            self.min_value
        } else {
            Mth::clamp_f64(value, self.min_value, self.max_value)
        }
    }
}
