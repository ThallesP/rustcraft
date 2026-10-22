use serde_json::{Map, Value};

/// A float written as its raw IEEE 754 bits, the way TESTING.md encodes every float.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FloatBits {
    F32(u32),
    F64(u64),
}

impl FloatBits {
    /// Reads 8 or 16 lowercase hex digits as float bits; anything else isn't a float.
    pub fn parse(value: &Value) -> Option<Self> {
        let text = value.as_str()?;
        if !text
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
        {
            return None;
        }
        match text.len() {
            8 => u32::from_str_radix(text, 16).ok().map(FloatBits::F32),
            16 => u64::from_str_radix(text, 16).ok().map(FloatBits::F64),
            _ => None,
        }
    }

    fn as_f64(self) -> f64 {
        match self {
            FloatBits::F32(bits) => f64::from(f32::from_bits(bits)),
            FloatBits::F64(bits) => f64::from_bits(bits),
        }
    }

    fn is_nan(self) -> bool {
        self.as_f64().is_nan()
    }

    // Maps the bits onto a line where neighbouring floats are neighbouring integers.
    fn ordered(self) -> i128 {
        let (bits, sign_bit) = match self {
            FloatBits::F32(bits) => (i128::from(bits), 1i128 << 31),
            FloatBits::F64(bits) => (i128::from(bits), 1i128 << 63),
        };
        if bits & sign_bit == 0 {
            bits
        } else {
            sign_bit - bits
        }
    }
}

/// Distance between two floats of the same width in units in the last place.
pub fn ulps(a: FloatBits, b: FloatBits) -> Option<u128> {
    let same_width = matches!(
        (a, b),
        (FloatBits::F32(_), FloatBits::F32(_)) | (FloatBits::F64(_), FloatBits::F64(_))
    );
    if !same_width || a.is_nan() || b.is_nan() {
        return None;
    }
    Some(a.ordered().abs_diff(b.ordered()))
}

/// A value for humans: floats show their decimal value next to the bits.
pub fn describe(value: Option<&Value>) -> String {
    let Some(value) = value else {
        return "(missing)".to_string();
    };
    match FloatBits::parse(value) {
        Some(FloatBits::F32(bits)) => format!("{:?} ({bits:08x})", f32::from_bits(bits)),
        Some(FloatBits::F64(bits)) => format!("{:?} ({bits:016x})", f64::from_bits(bits)),
        None => value.to_string(),
    }
}

/// Flattens arrays and objects into `path → leaf`, e.g. `pos[1]`, so differences show per element.
pub fn flatten(value: &Value) -> Map<String, Value> {
    let mut leaves = Map::new();
    flatten_into(value, String::new(), &mut leaves);
    leaves
}

fn flatten_into(value: &Value, path: String, leaves: &mut Map<String, Value>) {
    match value {
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                flatten_into(item, format!("{path}[{index}]"), leaves);
            }
        }
        Value::Object(fields) => {
            for (key, field) in fields {
                let path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                flatten_into(field, path, leaves);
            }
        }
        leaf => {
            leaves.insert(path, leaf.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn parses_float_bits_and_leaves_other_strings_alone() {
        assert_eq!(
            FloatBits::parse(&json!("3f800000")),
            Some(FloatBits::F32(0x3f80_0000))
        );
        assert_eq!(
            FloatBits::parse(&json!("3ff0000000000000")),
            Some(FloatBits::F64(0x3ff0_0000_0000_0000))
        );
        assert_eq!(FloatBits::parse(&json!("STANDING")), None);
        assert_eq!(FloatBits::parse(&json!("3F800000")), None);
        assert_eq!(FloatBits::parse(&json!(1)), None);
    }

    #[test]
    fn counts_ulps_across_zero() {
        let one = FloatBits::F32(1.0f32.to_bits());
        let next = FloatBits::F32(1.0f32.next_up().to_bits());
        assert_eq!(ulps(one, next), Some(1));
        assert_eq!(
            ulps(
                FloatBits::F64(0.0f64.to_bits()),
                FloatBits::F64((-0.0f64).to_bits())
            ),
            Some(0)
        );
        assert_eq!(
            ulps(
                FloatBits::F32(f32::from_bits(1).to_bits()),
                FloatBits::F32((-f32::from_bits(1)).to_bits())
            ),
            Some(2)
        );
        assert_eq!(ulps(one, FloatBits::F64(1.0f64.to_bits())), None);
    }

    #[test]
    fn describes_floats_with_their_bits() {
        assert_eq!(describe(Some(&json!("3f800000"))), "1.0 (3f800000)");
        assert_eq!(describe(Some(&json!(true))), "true");
        assert_eq!(describe(None), "(missing)");
    }

    #[test]
    fn flattens_nested_values() {
        let leaves = flatten(&json!({ "pos": ["a", "b"], "on_ground": true }));
        assert_eq!(leaves["pos[1]"], json!("b"));
        assert_eq!(leaves["on_ground"], json!(true));
    }
}
