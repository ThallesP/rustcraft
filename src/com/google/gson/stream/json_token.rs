//! com/google/gson/stream/JsonToken.java (Gson 2.14.0)
//!
//! A structure, name or value type in a JSON-encoded string, which `JsonReader.peek` returns.

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

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JsonToken {
    BEGIN_ARRAY,
    END_ARRAY,
    BEGIN_OBJECT,
    END_OBJECT,
    NAME,
    STRING,
    NUMBER,
    BOOLEAN,
    NULL,
    END_DOCUMENT,
}

impl JsonToken {
    pub const VALUES: [JsonToken; 10] = [
        JsonToken::BEGIN_ARRAY,
        JsonToken::END_ARRAY,
        JsonToken::BEGIN_OBJECT,
        JsonToken::END_OBJECT,
        JsonToken::NAME,
        JsonToken::STRING,
        JsonToken::NUMBER,
        JsonToken::BOOLEAN,
        JsonToken::NULL,
        JsonToken::END_DOCUMENT,
    ];

    pub fn name(self) -> &'static str {
        match self {
            JsonToken::BEGIN_ARRAY => "BEGIN_ARRAY",
            JsonToken::END_ARRAY => "END_ARRAY",
            JsonToken::BEGIN_OBJECT => "BEGIN_OBJECT",
            JsonToken::END_OBJECT => "END_OBJECT",
            JsonToken::NAME => "NAME",
            JsonToken::STRING => "STRING",
            JsonToken::NUMBER => "NUMBER",
            JsonToken::BOOLEAN => "BOOLEAN",
            JsonToken::NULL => "NULL",
            JsonToken::END_DOCUMENT => "END_DOCUMENT",
        }
    }
}

/// `toString()`: the constant's name.
impl fmt::Display for JsonToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}
