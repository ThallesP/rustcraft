//! jdk/internal/math/ToDecimal.java (OpenJDK 25): what `DoubleToDecimal` and `FloatToDecimal`
//! share, writing the digits into a buffer. Only its LATIN1 coder is ported, which is the one
//! `Double.toString` and `Float.toString` use: a byte per character.

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

/// Used for left-to-right digit extraction.
pub(crate) const MASK_28: i32 = (1 << 28) - 1;

pub(crate) const NON_SPECIAL: i32 = 0 << 8;
pub(crate) const PLUS_ZERO: i32 = 1 << 8;
pub(crate) const MINUS_ZERO: i32 = 2 << 8;
pub(crate) const PLUS_INF: i32 = 3 << 8;
pub(crate) const MINUS_INF: i32 = 4 << 8;
pub(crate) const NAN: i32 = 5 << 8;

/// `Math.multiplyHigh(long, long)`: the high 64 bits of the 128-bit product.
pub(crate) fn multiply_high(x: i64, y: i64) -> i64 {
    ((i128::from(x) * i128::from(y)) >> 64) as i64
}

pub(crate) fn put_char(str: &mut [u8], index: i32, c: u8) -> i32 {
    str[index as usize] = c;
    index + 1
}

pub(crate) fn put_digit(str: &mut [u8], index: i32, d: i32) -> i32 {
    put_char(str, index, (i32::from(b'0') + d) as u8)
}

/// Left-to-right digit extraction: algorithm 1 in Bouvier & Zimmermann, with b = 10, k = 8,
/// n = 28.
pub(crate) fn put8_digits(str: &mut [u8], index: i32, m: i32) -> i32 {
    let mut y = y(m);
    for i in 0..8 {
        let t = y.wrapping_mul(10);
        str[(index + i) as usize] = (i32::from(b'0') + ((t as u32) >> 28) as i32) as u8;
        y = t & MASK_28;
    }
    index + 8
}

/// floor((a + 1) 2^28 / 10^8) - 1, for a < 10^8.
pub(crate) fn y(a: i32) -> i32 {
    ((multiply_high(i64::from(a + 1) << 28, 193_428_131_138_340_668) as u64 >> 20) as i32) - 1
}

/// Drops the zeroes at the end, but not the one right after the point.
pub(crate) fn remove_trailing_zeroes(str: &[u8], mut index: i32) -> i32 {
    while str[(index - 1) as usize] == b'0' {
        index -= 1;
    }
    if str[(index - 1) as usize] == b'.' {
        index += 1;
    }
    index
}

pub(crate) fn special(r#type: i32) -> &'static str {
    match r#type {
        PLUS_ZERO => "0.0",
        MINUS_ZERO => "-0.0",
        PLUS_INF => "Infinity",
        MINUS_INF => "-Infinity",
        _ => "NaN",
    }
}

/// `new String(str, 0, size, ISO_8859_1)`, or the special value's name: what `toString` returns.
pub(crate) fn to_string(str: &[u8], pair: i32) -> String {
    let r#type = pair & 0xff00;
    if r#type == NON_SPECIAL {
        let size = (pair & 0xff) as usize;
        return str[..size].iter().map(|&byte| char::from(byte)).collect();
    }
    special(r#type).to_owned()
}
