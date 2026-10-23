//! com/google/common/math/DoubleMath.java
//!
//! Only `fuzzyEquals`, which the shapes compare coordinates with. Checked through the shapes'
//! goldens.

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

pub struct DoubleMath;

impl DoubleMath {
    /// Whether two doubles are within the tolerance of each other, or equal (infinities), or both
    /// NaN. Throws `IllegalArgumentException` for a negative tolerance, a panic here.
    pub fn fuzzy_equals(a: f64, b: f64, tolerance: f64) -> bool {
        assert!(tolerance >= 0.0, "tolerance ({tolerance}) must be >= 0");
        (a - b).copysign(1.0) <= tolerance || a == b || (a.is_nan() && b.is_nan())
    }
}
