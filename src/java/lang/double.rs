//! java.lang.Double, where it differs from Rust's std: what `equals` and `hashCode` methods use.

use crate::jdk::internal::math::double_to_decimal;

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
