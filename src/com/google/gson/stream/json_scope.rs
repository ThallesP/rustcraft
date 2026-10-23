//! com/google/gson/stream/JsonScope.java (Gson 2.14.0)
//!
//! What the reader's and writer's nesting stacks hold: the kind of each open scope, Java's ints.

/*
 * Copyright (C) 2010 Google Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * Translated to Rust from Gson 2.14.0, and changed: see above.
 */

/// An array with no elements requires no separator before the next element.
pub(super) const EMPTY_ARRAY: i32 = 1;

/// An array with at least one value requires a separator before the next element.
pub(super) const NONEMPTY_ARRAY: i32 = 2;

/// An object with no name/value pairs requires no separator before the next element.
pub(super) const EMPTY_OBJECT: i32 = 3;

/// An object whose most recent element is a key. The next element must be a value.
pub(super) const DANGLING_NAME: i32 = 4;

/// An object with at least one name/value pair requires a separator before the next element.
pub(super) const NONEMPTY_OBJECT: i32 = 5;

/// No top-level value has been started yet.
pub(super) const EMPTY_DOCUMENT: i32 = 6;

/// A top-level value has already been started.
pub(super) const NONEMPTY_DOCUMENT: i32 = 7;

/// A document that's been closed and cannot be accessed.
pub(super) const CLOSED: i32 = 8;
