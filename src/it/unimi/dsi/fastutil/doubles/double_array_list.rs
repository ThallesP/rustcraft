//! it/unimi/dsi/fastutil/doubles/DoubleArrayList.java
//!
//! A list over an array, as `DoubleArrayList.wrap(array)` and `wrap(array, length)` make it: its
//! size is the length, and it shares the array, which `equals` compares by reference first. Only
//! what the shapes call is ported, and the lists they make never change, so the array is an
//! `Arc<[f64]>`. `DoubleLists.unmodifiable` of one reads the same, so it is one too. Reading past
//! the size throws `IndexOutOfBoundsException`, a panic here.

/*
 * Copyright (C) 2002-2024 Sebastiano Vigna
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * Translated to Rust and changed for rustcraft.
 */

use std::any::Any;
use std::sync::Arc;

use crate::it::unimi::dsi::fastutil::doubles::double_list::DoubleList;

/// A list of the first `size` doubles of an array it shares.
#[derive(Clone, Debug)]
pub struct DoubleArrayList {
    a: Arc<[f64]>,
    size: i32,
}

impl DoubleArrayList {
    /// `wrap(double[])`.
    pub fn wrap(a: Arc<[f64]>) -> DoubleArrayList {
        let size = a.len() as i32;
        DoubleArrayList { a, size }
    }

    /// `wrap(double[], int)`, which throws when the length is past the array's.
    pub fn wrap_length(a: Arc<[f64]>, length: i32) -> DoubleArrayList {
        assert!(
            length as usize <= a.len(),
            "The specified length ({length}) is greater than the array size ({})",
            a.len()
        );
        DoubleArrayList { a, size: length }
    }

    /// `new DoubleArrayList(double[])`, a copy of the array.
    pub fn from_slice(a: &[f64]) -> DoubleArrayList {
        DoubleArrayList::wrap(Arc::from(a))
    }
}

impl DoubleList for DoubleArrayList {
    fn get_double(&self, index: i32) -> f64 {
        assert!(
            index >= 0 && index < self.size,
            "Index ({index}) is greater than or equal to list size ({})",
            self.size
        );
        self.a[index as usize]
    }

    fn size(&self) -> i32 {
        self.size
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn array(&self) -> Option<&Arc<[f64]>> {
        Some(&self.a)
    }
}
