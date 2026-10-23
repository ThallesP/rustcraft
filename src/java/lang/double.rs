//! java.lang.Double, where it differs from Rust's std: what `equals` and `hashCode` methods use,
//! `compare`, `toString`, and `parseDouble`, whose grammar is Java's (surrounding whitespace, `NaN`,
//! `Infinity`, hexadecimal, a type suffix).

use crate::java::lang::number_format_exception::NumberFormatException;
use crate::jdk::internal::math::{double_to_decimal, floating_decimal};

pub struct Double;

impl Double {
    /// The bits, with every NaN made the canonical one, so all NaNs compare and hash the same.
    /// `doubleToRawLongBits` is `to_bits`.
    pub fn double_to_long_bits(value: f64) -> i64 {
        if value.is_nan() {
            0x7ff8000000000000
        } else {
            value.to_bits() as i64
        }
    }

    /// `compare(double, double)`, `compareTo`'s order: by value, -0.0 below 0.0, and every NaN
    /// equal to every other and above everything else.
    pub fn compare(d1: f64, d2: f64) -> std::cmp::Ordering {
        if d1 < d2 {
            return std::cmp::Ordering::Less;
        }
        if d1 > d2 {
            return std::cmp::Ordering::Greater;
        }
        Double::double_to_long_bits(d1).cmp(&Double::double_to_long_bits(d2))
    }

    /// The two halves of the bits, XORed.
    pub fn hash_code(value: f64) -> i32 {
        let bits = Double::double_to_long_bits(value);
        (bits ^ ((bits as u64) >> 32) as i64) as i32
    }

    /// `toString(double)`: the shortest decimal that rounds back to the double, in Java's format
    /// (`1.0`, `0.001`, `1.0E7`, `1.0E-4`, `NaN`).
    pub fn to_string(value: f64) -> String {
        double_to_decimal::to_string(value)
    }

    /// `parseDouble(String)`: the decimal or hexadecimal value, correctly rounded.
    pub fn parse_double(s: &str) -> Result<f64, NumberFormatException> {
        floating_decimal::parse_double(s)
    }
}

#[cfg(test)]
mod tests {
    use super::Double;
    use crate::golden::Golden;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("java.lang.Double");
        golden.check("doubleToLongBits(D)J", Double::double_to_long_bits);
        golden.check("hashCode(D)I", Double::hash_code);
        golden.check("toString(D)Ljava/lang/String;", Double::to_string);
        golden.finish();
    }
}
