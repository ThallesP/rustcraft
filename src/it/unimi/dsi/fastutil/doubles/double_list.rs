//! it/unimi/dsi/fastutil/doubles/DoubleList.java, with AbstractDoubleList.java's `equals`.
//!
//! A voxel shape's coordinates along an axis are a `DoubleList`, of the kinds the shapes make
//! (`DoubleArrayList`, and the shapes' own `CubePointRange`, `OffsetDoubleList` and
//! `NonOverlappingMerger`), and the shapes tell a `CubePointRange` apart with `instanceof`, which
//! is `as_any`. Only what the shapes call is ported: `getDouble`, `size` and `equals`. fastutil
//! publishes no sources with the game, so this follows its bytecode (`fastutil-8.5.18.jar`).

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

/// A list of doubles, shared like Java's references.
pub trait DoubleList: Any + Send + Sync {
    fn get_double(&self, index: i32) -> f64;

    fn size(&self) -> i32;

    /// For `instanceof`.
    fn as_any(&self) -> &dyn Any;

    /// The array a `DoubleArrayList` wraps, which its `equals` compares by reference first.
    fn array(&self) -> Option<&Arc<[f64]>> {
        None
    }
}

/// `first.equals(second)`: the same size, and each element `==` the other's, so -0.0 equals 0.0
/// and NaN equals nothing. Java first returns true for the same list, and `DoubleArrayList` for
/// two lists over the same array, which only differs from comparing the elements when they hold
/// a NaN.
pub fn equals(first: &Arc<dyn DoubleList>, second: &Arc<dyn DoubleList>) -> bool {
    if std::ptr::addr_eq(Arc::as_ptr(first), Arc::as_ptr(second)) {
        return true;
    }
    let size = first.size();
    if size != second.size() {
        return false;
    }
    if let (Some(a1), Some(a2)) = (first.array(), second.array())
        && Arc::ptr_eq(a1, a2)
    {
        return true;
    }
    (0..size).all(|index| first.get_double(index) == second.get_double(index))
}
