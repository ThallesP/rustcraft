//! jdk/internal/math/FloatingDecimal.java (OpenJDK 25): `Double.parseDouble` and
//! `Float.parseFloat`, which Gson's numbers parse with (`LazilyParsedNumber`, `JsonReader.nextDouble`,
//! `JsonPrimitive.getAsFloat` of a string). Only `readJavaFormatString` is ported, the scanner
//! deciding what Java accepts (surrounding whitespace, a sign, `NaN`, `Infinity`, hexadecimal with
//! a binary exponent, a trailing `f`, `F`, `d` or `D`) and converting hexadecimal input itself.
//! For decimal input, Java's `ASCIIToBinaryBuffer` rounds the significant digits it copied
//! correctly, which Rust's parser does as well, straight to the format asked for, so the port hands
//! it the same digits and exponent. The binary-to-decimal half is `DoubleToDecimal`'s.
//! `JsonPrimitive`'s golden suite checks it, through `getAsFloat` and `getAsDouble` of strings.

/*
 * Copyright (c) 1996, 2024, Oracle and/or its affiliates. All rights reserved.
 * DO NOT ALTER OR REMOVE COPYRIGHT NOTICES OR THIS FILE HEADER.
 *
 * This code is free software; you can redistribute it and/or modify it
 * under the terms of the GNU General Public License version 2 only, as
 * published by the Free Software Foundation.  Oracle designates this
 * particular file as subject to the "Classpath" exception as provided
 * by Oracle in the LICENSE file that accompanied this code.
 *
 * This code is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
 * FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License
 * version 2 for more details (a copy is included in the LICENSE file that
 * accompanied this code).
 *
 * You should have received a copy of the GNU General Public License version
 * 2 along with this work; if not, write to the Free Software Foundation,
 * Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301 USA.
 *
 * Please contact Oracle, 500 Oracle Parkway, Redwood Shores, CA 94065 USA
 * or visit www.oracle.com if you need additional information or have any
 * questions.
 */

use crate::java::lang::number_format_exception::NumberFormatException;
use crate::jdk::internal::math::math_utils::flog2pow10;

const BINARY_32_IX: usize = 0;
const BINARY_64_IX: usize = 1;

/// The precision of each format.
const P: [i32; 2] = [24, 53];
/// `EP_MIN = max{e : 10^e <= MIN_VALUE/2}`: less or equal values round to zero.
const EP_MIN: [i64; 2] = [-46, -324];
/// `EP_MAX = min{e : MAX_VALUE + ulp(MAX_VALUE)/2 <= 10^(e-1)}`: greater or equal values round to
/// infinity.
const EP_MAX: [i64; 2] = [40, 310];
/// The exponent's width.
const W: [i32; 2] = [(1 << 5) - P[BINARY_32_IX], (1 << 6) - P[BINARY_64_IX]];
/// The maximum exponent in the `m 2^e` representation.
const E_MAX: [i32; 2] = [(1 << (W[0] - 1)) - 1, (1 << (W[1] - 1)) - 1];
/// The minimum exponent in the `m 2^e` representation.
const E_MIN: [i32; 2] = [1 - E_MAX[0], 1 - E_MAX[1]];
/// The minimum exponent in the `c 2^q` representation.
const Q_MIN: [i32; 2] = [E_MIN[0] - (P[0] - 1), E_MIN[1] - (P[1] - 1)];
/// The most significant `P` bits, a rounding bit and a sticky bit need up to `P/4 + 2` hex digits.
const HEX_COUNT: [i32; 2] = [P[0] / 4 + 2, P[1] / 4 + 2];

const MAX_OUT: usize = 1_000;
const OMITTED: &str = " ... ";
const L_HALF: usize = (MAX_OUT - OMITTED.len()) / 2;
const R_HALF: usize = MAX_OUT - (L_HALF + OMITTED.len());

/// What the scanner makes of its input: a value it worked out (zeros, infinities, NaN,
/// hexadecimal input), or a decimal's significant digits and exponent for the conversion.
enum AsciiToBinary {
    Double(f64),
    Float(f32),
    Decimal {
        negative: bool,
        /// The value is `0.d_1 … d_n × 10^decimal_exponent`.
        decimal_exponent: i32,
        digits: Vec<u8>,
    },
}

/// `parseDouble(String)`.
pub fn parse_double(s: &str) -> Result<f64, NumberFormatException> {
    Ok(match read_java_format_string(s, BINARY_64_IX)? {
        AsciiToBinary::Double(value) => value,
        AsciiToBinary::Float(_) => unreachable!("a double was asked for"),
        AsciiToBinary::Decimal {
            negative,
            decimal_exponent,
            digits,
        } => {
            let magnitude: f64 = decimal_text(&digits, decimal_exponent)
                .parse()
                .expect("decimal digits and an exponent");
            if negative { -magnitude } else { magnitude }
        }
    })
}

/// `parseFloat(String)`, rounded straight to a float, not through a double.
pub fn parse_float(s: &str) -> Result<f32, NumberFormatException> {
    Ok(match read_java_format_string(s, BINARY_32_IX)? {
        AsciiToBinary::Float(value) => value,
        AsciiToBinary::Double(_) => unreachable!("a float was asked for"),
        AsciiToBinary::Decimal {
            negative,
            decimal_exponent,
            digits,
        } => {
            let magnitude: f32 = decimal_text(&digits, decimal_exponent)
                .parse()
                .expect("decimal digits and an exponent");
            if negative { -magnitude } else { magnitude }
        }
    })
}

// `0.d_1 … d_n e<exponent>`, which Rust's parser rounds correctly like ASCIIToBinaryBuffer.
fn decimal_text(digits: &[u8], decimal_exponent: i32) -> String {
    format!(
        "0.{}e{decimal_exponent}",
        std::str::from_utf8(digits).expect("ASCII digits")
    )
}

fn zero(ix: usize, negative: bool) -> AsciiToBinary {
    match ix {
        BINARY_32_IX => AsciiToBinary::Float(if negative { -0.0 } else { 0.0 }),
        _ => AsciiToBinary::Double(if negative { -0.0 } else { 0.0 }),
    }
}

fn infinity(ix: usize, negative: bool) -> AsciiToBinary {
    match ix {
        BINARY_32_IX => AsciiToBinary::Float(if negative {
            f32::NEG_INFINITY
        } else {
            f32::INFINITY
        }),
        _ => AsciiToBinary::Double(if negative {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        }),
    }
}

/// The input must match `Double.valueOf`'s rules about leading and trailing whitespace and the
/// grammar. The scanner works on the string's UTF-16 units, like Java's.
fn read_java_format_string(s: &str, ix: usize) -> Result<AsciiToBinary, NumberFormatException> {
    let input: Vec<u16> = s.encode_utf16().collect();
    let len = input.len();
    let char_at = |i: usize| i32::from(input[i]);

    // Skip leading whitespaces.
    let mut i = skip_whitespaces(&input, 0);
    if i == len {
        return Err(NumberFormatException::new("empty String"));
    }

    // Scan the optional significand sign.
    let mut ch = char_at(i);
    let mut ssign = i32::from(b' ');
    if ch == i32::from(b'-') || ch == i32::from(b'+') {
        ssign = ch;
        i += 1;
    }
    let negative = ssign == i32::from(b'-');

    // `(ch | 0b10_0000) == letter` is ch == letter or that letter in upper case. Determine whether
    // this is a symbolic value or hex notation.
    let mut is_dec = true;
    if i < len {
        ch = char_at(i);
        if ch == i32::from(b'I') {
            scan_symbolic(s, &input, i, "Infinity")?;
            return Ok(infinity(ix, negative));
        }
        if ch == i32::from(b'N') {
            scan_symbolic(s, &input, i, "NaN")?;
            // The sign is ignored.
            return Ok(match ix {
                BINARY_32_IX => AsciiToBinary::Float(f32::NAN),
                _ => AsciiToBinary::Double(f64::NAN),
            });
        }
        if ch == i32::from(b'0') && i < len - 1 && (char_at(i + 1) | 0b10_0000) == i32::from(b'x') {
            is_dec = false;
            i += 2;
        }
    }

    let mut pt = 0; // the index after the point, 0 iff absent
    let start = i; // the start of the significand, after the optional sign

    // Skip optional leading zeros, including an optional point.
    while i < len {
        ch = char_at(i);
        if ch != i32::from(b'0') && ch != i32::from(b'.') {
            break;
        }
        i += 1;
        if ch == i32::from(b'.') {
            pt = check_multiple_points(pt, i)?;
        }
    }
    let lz = i; // the index after the leading group of zeros or point

    // Scan all remaining chars of the significand, including an optional point, and find the
    // index after the end of the trailing group of non-zeros.
    let mut tnz = 0; // the index after the trailing group of non-zeros, 0 iff absent
    while i < len {
        ch = char_at(i);
        if !is_digit(ch, is_dec) && ch != i32::from(b'.') {
            break;
        }
        i += 1;
        if ch == i32::from(b'.') {
            pt = check_multiple_points(pt, i)?;
        } else if ch != i32::from(b'0') {
            tnz = i;
        }
    }
    // There must be at least one digit.
    check(s, i - start > usize::from(pt != 0))?;
    let stop = i; // the index after the significand

    // Scan the exponent part, optional for decimal, mandatory for hex.
    let mut ep: i64 = 0;
    let mut has_exp = false;
    let marks_exponent = |ch: i32| {
        let lower = ch | 0b10_0000;
        (lower == i32::from(b'e') && is_dec) || (lower == i32::from(b'p') && !is_dec)
    };
    if i < len && marks_exponent(char_at(i)) {
        i += 1;

        // Scan the optional exponent sign.
        let mut esign = i32::from(b' ');
        if i < len && (char_at(i) == i32::from(b'-') || char_at(i) == i32::from(b'+')) {
            esign = char_at(i);
            i += 1;
        }

        // Scan the exponent digits, accumulated in ep, clamping at 10^10.
        while i < len && is_digit(char_at(i), true) {
            ep = append_dec_digit(ep, char_at(i));
            i += 1;
        }
        // At least 3 chars after the significand, or 2 when one is a digit.
        check(
            s,
            i - stop >= 3 || (i - stop == 2 && esign == i32::from(b' ')),
        )?;
        if esign == i32::from(b'-') {
            ep = -ep;
        }
        has_exp = true;
    }
    // |ep| < 10^10, or |ep| = 10^10 when considered "large", which makes a zero or an infinity.
    check(s, is_dec | has_exp)?;

    // Skip the optional [FfDd] suffix.
    if i < len && matches!(u8::try_from(char_at(i) | 0b10_0000), Ok(b'f' | b'd')) {
        i += 1;
    }

    // Skip optional trailing whitespaces, then it must be the end of the input.
    check(s, skip_whitespaces(&input, i) == len)?;

    // By now, the input is syntactically correct.
    if tnz == 0 {
        // All zero digits, so the exponent and the point don't matter.
        return Ok(zero(ix, negative));
    }

    // Virtually move the point to just after the last non-zero digit, adjusting the exponent,
    // and count the digits without the leading and trailing zeros. In decimal, moving the point
    // by one position means correcting ep by 1; in hexadecimal by 4.
    let emult: i64 = if is_dec { 1 } else { 4 };
    let mut n = (tnz - lz) as i64; // the number of significant digits, first approximation
    if pt == 0 {
        ep += emult * (stop - tnz) as i64;
    } else {
        ep += emult * (pt as i64 - tnz as i64);
        if pt > tnz {
            // The point was counted as a position.
            ep -= emult;
        } else if lz < pt {
            // lz < pt <= tnz
            n -= 1;
        }
    }
    // The magnitude is f 10^ep (decimal) or f 2^ep (hexadecimal), f the n digits in [lz, tnz).

    if !is_dec {
        // Rounding the leftmost P bits, a rounding bit and a sticky bit is the same as rounding
        // all bits: in hex digits, HEX_COUNT of them.
        let mut j = 0;
        i = lz;
        let mut c: i64 = 0;
        let le = n.min(i64::from(HEX_COUNT[ix]));
        while j < le {
            ch = char_at(i);
            i += 1;
            if ch != i32::from(b'.') {
                j += 1;
                c = (c << 4) | i64::from(digit_for(ch));
            }
        }
        if n > le {
            // Force a sticky bit.
            c |= 0b1;
            ep += 4 * (n - le);
        }

        let bl = (64 - c.leading_zeros()) as i64; // the bit length
        // x = c 2^ep, so 2^(ep+bl-1) <= x < 2^(ep+bl).
        if ep < i64::from(Q_MIN[ix]) - bl {
            return Ok(zero(ix, negative));
        }
        if ep > i64::from(E_MAX[ix]) - bl + 1 {
            return Ok(infinity(ix, negative));
        }
        let mut q = ep as i32; // the narrowing is safe
        let bl = bl as i32;
        let shr; // shift right iff shr > 0
        if q > E_MIN[ix] - bl {
            shr = bl - P[ix];
            q += shr;
        } else {
            shr = Q_MIN[ix] - q;
            q = Q_MIN[ix];
        }
        if shr > 0 {
            let thr: i64 = 1 << shr;
            let tail = (c & (thr - 1)) << 1;
            c = ((c as u64) >> shr) as i64;
            if tail > thr || (tail == thr && (c & 0b1) != 0) {
                c += 1;
                if c >= 1 << P[ix] {
                    // But in fact it can't be greater.
                    c = ((c as u64) >> 1) as i64;
                    q += 1;
                }
            }
        } else {
            c <<= -shr;
        }

        return Ok(match ix {
            BINARY_32_IX => AsciiToBinary::Float(build_float(negative, q, c)),
            _ => AsciiToBinary::Double(build_double(negative, q, c)),
        });
    }

    // For decimal input, an appropriate prefix of the digits is copied, and the conversion does
    // the math. With e' = n + ep, x = <0 . f_1 … f_n> 10^e', 10^(e'-1) <= x < 10^e'. Past
    // EP_MIN it rounds to zero, past EP_MAX to infinity.
    let e = (ep + n).clamp(EP_MIN[ix], EP_MAX[ix]);
    if e == EP_MIN[ix] {
        return Ok(zero(ix, negative));
    }
    if e == EP_MAX[ix] {
        return Ok(infinity(ix, negative));
    }
    let e = e as i32;

    // Enough digits to round correctly: the integer part, 1 - q digits of the fraction for the
    // q the conservative estimate ql allows, then a non-zero sticky digit if any are left.
    let ql = (flog2pow10(e - 1) - (P[ix] - 1)).max(Q_MIN[ix]);
    let np = i64::from(e) + i64::from((2 - ql).max(1));
    let mut digits;
    if n >= np {
        digits = copy_digits(&input, (np - 1) as usize, lz);
        // Any non-zero sticky digit.
        digits.push(b'3');
    } else {
        digits = copy_digits(&input, n as usize, lz);
    }
    Ok(AsciiToBinary::Decimal {
        negative,
        decimal_exponent: e,
        digits,
    })
}

fn build_double(negative: bool, q: i32, c: i64) -> f64 {
    let be: i64 = if c < 1 << (P[BINARY_64_IX] - 1) {
        0
    } else {
        i64::from(q) + ((1023 - 1) + i64::from(P[BINARY_64_IX]))
    };
    let bits = (if negative { 1u64 << 63 } else { 0 })
        | ((be as u64) << (P[BINARY_64_IX] - 1))
        | (c as u64 & ((1 << 52) - 1));
    f64::from_bits(bits)
}

fn build_float(negative: bool, q: i32, c: i64) -> f32 {
    let be: i32 = if c < 1 << (P[BINARY_32_IX] - 1) {
        0
    } else {
        q + ((127 - 1) + P[BINARY_32_IX])
    };
    let bits = (if negative { 1u32 << 31 } else { 0 })
        | ((be as u32) << (P[BINARY_32_IX] - 1))
        | (c as u32 & ((1 << 23) - 1));
    f32::from_bits(bits)
}

fn copy_digits(input: &[u16], len: usize, mut i: usize) -> Vec<u8> {
    let mut digits = Vec::with_capacity(len + 1);
    while digits.len() < len {
        let ch = input[i];
        i += 1;
        if ch != u16::from(b'.') {
            digits.push(ch as u8);
        }
    }
    digits
}

/// Appends the decimal digit `ch` to `v >= 0`, clamping at 10^10.
fn append_dec_digit(v: i64, ch: i32) -> i64 {
    if v < 10_000_000_000 / 10 {
        10 * v + i64::from(ch - i32::from(b'0'))
    } else {
        10_000_000_000
    }
}

/// Whether `ch` is a digit, `0-9`, or for hexadecimal also `A-F` or `a-f`.
fn is_digit(ch: i32, is_dec: bool) -> bool {
    (i32::from(b'0')..=i32::from(b'9')).contains(&ch)
        || (!is_dec && (i32::from(b'a')..=i32::from(b'f')).contains(&(ch | 0b10_0000)))
}

/// The value of `ch`, a hex digit.
fn digit_for(ch: i32) -> i32 {
    if ch <= i32::from(b'9') {
        ch - i32::from(b'0')
    } else {
        (ch | 0b10_0000) - (i32::from(b'a') - 10)
    }
}

/// From `i`, skips all chars in `['\0', ' ']`, and returns the index after them.
fn skip_whitespaces(input: &[u16], mut i: usize) -> usize {
    while i < input.len() && input[i] <= u16::from(b' ') {
        i += 1;
    }
    i
}

/// Scans `sub` at `i`, then optional trailing whitespaces, which must end the input.
fn scan_symbolic(s: &str, input: &[u16], i: usize, sub: &str) -> Result<(), NumberFormatException> {
    let sub: Vec<u16> = sub.encode_utf16().collect();
    let high = i + sub.len();
    check(
        s,
        high <= input.len()
            && input[i..high] == sub[..]
            && skip_whitespaces(input, high) == input.len(),
    )
}

/// `i` if this is the first point the scanner finds.
fn check_multiple_points(pt: usize, i: usize) -> Result<usize, NumberFormatException> {
    if pt != 0 {
        return Err(NumberFormatException::new("multiple points"));
    }
    Ok(i)
}

/// Java's message holds the input, its middle cut out past 1000 units.
fn check(s: &str, expected: bool) -> Result<(), NumberFormatException> {
    if expected {
        return Ok(());
    }
    let input: Vec<u16> = s.encode_utf16().collect();
    let shown = if input.len() > MAX_OUT {
        format!(
            "{}{OMITTED}{}",
            String::from_utf16_lossy(&input[..L_HALF]),
            String::from_utf16_lossy(&input[input.len() - R_HALF..])
        )
    } else {
        s.to_owned()
    };
    Err(NumberFormatException::new(format!(
        "For input string: \"{shown}\""
    )))
}
