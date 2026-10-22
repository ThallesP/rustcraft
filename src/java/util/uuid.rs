//! java.util.UUID.

use std::fmt;

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

    /// Java's `hashCode()`: the halves of both longs, xored.
    pub fn hash_code(&self) -> i32 {
        let hilo = self.most_sig_bits ^ self.least_sig_bits;
        ((hilo >> 32) as i32) ^ (hilo as i32)
    }
}

/// Lowercase hex in groups of 8, 4, 4, 4 and 12 digits.
impl fmt::Display for UUID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let most = self.most_sig_bits as u64;
        let least = self.least_sig_bits as u64;
        write!(
            f,
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            most >> 32,
            (most >> 16) & 0xffff,
            most & 0xffff,
            least >> 48,
            least & 0xffff_ffff_ffff
        )
    }
}

#[cfg(test)]
mod golden {
    use serde_json::Value;

    use super::UUID;
    use crate::golden::{Decode, Encode, Golden, elements};

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

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("java.util.UUID");
        golden.check_method("hashCode()I", |this: &mut UUID| this.hash_code());
        golden.check_method("toString()Ljava/lang/String;", |this: &mut UUID| {
            this.to_string()
        });
        golden.finish();
    }
}
