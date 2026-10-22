//! java/lang/FdLibm.java (OpenJDK 25), the Java port of fdlibm 5.3 that `StrictMath` runs, and
//! so `Math` too where HotSpot has no stub of its own. Only the functions the game calls.
//!
//! The libm crate ports musl, which rewrote parts of fdlibm: its `atan2` returns ±pi/2 for
//! |y/x| > 2^64 where fdlibm adds half of pi's low part, one ulp more for negative x.

/*
 * Copyright (c) 1998, 2024, Oracle and/or its affiliates. All rights reserved.
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
// fdlibm makes its NaNs as (x - x) / (x - x).
#![allow(clippy::eq_op)]

use std::f64::consts::PI;

const HUGE: f64 = 1.0e+300;

const EXP_BITS: i32 = 0x7ff0_0000;
const EXP_SIGNIF_BITS: i32 = 0x7fff_ffff;

// __LO(x): the low-order 32 bits.
fn lo(x: f64) -> i32 {
    x.to_bits() as i32
}

// __LO(x, low): x with its low-order 32 bits replaced.
fn with_lo(x: f64, low: i32) -> f64 {
    f64::from_bits((x.to_bits() & 0xFFFF_FFFF_0000_0000) | u64::from(low as u32))
}

// __HI(x): the high-order 32 bits.
fn hi(x: f64) -> i32 {
    (x.to_bits() >> 32) as i32
}

// __HI(x, high): x with its high-order 32 bits replaced.
fn with_hi(x: f64, high: i32) -> f64 {
    f64::from_bits((x.to_bits() & 0x0000_0000_FFFF_FFFF) | (u64::from(high as u32) << 32))
}

// Math.powerOfTwoD(n): 2^n for a normal exponent n.
fn power_of_two_d(n: i32) -> f64 {
    f64::from_bits(((i64::from(n) + 1023) << 52) as u64 & 0x7FF0_0000_0000_0000)
}

// Hex float literals, as bits since Rust has none.
const PIO2_HI: f64 = f64::from_bits(0x3ff921fb54442d18); // 0x1.921fb54442d18p0
const PIO2_LO: f64 = f64::from_bits(0x3c91a62633145c07); // 0x1.1a62633145c07p-54
const PIO4_HI: f64 = f64::from_bits(0x3fe921fb54442d18); // 0x1.921fb54442d18p-1
const P_S0: f64 = f64::from_bits(0x3fc5555555555555); // 0x1.5555555555555p-3
const P_S1: f64 = f64::from_bits(0xbfd4d61203eb6f7d); // -0x1.4d61203eb6f7dp-2
const P_S2: f64 = f64::from_bits(0x3fc9c1550e884455); // 0x1.9c1550e884455p-3
const P_S3: f64 = f64::from_bits(0xbfa48228b5688f3b); // -0x1.48228b5688f3bp-5
const P_S4: f64 = f64::from_bits(0x3f49efe07501b288); // 0x1.9efe07501b288p-11
const P_S5: f64 = f64::from_bits(0x3f023de10dfdf709); // 0x1.23de10dfdf709p-15
const Q_S1: f64 = f64::from_bits(0xc0033a271c8a2d4b); // -0x1.33a271c8a2d4bp1
const Q_S2: f64 = f64::from_bits(0x40002ae59c598ac8); // 0x1.02ae59c598ac8p1
const Q_S3: f64 = f64::from_bits(0xbfe6066c1b8d0159); // -0x1.6066c1b8d0159p-1
const Q_S4: f64 = f64::from_bits(0x3fb3b8c5b12e9282); // 0x1.3b8c5b12e9282p-4

pub struct Asin;

impl Asin {
    pub fn compute(x: f64) -> f64 {
        let mut t = 0.0;
        let hx = hi(x);
        let ix = hx & EXP_SIGNIF_BITS;
        if ix >= 0x3ff0_0000 {
            // |x| >= 1
            if ((ix - 0x3ff0_0000) | lo(x)) == 0 {
                // asin(1) = +-pi/2 with inexact
                return x * PIO2_HI + x * PIO2_LO;
            }
            return (x - x) / (x - x); // asin(|x| > 1) is NaN
        } else if ix < 0x3fe0_0000 {
            // |x| < 0.5
            if ix < 0x3e40_0000 {
                // if |x| < 2**-27
                if HUGE + x > 1.0 {
                    // return x with inexact if x != 0
                    return x;
                }
            } else {
                t = x * x;
            }
            let p = t * (P_S0 + t * (P_S1 + t * (P_S2 + t * (P_S3 + t * (P_S4 + t * P_S5)))));
            let q = 1.0 + t * (Q_S1 + t * (Q_S2 + t * (Q_S3 + t * Q_S4)));
            let w = p / q;
            return x + x * w;
        }
        // 1 > |x| >= 0.5
        let mut w = 1.0 - x.abs();
        t = w * 0.5;
        let mut p = t * (P_S0 + t * (P_S1 + t * (P_S2 + t * (P_S3 + t * (P_S4 + t * P_S5)))));
        let mut q = 1.0 + t * (Q_S1 + t * (Q_S2 + t * (Q_S3 + t * Q_S4)));
        let s = t.sqrt();
        if ix >= 0x3FEF_3333 {
            // if |x| > 0.975
            w = p / q;
            t = PIO2_HI - (2.0 * (s + s * w) - PIO2_LO);
        } else {
            w = s;
            w = with_lo(w, 0);
            let c = (t - w * w) / (s + w);
            let r = p / q;
            p = 2.0 * s * r - (PIO2_LO - 2.0 * c);
            q = PIO4_HI - 2.0 * w;
            t = PIO4_HI - (p - q);
        }
        if hx > 0 { t } else { -t }
    }
}

pub struct Acos;

impl Acos {
    pub fn compute(x: f64) -> f64 {
        let hx = hi(x);
        let ix = hx & EXP_SIGNIF_BITS;
        if ix >= 0x3ff0_0000 {
            // |x| >= 1
            if ((ix - 0x3ff0_0000) | lo(x)) == 0 {
                // |x| == 1
                if hx > 0 {
                    // acos(1) = 0
                    return 0.0;
                } else {
                    // acos(-1)= pi
                    return PI + 2.0 * PIO2_LO;
                }
            }
            return (x - x) / (x - x); // acos(|x| > 1) is NaN
        }
        if ix < 0x3fe0_0000 {
            // |x| < 0.5
            if ix <= 0x3c60_0000 {
                // if |x| < 2**-57
                return PIO2_HI + PIO2_LO;
            }
            let z = x * x;
            let p = z * (P_S0 + z * (P_S1 + z * (P_S2 + z * (P_S3 + z * (P_S4 + z * P_S5)))));
            let q = 1.0 + z * (Q_S1 + z * (Q_S2 + z * (Q_S3 + z * Q_S4)));
            let r = p / q;
            PIO2_HI - (x - (PIO2_LO - x * r))
        } else if hx < 0 {
            // x < -0.5
            let z = (1.0 + x) * 0.5;
            let p = z * (P_S0 + z * (P_S1 + z * (P_S2 + z * (P_S3 + z * (P_S4 + z * P_S5)))));
            let q = 1.0 + z * (Q_S1 + z * (Q_S2 + z * (Q_S3 + z * Q_S4)));
            let s = z.sqrt();
            let r = p / q;
            let w = r * s - PIO2_LO;
            PI - 2.0 * (s + w)
        } else {
            // x > 0.5
            let z = (1.0 - x) * 0.5;
            let s = z.sqrt();
            let mut df = s;
            df = with_lo(df, 0);
            let c = (z - df * df) / (s + df);
            let p = z * (P_S0 + z * (P_S1 + z * (P_S2 + z * (P_S3 + z * (P_S4 + z * P_S5)))));
            let q = 1.0 + z * (Q_S1 + z * (Q_S2 + z * (Q_S3 + z * Q_S4)));
            let r = p / q;
            let w = r * s + c;
            2.0 * (df + w)
        }
    }
}

pub struct Atan;

const ATANHI: [f64; 4] = [
    f64::from_bits(0x3fddac670561bb4f), // 0x1.dac670561bb4fp-2, atan(0.5)hi
    f64::from_bits(0x3fe921fb54442d18), // 0x1.921fb54442d18p-1, atan(1.0)hi
    f64::from_bits(0x3fef730bd281f69b), // 0x1.f730bd281f69bp-1, atan(1.5)hi
    f64::from_bits(0x3ff921fb54442d18), // 0x1.921fb54442d18p0, atan(inf)hi
];

const ATANLO: [f64; 4] = [
    f64::from_bits(0x3c7a2b7f222f65e2), // 0x1.a2b7f222f65e2p-56, atan(0.5)lo
    f64::from_bits(0x3c81a62633145c07), // 0x1.1a62633145c07p-55, atan(1.0)lo
    f64::from_bits(0x3c7007887af0cbbd), // 0x1.007887af0cbbdp-56, atan(1.5)lo
    f64::from_bits(0x3c91a62633145c07), // 0x1.1a62633145c07p-54, atan(inf)lo
];

const A_T: [f64; 11] = [
    f64::from_bits(0x3fd555555555550d), // 0x1.555555555550dp-2
    f64::from_bits(0xbfc999999998ebc4), // -0x1.999999998ebc4p-3
    f64::from_bits(0x3fc24924920083ff), // 0x1.24924920083ffp-3
    f64::from_bits(0xbfbc71c6fe231671), // -0x1.c71c6fe231671p-4
    f64::from_bits(0x3fb745cdc54c206e), // 0x1.745cdc54c206ep-4
    f64::from_bits(0xbfb3b0f2af749a6d), // -0x1.3b0f2af749a6dp-4
    f64::from_bits(0x3fb10d66a0d03d51), // 0x1.10d66a0d03d51p-4
    f64::from_bits(0xbfadde2d52defd9a), // -0x1.dde2d52defd9ap-5
    f64::from_bits(0x3fa97b4b24760deb), // 0x1.97b4b24760debp-5
    f64::from_bits(0xbfa2b4442c6a6c2f), // -0x1.2b4442c6a6c2fp-5
    f64::from_bits(0x3f90ad3ae322da11), // 0x1.0ad3ae322da11p-6
];

impl Atan {
    pub fn compute(mut x: f64) -> f64 {
        let id: i32;
        let hx = hi(x);
        let ix = hx & EXP_SIGNIF_BITS;
        if ix >= 0x4410_0000 {
            // if |x| >= 2^66
            if ix > EXP_BITS || (ix == EXP_BITS && (lo(x) != 0)) {
                return x + x; // NaN
            }
            if hx > 0 {
                return ATANHI[3] + ATANLO[3];
            } else {
                return -ATANHI[3] - ATANLO[3];
            }
        }
        if ix < 0x3fdc_0000 {
            // |x| < 0.4375
            if ix < 0x3e20_0000 {
                // |x| < 2^-29
                if HUGE + x > 1.0 {
                    // raise inexact
                    return x;
                }
            }
            id = -1;
        } else {
            x = x.abs();
            if ix < 0x3ff3_0000 {
                // |x| < 1.1875
                if ix < 0x3fe60000 {
                    // 7/16 <= |x| < 11/16
                    id = 0;
                    x = (2.0 * x - 1.0) / (2.0 + x);
                } else {
                    // 11/16 <= |x| < 19/16
                    id = 1;
                    x = (x - 1.0) / (x + 1.0);
                }
            } else if ix < 0x4003_8000 {
                // |x| < 2.4375
                id = 2;
                x = (x - 1.5) / (1.0 + 1.5 * x);
            } else {
                // 2.4375 <= |x| < 2^66
                id = 3;
                x = -1.0 / x;
            }
        }
        // end of argument reduction
        let mut z = x * x;
        let w = z * z;
        // break sum from i=0 to 10 aT[i]z**(i+1) into odd and even poly
        let s1 =
            z * (A_T[0] + w * (A_T[2] + w * (A_T[4] + w * (A_T[6] + w * (A_T[8] + w * A_T[10])))));
        let s2 = w * (A_T[1] + w * (A_T[3] + w * (A_T[5] + w * (A_T[7] + w * A_T[9]))));
        if id < 0 {
            x - x * (s1 + s2)
        } else {
            let id = id as usize;
            z = ATANHI[id] - ((x * (s1 + s2) - ATANLO[id]) - x);
            if hx < 0 { -z } else { z }
        }
    }
}

pub struct Atan2;

const TINY: f64 = 1.0e-300;
const PI_O_4: f64 = f64::from_bits(0x3fe921fb54442d18); // 0x1.921fb54442d18p-1
const PI_O_2: f64 = f64::from_bits(0x3ff921fb54442d18); // 0x1.921fb54442d18p0
const PI_LO: f64 = f64::from_bits(0x3ca1a62633145c07); // 0x1.1a62633145c07p-53

impl Atan2 {
    pub fn compute(y: f64, x: f64) -> f64 {
        let hx = hi(x);
        let ix = hx & EXP_SIGNIF_BITS;
        let lx = lo(x);
        let hy = hi(y);
        let iy = hy & EXP_SIGNIF_BITS;
        let ly = lo(y);
        if x.is_nan() || y.is_nan() {
            return x + y;
        }
        if (hx.wrapping_sub(0x3ff0_0000) | lx) == 0 {
            // x = 1.0
            return Atan::compute(y);
        }
        let m = ((hy >> 31) & 1) | ((hx >> 30) & 2); // 2*sign(x) + sign(y)

        // when y = 0
        if (iy | ly) == 0 {
            match m {
                0 | 1 => return y,      // atan(+/-0, +anything)  = +/-0
                2 => return PI + TINY,  // atan(+0,   -anything)  =  pi
                _ => return -PI - TINY, // atan(-0,   -anything)  = -pi
            }
        }
        // when x = 0
        if (ix | lx) == 0 {
            return if hy < 0 {
                -PI_O_2 - TINY
            } else {
                PI_O_2 + TINY
            };
        }

        // when x is INF
        if ix == EXP_BITS {
            if iy == EXP_BITS {
                return match m {
                    0 => PI_O_4 + TINY,        // atan(+INF, +INF)
                    1 => -PI_O_4 - TINY,       // atan(-INF, +INF)
                    2 => 3.0 * PI_O_4 + TINY,  // atan(+INF, -INF)
                    _ => -3.0 * PI_O_4 - TINY, // atan(-INF, -INF)
                };
            } else {
                return match m {
                    0 => 0.0,        // atan(+..., +INF)
                    1 => -0.0,       // atan(-..., +INF)
                    2 => PI + TINY,  // atan(+..., -INF)
                    _ => -PI - TINY, // atan(-..., -INF)
                };
            }
        }
        // when y is INF
        if iy == EXP_BITS {
            return if hy < 0 {
                -PI_O_2 - TINY
            } else {
                PI_O_2 + TINY
            };
        }

        // compute y/x
        let k = (iy - ix) >> 20;
        let z = if k > 60 {
            // |y/x| >  2**60
            PI_O_2 + 0.5 * PI_LO
        } else if hx < 0 && k < -60 {
            // |y|/x < -2**60
            0.0
        } else {
            // safe to do y/x
            Atan::compute((y / x).abs())
        };
        match m {
            0 => z,                // atan(+, +)
            1 => -z,               // atan(-, +)
            2 => PI - (z - PI_LO), // atan(+, -)
            _ => (z - PI_LO) - PI, // atan(-, -), case 3
        }
    }
}

pub struct Hypot;

const TWO_MINUS_600: f64 = f64::from_bits(0x1a70000000000000); // 0x1.0p-600
const TWO_PLUS_600: f64 = f64::from_bits(0x6570000000000000); // 0x1.0p+600
const ABOUT_TWO_PLUS_500: f64 = f64::from_bits(0x5f300000ffffffff); // 0x1.00000_ffff_ffffp500
const TWO_MINUS_500: f64 = f64::from_bits(0x20b0000000000000); // 0x1.0p-500
const TWO_PLUS_1022: f64 = f64::from_bits(0x7fd0000000000000); // 0x1.0p1022

impl Hypot {
    /// sqrt(x*x + y*y) to within an ulp, without overflowing or underflowing in between: the
    /// high word of each operand stands in for its magnitude, and splits it in two halves whose
    /// products are exact.
    pub fn compute(x: f64, y: f64) -> f64 {
        let mut a = x.abs();
        let mut b = y.abs();

        if !a.is_finite() || !b.is_finite() {
            if a == f64::INFINITY || b == f64::INFINITY {
                return f64::INFINITY;
            } else {
                return a + b; // Propagate NaN significand bits
            }
        }

        if b > a {
            std::mem::swap(&mut a, &mut b);
        }

        let mut ha = hi(a); // high word of a
        let mut hb = hi(b); // high word of b

        if ha.wrapping_sub(hb) > 0x3c00000 {
            return a + b; // x / y > 2**60
        }

        let mut k = 0;
        if a > ABOUT_TWO_PLUS_500 {
            // a > ~2**500: scale a and b by 2**-600
            ha = ha.wrapping_sub(0x25800000);
            hb = hb.wrapping_sub(0x25800000);
            a *= TWO_MINUS_600;
            b *= TWO_MINUS_600;
            k += 600;
        }
        let mut t1: f64;
        let t2: f64;
        if b < TWO_MINUS_500 {
            // b < 2**-500
            if b < f64::MIN_POSITIVE {
                // subnormal b or 0
                if b == 0.0 {
                    return a;
                }
                t1 = TWO_PLUS_1022; // t1 = 2^1022
                b *= t1;
                a *= t1;
                k -= 1022;
            } else {
                // scale a and b by 2^600
                ha = ha.wrapping_add(0x25800000); // a *= 2^600
                hb = hb.wrapping_add(0x25800000); // b *= 2^600
                a *= TWO_PLUS_600;
                b *= TWO_PLUS_600;
                k -= 600;
            }
        }
        // medium size a and b
        let mut w = a - b;
        if w > b {
            t1 = 0.0;
            t1 = with_hi(t1, ha);
            t2 = a - t1;
            w = (t1 * t1 - (b * (-b) - t2 * (a + t1))).sqrt();
        } else {
            a += a;
            let mut y1 = 0.0;
            y1 = with_hi(y1, hb);
            let y2 = b - y1;
            t1 = 0.0;
            t1 = with_hi(t1, ha.wrapping_add(0x00100000));
            t2 = a - t1;
            w = (t1 * y1 - (w * (-w) - (t1 * y2 + t2 * b))).sqrt();
        }
        if k != 0 { power_of_two_d(k) * w } else { w }
    }
}
