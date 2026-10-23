//! com/google/gson/JsonSyntaxException.java (Gson 2.14.0)
//!
//! What Gson throws for malformed JSON, which `ByteBufCodecs.lenientJson` catches: a
//! `java::lang::throwable::Throwable` of this class, made like Java's constructors. It wraps the
//! reader's `MalformedJsonException`, `EOFException` or `NumberFormatException`, whose `toString()`
//! is its message.

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

/// The class, whose constructors make the `Throwable` Java throws.
pub struct JsonSyntaxException;

// Java's constructors make an exception of this class, which is a `Throwable` here.
#[allow(clippy::new_ret_no_self)]
impl JsonSyntaxException {
    pub const CLASS_NAME: &str = "com.google.gson.JsonSyntaxException";

    /// `new JsonSyntaxException(msg)`.
    pub fn new(msg: impl Into<String>) -> Throwable {
        Throwable::new(JsonSyntaxException::CLASS_NAME, msg)
    }

    /// `new JsonSyntaxException(cause)`: the cause's `toString()` is the message.
    pub fn from_cause(cause: Throwable) -> Throwable {
        Throwable::with_cause(JsonSyntaxException::CLASS_NAME, cause)
    }
}
