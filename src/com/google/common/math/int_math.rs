//! com/google/common/math/IntMath.java
//!
//! Only `gcd`, which the shapes merge cubes' coordinates with (`Shapes.lcm`,
//! `DiscreteCubeMerger`). Checked through `Shapes.lcm`'s goldens.

/*
 * Copyright (C) 2011 The Guava Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
 * in compliance with the License. You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software distributed under the License
 * is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express
 * or implied. See the License for the specific language governing permissions and limitations under
 * the License.
 *
 * Translated to Rust and changed for rustcraft.
 */

pub struct IntMath;

impl IntMath {
    /// The greatest common divisor of two non-negative ints, by the binary algorithm, and 0 for
    /// two zeros. Throws `IllegalArgumentException` for a negative one, a panic here.
    pub fn gcd(a: i32, b: i32) -> i32 {
        assert!(a >= 0, "a ({a}) must be >= 0");
        assert!(b >= 0, "b ({b}) must be >= 0");
        if a == 0 {
            return b;
        } else if b == 0 {
            return a;
        }
        let a_twos = a.trailing_zeros();
        let mut a = a >> a_twos;
        let b_twos = b.trailing_zeros();
        let mut b = b >> b_twos;
        while a != b {
            let delta = a - b;
            let min_delta_or_zero = delta & (delta >> (i32::BITS - 1));
            a = delta - min_delta_or_zero - min_delta_or_zero;
            b += min_delta_or_zero;
            a >>= a.trailing_zeros();
        }
        a << a_twos.min(b_twos)
    }
}
