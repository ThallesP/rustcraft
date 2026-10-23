//! com/google/gson/stream/MalformedJsonException.java (Gson 2.14.0)
//!
//! What a `JsonReader` throws for input that isn't JSON (in its strictness): an `IOException` in
//! Java, a `java::lang::throwable::Throwable` of this class here.

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

use crate::java::lang::throwable::Throwable;

/// The class, whose constructor makes the `Throwable` Java throws.
pub struct MalformedJsonException;

// Java's constructors make an exception of this class, which is a `Throwable` here.
#[allow(clippy::new_ret_no_self)]
impl MalformedJsonException {
    pub const CLASS_NAME: &str = "com.google.gson.stream.MalformedJsonException";

    /// `new MalformedJsonException(msg)`.
    pub fn new(msg: impl Into<String>) -> Throwable {
        Throwable::new(MalformedJsonException::CLASS_NAME, msg)
    }
}
