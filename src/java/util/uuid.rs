//! java.util.UUID.

use std::fmt;

use md5::{Digest, Md5};

use crate::java::lang::character::Character;
use crate::java::lang::number_format_exception::NumberFormatException;

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
    /// `fromString`: the fast form (36 characters, dashes in place, hex digits), else the lenient
    /// one (five dash-separated hex fields, each `Long.parseLong`'d and masked). The `Err` is the
    /// `IllegalArgumentException`'s message (a `NumberFormatException`'s too).
    pub fn from_string(name: &str) -> Result<UUID, String> {
        let chars: Vec<u16> = name.encode_utf16().collect();
        if chars.len() == 36 && [8, 13, 18, 23].iter().all(|&i| chars[i] == u16::from(b'-')) {
            let nibbles = |pos: usize| -> i64 {
                let mut value = 0i64;
                for &ch in &chars[pos..pos + 4] {
                    match char::from_u32(u32::from(ch)).and_then(|c| c.to_digit(16)) {
                        Some(digit) if ch <= 0xff => value = value << 4 | i64::from(digit),
                        _ => return -1,
                    }
                }
                value
            };
            let parts = [0, 4, 9, 14, 19, 24, 28, 32].map(nibbles);
            if parts.iter().all(|&part| part >= 0) {
                return Ok(UUID::new(
                    parts[0] << 48 | parts[1] << 32 | parts[2] << 16 | parts[3],
                    parts[4] << 48 | parts[5] << 32 | parts[6] << 16 | parts[7],
                ));
            }
        }
        UUID::from_string1(name, &chars)
    }

    fn from_string1(name: &str, chars: &[u16]) -> Result<UUID, String> {
        let len = chars.len();
        if len > 36 {
            return Err("UUID string too large".to_owned());
        }
        let dash = |from: usize| {
            chars[from.min(len)..]
                .iter()
                .position(|&c| c == u16::from(b'-'))
                .map(|i| i + from)
        };
        let dash1 = dash(0);
        let dash2 = dash1.and_then(|d| dash(d + 1));
        let dash3 = dash2.and_then(|d| dash(d + 1));
        let dash4 = dash3.and_then(|d| dash(d + 1));
        let dash5 = dash4.and_then(|d| dash(d + 1));
        let (Some(dash1), Some(dash2), Some(dash3), Some(dash4), None) =
            (dash1, dash2, dash3, dash4, dash5)
        else {
            return Err(format!("Invalid UUID string: {name}"));
        };
        let parse = |begin: usize, end: usize| {
            parse_long_range(chars, begin, end)
                .map_err(|error| error.get_message().unwrap_or("null").to_owned())
        };
        let mut most_sig_bits = parse(0, dash1)? & 0xffffffff;
        most_sig_bits <<= 16;
        most_sig_bits |= parse(dash1 + 1, dash2)? & 0xffff;
        most_sig_bits <<= 16;
        most_sig_bits |= parse(dash2 + 1, dash3)? & 0xffff;
        let mut least_sig_bits = parse(dash3 + 1, dash4)? & 0xffff;
        least_sig_bits <<= 48;
        least_sig_bits |= parse(dash4 + 1, len)? & 0xffffffffffff;
        Ok(UUID::new(most_sig_bits, least_sig_bits))
    }

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

// `Long.parseLong(CharSequence, int, int, 16)`, JDK 25's: a sign, then hex digits, accumulated
// negatively; the error names the range and the index in it.
fn parse_long_range(
    s: &[u16],
    begin_index: usize,
    end_index: usize,
) -> Result<i64, NumberFormatException> {
    let radix: i64 = 16;
    if begin_index == end_index {
        return Err(NumberFormatException::for_input_string_radix("", 16));
    }
    let mut digit: i32 = !0xFF;
    let mut i = begin_index;
    let first_char = s[i];
    i += 1;
    if first_char != u16::from(b'-') && first_char != u16::from(b'+') {
        digit = Character::digit(first_char, 16);
    }
    if digit >= 0 || (digit == !0xFF && end_index - begin_index > 1) {
        let limit = if first_char != u16::from(b'-') {
            i64::MIN + 1
        } else {
            i64::MIN
        };
        let multmin = limit / radix;
        let mut result = -i64::from(digit & 0xFF);
        let mut in_range = true;
        while i < end_index {
            digit = Character::digit(s[i], 16);
            i += 1;
            if digit < 0 {
                break;
            }
            in_range = result > multmin
                || (result == multmin && i64::from(digit) <= radix * multmin - limit);
            if !in_range {
                break;
            }
            result = radix * result - i64::from(digit);
        }
        if in_range && i == end_index && digit >= 0 {
            return Ok(if first_char != u16::from(b'-') {
                -result
            } else {
                result
            });
        }
    }
    let error_index = i - usize::from(digit >= -1);
    Err(NumberFormatException::for_char_sequence(
        s,
        begin_index,
        end_index,
        error_index,
    ))
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
