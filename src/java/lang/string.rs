//! java.lang.String, where it differs from Rust's str.

pub trait JavaString {
    /// Runs over UTF-16 code units, so anything outside ASCII hashes differently than over bytes or chars.
    fn hash_code(&self) -> i32;

    /// The difference of the first UTF-16 code units that differ, or else of the lengths in
    /// them: not just the sign, and not in code point order beyond the BMP.
    fn compare_to(&self, another_string: &str) -> i32;
}

impl JavaString for str {
    fn hash_code(&self) -> i32 {
        self.encode_utf16().fold(0, |hash: i32, unit| {
            hash.wrapping_mul(31).wrapping_add(i32::from(unit))
        })
    }

    fn compare_to(&self, another_string: &str) -> i32 {
        let mismatch = self
            .encode_utf16()
            .zip(another_string.encode_utf16())
            .find(|(c1, c2)| c1 != c2);
        match mismatch {
            Some((c1, c2)) => i32::from(c1) - i32::from(c2),
            None => {
                self.encode_utf16().count() as i32 - another_string.encode_utf16().count() as i32
            }
        }
    }
}

/// U+FFFD, what malformed input decodes to.
const REPL: char = '\u{fffd}';

/// `new String(bytes, StandardCharsets.UTF_8)`, which Netty's `ByteBuf.toString` runs. Malformed
/// input becomes U+FFFD the JDK's way (`String.decodeUTF8_UTF16`), which isn't Rust's
/// `from_utf8_lossy`'s: an encoded surrogate (`ED A0 80`) is one replacement, not three, and so is
/// one cut short at the end (`ED A0`). Java's Latin-1 fast path for compact strings decodes the
/// same characters. Every surrogate comes in a pair, so the result fits a Rust string.
pub fn from_bytes_utf8(src: &[i8]) -> String {
    let sl = src.len();
    let mut sp = 0;
    let mut dst = String::with_capacity(sl);
    while sp < sl {
        let mut b1 = i32::from(src[sp]);
        sp += 1;
        if b1 >= 0 {
            dst.push(char::from(b1 as u8));
        } else if (b1 >> 5) == -2 && (b1 & 0x1e) != 0 {
            if sp < sl {
                let b2 = i32::from(src[sp]);
                sp += 1;
                if is_not_continuation(b2) {
                    dst.push(REPL);
                    sp -= 1;
                } else {
                    dst.push(decode2(b1, b2));
                }
                continue;
            }
            dst.push(REPL);
            break;
        } else if (b1 >> 4) == -2 {
            if sp + 1 < sl {
                let b2 = i32::from(src[sp]);
                let b3 = i32::from(src[sp + 1]);
                sp += 2;
                if is_malformed3(b1, b2, b3) {
                    dst.push(REPL);
                    sp -= 3;
                    sp += malformed3(src, sp);
                } else {
                    // A surrogate on its own is malformed too: decode3 gives a UTF-16 unit.
                    dst.push(char::from_u32(decode3(b1, b2, b3)).unwrap_or(REPL));
                }
                continue;
            }
            if sp < sl && is_malformed3_2(b1, i32::from(src[sp])) {
                dst.push(REPL);
                continue;
            }
            dst.push(REPL);
            break;
        } else if (b1 >> 3) == -2 {
            if sp + 2 < sl {
                let b2 = i32::from(src[sp]);
                let b3 = i32::from(src[sp + 1]);
                let b4 = i32::from(src[sp + 2]);
                sp += 3;
                let uc = decode4(b1, b2, b3, b4);
                if is_malformed4(b2, b3, b4) || !is_supplementary_code_point(uc) {
                    // shortest form check
                    dst.push(REPL);
                    sp -= 4;
                    sp += malformed4(src, sp);
                } else {
                    // Java's high and low surrogates.
                    dst.push(char::from_u32(uc as u32).expect("a supplementary code point"));
                }
                continue;
            }
            b1 &= 0xff;
            if b1 > 0xf4 || sp < sl && is_malformed4_2(b1, i32::from(src[sp]) & 0xff) {
                dst.push(REPL);
                continue;
            }
            sp += 1;
            dst.push(REPL);
            if sp < sl && is_malformed4_3(i32::from(src[sp])) {
                continue;
            }
            break;
        } else {
            dst.push(REPL);
        }
    }
    dst
}

fn is_not_continuation(b: i32) -> bool {
    (b & 0xc0) != 0x80
}

fn is_malformed3(b1: i32, b2: i32, b3: i32) -> bool {
    (b1 == -0x20 && (b2 & 0xe0) == 0x80) || (b2 & 0xc0) != 0x80 || (b3 & 0xc0) != 0x80
}

fn is_malformed3_2(b1: i32, b2: i32) -> bool {
    (b1 == -0x20 && (b2 & 0xe0) == 0x80) || (b2 & 0xc0) != 0x80
}

fn is_malformed4(b2: i32, b3: i32, b4: i32) -> bool {
    (b2 & 0xc0) != 0x80 || (b3 & 0xc0) != 0x80 || (b4 & 0xc0) != 0x80
}

/// Takes `b1` and `b2` unsigned.
fn is_malformed4_2(b1: i32, b2: i32) -> bool {
    (b1 == 0xf0 && !(0x90..=0xbf).contains(&b2))
        || (b1 == 0xf4 && (b2 & 0xf0) != 0x80)
        || (b2 & 0xc0) != 0x80
}

fn is_malformed4_3(b3: i32) -> bool {
    (b3 & 0xc0) != 0x80
}

fn is_supplementary_code_point(code_point: i32) -> bool {
    (0x10000..0x110000).contains(&code_point)
}

// The constants are Java's sign-extended bytes: (byte) 0xC0 is -0x40, (byte) 0x80 is -0x80.
fn decode2(b1: i32, b2: i32) -> char {
    let unit = ((b1 << 6) ^ b2) ^ ((-0x40 << 6) ^ -0x80);
    char::from_u32(u32::from(unit as u16)).expect("two bytes decode to U+0080..U+07FF")
}

/// A UTF-16 unit, which may be a surrogate.
fn decode3(b1: i32, b2: i32, b3: i32) -> u32 {
    let unit = (b1 << 12) ^ (b2 << 6) ^ (b3 ^ ((-0x20 << 12) ^ (-0x80 << 6) ^ -0x80));
    u32::from(unit as u16)
}

fn decode4(b1: i32, b2: i32, b3: i32, b4: i32) -> i32 {
    (b1 << 18)
        ^ (b2 << 12)
        ^ (b3 << 6)
        ^ (b4 ^ ((-0x10 << 18) ^ (-0x80 << 12) ^ (-0x80 << 6) ^ -0x80))
}

/// How many bytes a malformed 3-byte sequence's replacement covers: the lead, and its second
/// byte too if that was a continuation the lead allows.
fn malformed3(src: &[i8], sp: usize) -> usize {
    let b1 = i32::from(src[sp]);
    let b2 = i32::from(src[sp + 1]); // no need to lookup b3
    if (b1 == -0x20 && (b2 & 0xe0) == 0x80) || is_not_continuation(b2) {
        1
    } else {
        2
    }
}

fn malformed4(src: &[i8], sp: usize) -> usize {
    // we don't care the speed here
    let b1 = i32::from(src[sp]) & 0xff;
    let b2 = i32::from(src[sp + 1]) & 0xff;
    if b1 > 0xf4
        || (b1 == 0xf0 && !(0x90..=0xbf).contains(&b2))
        || (b1 == 0xf4 && (b2 & 0xf0) != 0x80)
        || is_not_continuation(b2)
    {
        return 1;
    }
    if is_not_continuation(i32::from(src[sp + 2])) {
        return 2;
    }
    3
}

#[cfg(test)]
mod tests {
    use super::{JavaString, from_bytes_utf8};
    use crate::golden::Golden;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("java.lang.String");
        golden.check("hashCode()I", |string: String| string.hash_code());
        golden.check(
            "compareTo(Ljava/lang/String;)I",
            |string: String, another_string: String| string.compare_to(&another_string),
        );
        golden.check(
            "<init>([BLjava/nio/charset/Charset;)V",
            |bytes: Vec<i8>, charset: String| {
                assert_eq!(charset, "UTF-8", "only UTF-8 is ported");
                from_bytes_utf8(&bytes)
            },
        );
        golden.finish();
    }
}
