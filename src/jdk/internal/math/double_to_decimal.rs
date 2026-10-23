//! jdk/internal/math/DoubleToDecimal.java (OpenJDK 25): `Double.toString`, Raffaello Giulietti's
//! Schubfach, which renders a double as the shortest decimal that rounds back to it (one of two
//! closest to it when two are as short), in Java's format. Not ported: `split` and `putDecimal`,
//! which `Formatter` and `StringBuilder` use.
//!
//! For full details see Giulietti, "The Schubfach way to render doubles"; IEEE 754; and Bouvier &
//! Zimmermann, "Division-Free Binary-to-Decimal Conversion". Divisions are avoided altogether.

/*
 * Copyright (c) 2021, 2024, Oracle and/or its affiliates. All rights reserved.
 * Copyright (c) 2024, Alibaba Group Holding Limited. All Rights Reserved.
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

use crate::jdk::internal::math::math_utils::{
    flog2pow10, flog10pow2, flog10three_quarters_pow2, g0, g1, pow10,
};
use crate::jdk::internal::math::to_decimal::{
    self, MASK_28, MINUS_INF, MINUS_ZERO, NAN, PLUS_INF, PLUS_ZERO, multiply_high, put_char,
    put_digit, put8_digits, remove_trailing_zeroes, y,
};

/// The precision in bits.
const P: i32 = 53;
/// Exponent width in bits.
const W: i32 = (64 - 1) - (P - 1);
/// Minimum value of the exponent: -(2^(W-1)) - P + 3.
const Q_MIN: i32 = (-1 << (W - 1)) - P + 3;
/// Threshold to detect tiny values, as in section 8.2.1.
const C_TINY: i64 = 3;
/// H is as in section 8.1.
const H: i32 = 17;
/// Minimum value of the significand of a normal value: 2^(P-1).
const C_MIN: i64 = 1 << (P - 1);
/// Mask to extract the biased exponent.
const BQ_MASK: i32 = (1 << W) - 1;
/// Mask to extract the fraction bits.
const T_MASK: i64 = (1 << (P - 1)) - 1;
/// Used in `rop`.
const MASK_63: i64 = i64::MAX;

/// Room for the longer of the forms -ddddd.dddddddddddd, -0.00ddddddddddddddddd and
/// -d.ddddddddddddddddE-eee, where there are H digits d.
pub const MAX_CHARS: usize = (H + 7) as usize;

/// `toString(double)`: `Double.toString`.
pub fn to_string(v: f64) -> String {
    let mut str = [0; MAX_CHARS];
    let pair = to_decimal(&mut str, 0, v);
    to_decimal::to_string(&str, pair)
}

/// The size in the lower byte and the type in the higher one: `PLUS_ZERO` iff v is 0.0,
/// `MINUS_ZERO` iff -0.0, `PLUS_INF` and `MINUS_INF` for the infinities, `NAN`, or else
/// `NON_SPECIAL`.
fn to_decimal(str: &mut [u8], mut index: i32, v: f64) -> i32 {
    // For finite v != 0, determine integers c and q such that |v| = c 2^q and Q_MIN <= q <= Q_MAX
    // and either 2^(P-1) <= c < 2^P (normal) or 0 < c < 2^(P-1) and q = Q_MIN (subnormal).
    let bits = v.to_bits() as i64;
    let t = bits & T_MASK;
    let bq = ((bits as u64) >> (P - 1)) as i32 & BQ_MASK;
    if bq < BQ_MASK {
        let start = index;
        if bits < 0 {
            index = put_char(str, index, b'-');
        }
        if bq != 0 {
            // A normal value. Here mq = -q.
            let mq = -Q_MIN + 1 - bq;
            let c = C_MIN | t;
            // The fast path discussed in section 8.3.
            if (0 < mq) & (mq < P) {
                let f = c >> mq;
                if f << mq == c {
                    return to_chars(str, index, f, 0) - start;
                }
            }
            return to_decimal_of(str, index, -mq, c, 0) - start;
        }
        if t != 0 {
            // A subnormal value.
            return if t < C_TINY {
                to_decimal_of(str, index, Q_MIN, 10 * t, -1)
            } else {
                to_decimal_of(str, index, Q_MIN, t, 0)
            } - start;
        }
        return if bits == 0 { PLUS_ZERO } else { MINUS_ZERO };
    }
    if t != 0 {
        return NAN;
    }
    if bits > 0 { PLUS_INF } else { MINUS_INF }
}

/// `toDecimal(byte[], int, int, long, int, FormattedFPDecimal)`: the skeleton of figure 7, with
/// figure 9's computations. cb is c-bar, cbr and cbl c-bar-r and -l, vb, vbr and vbl v-bar, -r
/// and -l, and rop r-o-prime.
fn to_decimal_of(str: &mut [u8], index: i32, q: i32, c: i64, dk: i32) -> i32 {
    let out = c as i32 & 0x1;
    let cb = c << 2;
    let cbr = cb + 2;
    let (cbl, k) = if (c != C_MIN) | (q == Q_MIN) {
        // Regular spacing.
        (cb - 2, flog10pow2(q))
    } else {
        // Irregular spacing.
        (cb - 1, flog10three_quarters_pow2(q))
    };
    let h = q + flog2pow10(-k) + 2;

    // g1 and g0 are as in section 9.8.3, so g = g1 2^63 + g0.
    let g1 = g1(k);
    let g0 = g0(k);

    let vb = rop(g1, g0, cb << h);
    let vbl = rop(g1, g0, cbl << h);
    let vbr = rop(g1, g0, cbr << h);

    let s = vb >> 2;
    if s >= 100 {
        // For n = 17, m = 1 the table in section 10 shows
        //     s' = floor(s / 10) = floor(s 115_292_150_460_684_698 2^4 / 2^64).
        // sp10 = 10 s' and tp10 = 10 t'; upin iff u' = sp10 10^k is in Rv, and wpin iff
        // w' = tp10 10^k is (section 9.3).
        let sp10 = 10 * multiply_high(s, 115_292_150_460_684_698 << 4);
        let tp10 = sp10 + 10;
        let upin = vbl + i64::from(out) <= sp10 << 2;
        let wpin = (tp10 << 2) + i64::from(out) <= vbr;
        if upin != wpin {
            return to_chars(str, index, if upin { sp10 } else { tp10 }, k);
        }
    }

    // 10 <= s < 100, or s >= 100 and u', w' not in Rv: uin iff u = s 10^k is in Rv, and win
    // iff w = t 10^k is (section 9.3).
    let t = s + 1;
    let uin = vbl + i64::from(out) <= s << 2;
    let win = (t << 2) + i64::from(out) <= vbr;
    if uin != win {
        // Exactly one of u or w lies in Rv.
        return to_chars(str, index, if uin { s } else { t }, k + dk);
    }
    // Both u and w lie in Rv: the one closest to v (section 9.3).
    let cmp = vb - ((s + t) << 1);
    let f = if cmp < 0 || cmp == 0 && (s & 0x1) == 0 {
        s
    } else {
        t
    };
    to_chars(str, index, f, k + dk)
}

/// rop(cp g 2^(-127)), where g = g1 2^63 + g0 (section 9.9 and figure 8).
fn rop(g1: i64, g0: i64, cp: i64) -> i64 {
    let x1 = multiply_high(g0, cp);
    let y0 = g1.wrapping_mul(cp);
    let y1 = multiply_high(g1, cp);
    let z = ((y0 as u64 >> 1) as i64).wrapping_add(x1);
    let vbp = y1.wrapping_add((z as u64 >> 63) as i64);
    vbp | ((z & MASK_63).wrapping_add(MASK_63) as u64 >> 63) as i64
}

/// Formats the decimal f 10^e (section 10).
fn to_chars(str: &mut [u8], index: i32, mut f: i64, mut e: i32) -> i32 {
    // len such that 10^(len-1) <= f < 10^len.
    let mut len = flog10pow2(64 - f.leading_zeros() as i32);
    if f >= pow10(len) {
        len += 1;
    }

    // Transform f and e so that 10^(H-1) <= f < 10^H and fp 10^ep = f 10^(e-H) = 0.f 10^e, fp
    // and ep being the original f and e.
    f *= pow10(H - len);
    e += len;

    // The to_chars? functions extract digits left to right in ints, of at most 8 digits, so the
    // H = 17 digits of f are split into h, the most significant, m, the next 8, and l, the last
    // 8. For n = 17, m = 8 the table in section 10 shows
    //     floor(f / 10^8) = floor(floor(193_428_131_138_340_668 f / 2^64) / 2^20)
    // and for n = 9, m = 8
    //     floor(hm / 10^8) = floor(1_441_151_881 hm / 2^57).
    let hm = (multiply_high(f, 193_428_131_138_340_668) as u64 >> 20) as i64;
    let l = (f - 100_000_000 * hm) as i32;
    let h = ((hm * 1_441_151_881) as u64 >> 57) as i32;
    let m = (hm - 100_000_000 * i64::from(h)) as i32;

    if 0 < e && e <= 7 {
        return to_chars1(str, index, h, m, l, e);
    }
    if -3 < e && e <= 0 {
        return to_chars2(str, index, h, m, l, e);
    }
    to_chars3(str, index, h, m, l, e)
}

/// 0 < e <= 7: plain format without leading zeroes.
fn to_chars1(str: &mut [u8], mut index: i32, h: i32, m: i32, l: i32, e: i32) -> i32 {
    index = put_digit(str, index, h);
    let mut y = y(m);
    let mut i = 1;
    while i < e {
        let t = y.wrapping_mul(10);
        index = put_digit(str, index, (t as u32 >> 28) as i32);
        y = t & MASK_28;
        i += 1;
    }
    index = put_char(str, index, b'.');
    while i <= 8 {
        let t = y.wrapping_mul(10);
        index = put_digit(str, index, (t as u32 >> 28) as i32);
        y = t & MASK_28;
        i += 1;
    }
    low_digits(str, index, l)
}

/// -3 < e <= 0: plain format with leading zeroes.
fn to_chars2(str: &mut [u8], mut index: i32, h: i32, m: i32, l: i32, mut e: i32) -> i32 {
    index = put_digit(str, index, 0);
    index = put_char(str, index, b'.');
    while e < 0 {
        index = put_digit(str, index, 0);
        e += 1;
    }
    index = put_digit(str, index, h);
    index = put8_digits(str, index, m);
    low_digits(str, index, l)
}

/// -3 >= e | e > 7: computerized scientific notation.
fn to_chars3(str: &mut [u8], mut index: i32, h: i32, m: i32, l: i32, e: i32) -> i32 {
    index = put_digit(str, index, h);
    index = put_char(str, index, b'.');
    index = put8_digits(str, index, m);
    index = low_digits(str, index, l);
    exponent(str, index, e - 1)
}

fn low_digits(str: &mut [u8], mut index: i32, l: i32) -> i32 {
    if l != 0 {
        index = put8_digits(str, index, l);
    }
    remove_trailing_zeroes(str, index)
}

fn exponent(str: &mut [u8], mut index: i32, mut e: i32) -> i32 {
    index = put_char(str, index, b'E');
    if e < 0 {
        index = put_char(str, index, b'-');
        e = -e;
    }
    if e < 10 {
        return put_digit(str, index, e);
    }
    if e >= 100 {
        // For n = 3, m = 2 the table in section 10 shows floor(e / 100) = floor(1_311 e / 2^17).
        let d = (e * 1_311) as u32 >> 17;
        index = put_digit(str, index, d as i32);
        e -= 100 * d as i32;
    }
    // For n = 2, m = 1 the table in section 10 shows floor(e / 10) = floor(103 e / 2^10).
    let d = ((e * 103) as u32 >> 10) as i32;
    index = put_digit(str, index, d);
    put_digit(str, index, e - 10 * d)
}
