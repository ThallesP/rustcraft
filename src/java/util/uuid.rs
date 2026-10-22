//! java.util.UUID.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UUID {
    most_sig_bits: i64,
    least_sig_bits: i64,
}

impl UUID {
    pub const fn new(most_sig_bits: i64, least_sig_bits: i64) -> UUID {
        UUID {
            most_sig_bits,
            least_sig_bits,
        }
    }

    pub const fn get_most_significant_bits(&self) -> i64 {
        self.most_sig_bits
    }

    pub const fn get_least_significant_bits(&self) -> i64 {
        self.least_sig_bits
    }
}

#[cfg(test)]
mod golden {
    use serde_json::Value;

    use super::UUID;
    use crate::golden::{Decode, Encode, elements};

    // Vanilla writes a UUID as [most, least].
    impl Decode for UUID {
        fn decode(value: &Value) -> Self {
            let [most, least] = elements(value);
            UUID::new(i64::decode(most), i64::decode(least))
        }
    }

    impl Encode for UUID {
        fn encode(&self) -> Value {
            Value::from(vec![self.most_sig_bits, self.least_sig_bits])
        }
    }
}
