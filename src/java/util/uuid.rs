//! java.util.UUID.

use std::fmt;

use md5::{Digest, Md5};

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

    /// `nameUUIDFromBytes(byte[])`: a version 3 UUID, the MD5 of the name with its version and
    /// variant bits set.
    pub fn name_uuid_from_bytes(name: &[i8]) -> UUID {
        let name: Vec<u8> = name.iter().map(|&byte| byte as u8).collect();
        let mut md5_bytes: [u8; 16] = Md5::digest(name).into();
        md5_bytes[6] &= 0x0f; // clear version
        md5_bytes[6] |= 0x30; // set to version 3
        md5_bytes[8] &= 0x3f; // clear variant
        md5_bytes[8] |= 0x80; // set to IETF variant
        UUID::from_data(md5_bytes)
    }

    /// The private `UUID(byte[])`: the most significant long from the first 8 bytes, big-endian,
    /// and the least from the rest.
    fn from_data(data: [u8; 16]) -> UUID {
        let mut msb: i64 = 0;
        let mut lsb: i64 = 0;
        for byte in &data[0..8] {
            msb = (msb << 8) | i64::from(*byte);
        }
        for byte in &data[8..16] {
            lsb = (lsb << 8) | i64::from(*byte);
        }
        UUID::new(msb, lsb)
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
        golden.check("nameUUIDFromBytes([B)Ljava/util/UUID;", |name: Vec<i8>| {
            UUID::name_uuid_from_bytes(&name)
        });
        golden.check_method("hashCode()I", |this: &mut UUID| this.hash_code());
        golden.check_method("toString()Ljava/lang/String;", |this: &mut UUID| {
            this.to_string()
        });
        golden.finish();
    }
}
