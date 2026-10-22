//! java.lang.Float, where it differs from Rust's std: what `equals` and `hashCode` methods use.

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

    pub fn hash_code(value: f32) -> i32 {
        Float::float_to_int_bits(value)
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
        golden.finish();
    }
}
