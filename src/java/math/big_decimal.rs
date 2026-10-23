//! java/math/BigDecimal.java (OpenJDK 25), as far as Gson reaches it: `new BigDecimal(String)`
//! (through `NumberLimits.parseBigDecimal`, for `LazilyParsedNumber`'s fallbacks and
//! `JsonPrimitive.getAsBigDecimal`), the conversions `intValue`, `longValue`, `longValueExact`,
//! `floatValue` and `doubleValue`, `compareTo` (`JsonPrimitive.equals`), `scale`, and `toString`.
//!
//! Java keeps the unscaled value in a long when it fits and in a `BigInteger` otherwise; the port
//! keeps its decimal digits, which is the same value either way. The conversions follow from the
//! value: `longValue` is the integer part's low 64 bits, and `floatValue` and `doubleValue` are
//! correctly rounded, as Java's are, which Rust's parser does from the digits. The constructor
//! keeps Java's two paths (up to 18 characters and longer), whose error messages differ. There is
//! no arithmetic and no `MathContext`: Gson only parses and converts. `JsonPrimitive`'s golden
//! suite checks it, through `getAsBigDecimal` and `LazilyParsedNumber`'s fallbacks.

/*
 * Copyright (c) 1996, 2025, Oracle and/or its affiliates. All rights reserved.
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

/*
 * Portions Copyright IBM Corporation, 2001. All Rights Reserved.
 */

use std::cmp::Ordering;
use std::fmt;

use crate::java::lang::arithmetic_exception::ArithmeticException;
use crate::java::lang::character::Character;
use crate::java::lang::number_format_exception::NumberFormatException;

/// The longest significand (with its point and exponent) the constructor reads into a long.
const MAX_COMPACT_DIGITS: usize = 18;

/// `unscaledValue × 10^-scale`. Equal values of different scales (`1.0` and `1.00`) are different
/// decimals, as in Java's `equals`; `compare_to` compares the values.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BigDecimal {
    /// The unscaled value's sign: -1, 0 or 1. Zero has no sign.
    signum: i32,
    /// The unscaled value's decimal digits, most significant first, without leading zeros, so
    /// none for zero.
    digits: Vec<u8>,
    scale: i32,
}

impl BigDecimal {
    /// `new BigDecimal(String)`: an optional sign, decimal digits (any Unicode decimal digit, as
    /// `Character.digit` reads them) with an optional point, then an optional exponent after `e`
    /// or `E`. A string it reads past the end of (`"1e"`, `""`) throws a `NumberFormatException`
    /// without a message, Java's `ArrayIndexOutOfBoundsException` caught and rethrown.
    pub fn new(val: &str) -> Result<BigDecimal, NumberFormatException> {
        let input: Vec<u16> = val.encode_utf16().collect();
        BigDecimal::from_chars(&input).ok_or_else(NumberFormatException::without_message)?
    }

    // The constructor's body, `None` where Java indexes past the array.
    fn from_chars(input: &[u16]) -> Option<Result<BigDecimal, NumberFormatException>> {
        let mut offset = 0usize;
        let mut len = input.len();
        let mut prec = 0usize; // the precision
        let mut scl: i64 = 0; // the scale
        let mut coeff: Vec<u8> = Vec::new(); // the significand's digits

        // Handle the sign.
        let mut isneg = false;
        if *input.get(offset)? == u16::from(b'-') {
            isneg = true;
            offset += 1;
            len -= 1;
        } else if input[offset] == u16::from(b'+') {
            offset += 1;
            len -= 1;
        }

        // Now at the significand's numeric part.
        let mut dot = false;
        let is_compact = len <= MAX_COMPACT_DIGITS;
        if is_compact {
            // Java computes the value in a long as it goes; `coeff` holds its digits.
            while len > 0 {
                let c = input[offset];
                let digit = if (u16::from(b'0')..=u16::from(b'9')).contains(&c) {
                    Some((c - u16::from(b'0')) as u8)
                } else if c != u16::from(b'.') && Character::digit(c, 10) >= 0 {
                    Some(Character::digit(c, 10) as u8)
                } else {
                    None
                };
                if let Some(digit) = digit {
                    if digit == 0 {
                        if prec == 0 {
                            prec = 1;
                        } else if !coeff.is_empty() {
                            coeff.push(0);
                            prec += 1;
                        } // else the digit is a redundant leading zero
                    } else {
                        if prec != 1 || !coeff.is_empty() {
                            prec += 1; // unchanged if preceded by zeros
                        }
                        coeff.push(digit);
                    }
                    if dot {
                        scl += 1;
                    }
                } else if c == u16::from(b'.') {
                    if dot {
                        return Some(Err(NumberFormatException::new(
                            "Character array contains more than one decimal point.",
                        )));
                    }
                    dot = true;
                } else if c == u16::from(b'e') || c == u16::from(b'E') {
                    match parse_exp(input, offset, len)? {
                        Ok(exp) => scl -= exp,
                        Err(error) => return Some(Err(error)),
                    }
                    break;
                } else {
                    return Some(Err(NumberFormatException::new(format!(
                        "Character {} is neither a decimal digit number, decimal point, nor \"e\" notation exponential mark.",
                        String::from_utf16_lossy(&[c])
                    ))));
                }
                offset += 1;
                len -= 1;
            }
            if prec == 0 {
                return Some(Err(NumberFormatException::new("No digits found.")));
            }
        } else {
            while len > 0 {
                let c = input[offset];
                if (u16::from(b'0')..=u16::from(b'9')).contains(&c) || Character::digit(c, 10) >= 0
                {
                    let digit = Character::digit(c, 10) as u8;
                    if digit == 0 {
                        if prec == 0 {
                            prec = 1;
                        } else if !coeff.is_empty() {
                            coeff.push(0);
                            prec += 1;
                        } // else c must be a redundant leading zero
                    } else {
                        if prec != 1 || !coeff.is_empty() {
                            prec += 1; // unchanged if preceded by zeros
                        }
                        coeff.push(digit);
                    }
                    if dot {
                        scl += 1;
                    }
                    offset += 1;
                    len -= 1;
                    continue;
                }
                if c == u16::from(b'.') {
                    if dot {
                        return Some(Err(NumberFormatException::new(
                            "Character array contains more than one decimal point.",
                        )));
                    }
                    dot = true;
                    offset += 1;
                    len -= 1;
                    continue;
                }
                // An exponent is expected.
                if c != u16::from(b'e') && c != u16::from(b'E') {
                    return Some(Err(NumberFormatException::new(
                        "Character array is missing \"e\" notation exponential mark.",
                    )));
                }
                match parse_exp(input, offset, len)? {
                    Ok(exp) => scl -= exp,
                    Err(error) => return Some(Err(error)),
                }
                break;
            }
            if prec == 0 {
                return Some(Err(NumberFormatException::new("No digits found.")));
            }
        }
        let Ok(scale) = i32::try_from(scl) else {
            return Some(Err(NumberFormatException::new("Exponent overflow.")));
        };
        let signum = match (coeff.is_empty(), isneg) {
            (true, _) => 0,
            (false, true) => -1,
            (false, false) => 1,
        };
        Some(Ok(BigDecimal {
            signum,
            digits: coeff,
            scale,
        }))
    }

    pub fn scale(&self) -> i32 {
        self.scale
    }

    /// The number of digits in the unscaled value, 1 for zero.
    pub fn precision(&self) -> i32 {
        self.digits.len().max(1) as i32
    }

    pub fn signum(&self) -> i32 {
        self.signum
    }

    // Whether a non-zero value's absolute value is less than one: it only has fraction digits.
    fn fraction_only(&self) -> bool {
        self.precision() <= self.scale
    }

    /// `intValue()`: the integer part's low 32 bits.
    pub fn int_value(&self) -> i32 {
        self.long_value() as i32
    }

    /// `longValue()`: the integer part, truncated toward zero, its low 64 bits like
    /// `toBigInteger().longValue()`. A scale of -64 or less leaves at least 64 factors of 2.
    pub fn long_value(&self) -> i64 {
        if self.signum == 0 || self.fraction_only() || self.scale <= -64 {
            return 0;
        }
        let integer_digits = (self.digits.len() as i64 - i64::from(self.scale)) as usize;
        let mut magnitude: u64 = 0;
        for index in 0..integer_digits {
            let digit = self.digits.get(index).copied().unwrap_or(0);
            magnitude = magnitude.wrapping_mul(10).wrapping_add(u64::from(digit));
        }
        let value = magnitude as i64;
        if self.signum < 0 {
            value.wrapping_neg()
        } else {
            value
        }
    }

    /// `longValueExact()`: the value, when it's a whole number that fits in a long.
    pub fn long_value_exact(&self) -> Result<i64, ArithmeticException> {
        if self.signum == 0 {
            return Ok(0);
        }
        // Numbers less than 1.0.
        if self.fraction_only() {
            return Err(ArithmeticException::new("Rounding necessary"));
        }
        // More than 19 digits in the integer part can't fit.
        if self.precision() - 19 > self.scale {
            return Err(ArithmeticException::new("Overflow"));
        }
        // Rounding to an integer throws if the fraction isn't zero.
        if self.scale > 0 {
            let fraction = &self.digits[self.digits.len() - self.scale as usize..];
            if fraction.iter().any(|&digit| digit != 0) {
                return Err(ArithmeticException::new("Rounding necessary"));
            }
        }
        let integer_digits = (self.digits.len() as i64 - i64::from(self.scale)) as usize;
        let mut value: i128 = 0;
        for index in 0..integer_digits {
            value = value * 10 + i128::from(self.digits.get(index).copied().unwrap_or(0));
        }
        if self.signum < 0 {
            value = -value;
        }
        i64::try_from(value).map_err(|_| ArithmeticException::new("Overflow"))
    }

    /// `floatValue()`: the value correctly rounded to a float. Zero is positive.
    pub fn float_value(&self) -> f32 {
        if self.signum == 0 {
            return 0.0;
        }
        let magnitude: f32 = self.decimal_text().parse().expect("a decimal");
        if self.signum < 0 {
            -magnitude
        } else {
            magnitude
        }
    }

    /// `doubleValue()`: the value correctly rounded to a double. Zero is positive.
    pub fn double_value(&self) -> f64 {
        if self.signum == 0 {
            return 0.0;
        }
        let magnitude: f64 = self.decimal_text().parse().expect("a decimal");
        if self.signum < 0 {
            -magnitude
        } else {
            magnitude
        }
    }

    // The absolute value as `<digits>e<-scale>`, which Rust's parser rounds correctly.
    fn decimal_text(&self) -> String {
        let digits: String = self
            .digits
            .iter()
            .map(|&digit| char::from(b'0' + digit))
            .collect();
        format!("{digits}e{}", -i64::from(self.scale))
    }

    /// `compareTo`: the values, whatever their scales (`1.0` and `1.00` are equal).
    pub fn compare_to(&self, val: &BigDecimal) -> Ordering {
        if self.signum != val.signum {
            return self.signum.cmp(&val.signum);
        }
        if self.signum == 0 {
            return Ordering::Equal;
        }
        // The position of the leading digit, then the digits from it.
        let adjusted =
            |decimal: &BigDecimal| decimal.digits.len() as i64 - i64::from(decimal.scale);
        let magnitude = adjusted(self).cmp(&adjusted(val)).then_with(|| {
            let length = self.digits.len().max(val.digits.len());
            (0..length)
                .map(|index| {
                    let digit =
                        |decimal: &BigDecimal| decimal.digits.get(index).copied().unwrap_or(0);
                    digit(self).cmp(&digit(val))
                })
                .find(|ordering| ordering.is_ne())
                .unwrap_or(Ordering::Equal)
        });
        if self.signum < 0 {
            magnitude.reverse()
        } else {
            magnitude
        }
    }
}

/// `parseExp`: the exponent after the `e` at `offset`, `len` characters from it to the end.
/// `None` where Java indexes past the array.
fn parse_exp(
    input: &[u16],
    mut offset: usize,
    mut len: usize,
) -> Option<Result<i64, NumberFormatException>> {
    let mut exp: i64 = 0;
    offset += 1;
    let mut c = *input.get(offset)?;
    len -= 1;
    let negexp = c == u16::from(b'-');
    // The optional sign.
    if negexp || c == u16::from(b'+') {
        offset += 1;
        c = *input.get(offset)?;
        len -= 1;
    }
    if len == 0 {
        return Some(Err(NumberFormatException::new("No exponent digits.")));
    }
    // Skip leading zeros in the exponent.
    while len > 10 && (c == u16::from(b'0') || Character::digit(c, 10) == 0) {
        offset += 1;
        c = *input.get(offset)?;
        len -= 1;
    }
    if len > 10 {
        return Some(Err(NumberFormatException::new(
            "Too many nonzero exponent digits.",
        )));
    }
    // c holds the exponent's first digit.
    loop {
        let v = if (u16::from(b'0')..=u16::from(b'9')).contains(&c) {
            i64::from(c - u16::from(b'0'))
        } else {
            let v = Character::digit(c, 10);
            if v < 0 {
                return Some(Err(NumberFormatException::new("Not a digit.")));
            }
            i64::from(v)
        };
        exp = exp * 10 + v;
        if len == 1 {
            break;
        }
        offset += 1;
        c = *input.get(offset)?;
        len -= 1;
    }
    if negexp {
        exp = -exp;
    }
    Some(Ok(exp))
}

/// `toString()`: plain when the scale isn't negative and the leading digit's exponent is at least
/// -6 (`0.000001`, `1.50`), scientific otherwise (`1E-7`, `1.5E+300`).
impl fmt::Display for BigDecimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let coeff: Vec<char> = if self.digits.is_empty() {
            vec!['0']
        } else {
            self.digits
                .iter()
                .map(|&digit| char::from(b'0' + digit))
                .collect()
        };
        if self.scale == 0 {
            if self.signum < 0 {
                f.write_str("-")?;
            }
            return f.write_str(&coeff.iter().collect::<String>());
        }
        let mut buf = String::new();
        if self.signum < 0 {
            buf.push('-');
        }
        let coeff_len = coeff.len() as i64;
        let adjusted = -i64::from(self.scale) + (coeff_len - 1);
        if self.scale >= 0 && adjusted >= -6 {
            // Plain number.
            let pad = i64::from(self.scale) - coeff_len; // the count of padding zeros
            if pad >= 0 {
                // The 0.xxx form.
                buf.push_str("0.");
                buf.extend(std::iter::repeat_n('0', pad as usize));
                buf.extend(&coeff);
            } else {
                // The xx.xx form.
                let point = (-pad) as usize;
                buf.extend(&coeff[..point]);
                buf.push('.');
                buf.extend(&coeff[point..]);
            }
        } else {
            // Scientific notation.
            buf.push(coeff[0]);
            if coeff.len() > 1 {
                buf.push('.');
                buf.extend(&coeff[1..]);
            }
            if adjusted != 0 {
                buf.push('E');
                if adjusted > 0 {
                    buf.push('+');
                }
                buf.push_str(&adjusted.to_string());
            }
        }
        f.write_str(&buf)
    }
}
