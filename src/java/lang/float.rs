//! java.lang.Float, where it differs from Rust's std: what `equals` and `hashCode` methods use,
//! `toString`, and `parseFloat`, whose grammar is Java's (surrounding whitespace, `NaN`,
//! `Infinity`, hexadecimal, a type suffix).

use crate::java::lang::number_format_exception::NumberFormatException;
use crate::jdk::internal::math::{float_to_decimal, floating_decimal};

pub struct Float;

impl Float {
    /// The bits, with every NaN made the canonical one, so all NaNs compare and hash the same.
    /// `floatToRawIntBits` is `to_bits`.
    pub fn float_to_int_bits(value: f32) -> i32 {
        if value.is_nan() {
            0x7fc00000
        } else {
            value.to_bits() as i32
        }
    }

    /// `Float.compare`: numerically, with -0.0 below 0.0 and every NaN above everything and
    /// equal to itself.
    pub fn compare(f1: f32, f2: f32) -> std::cmp::Ordering {
        if f1 < f2 {
            return std::cmp::Ordering::Less;
        }
        if f1 > f2 {
            return std::cmp::Ordering::Greater;
        }
        Float::float_to_int_bits(f1).cmp(&Float::float_to_int_bits(f2))
    }

    pub fn hash_code(value: f32) -> i32 {
        Float::float_to_int_bits(value)
    }

    /// `toString(float)`: the shortest decimal that rounds back to the float, in Java's format
    /// (`1.0`, `0.001`, `1.0E7`, `1.0E-4`, `NaN`).
    pub fn to_string(value: f32) -> String {
        float_to_decimal::to_string(value)
    }

    /// `parseFloat(String)`: the decimal or hexadecimal value, rounded straight to a float.
    pub fn parse_float(s: &str) -> Result<f32, NumberFormatException> {
        floating_decimal::parse_float(s)
    }
}

#[cfg(test)]
mod tests {
    use super::Float;
    use crate::golden::Golden;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("java.lang.Float");
        golden.check("floatToIntBits(F)I", Float::float_to_int_bits);
        golden.check("hashCode(F)I", Float::hash_code);
        golden.check("toString(F)Ljava/lang/String;", Float::to_string);
        golden.finish();
    }
}
