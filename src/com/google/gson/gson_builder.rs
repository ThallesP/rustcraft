//! com/google/gson/GsonBuilder.java (Gson 2.14.0)
//!
//! Only the settings `Gson.toJson(JsonElement)` reads: `serializeNulls`, `disableHtmlEscaping`,
//! `generateNonExecutableJson` and `setStrictness` (`ByteBufCodecs.lenientJson` makes its Gson
//! with `new GsonBuilder().disableHtmlEscaping().create()`). Type adapters, naming policies,
//! exclusions and formatting aren't ported.

/*
 * Copyright (C) 2008 Google Inc.
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

use crate::com::google::gson::gson::Gson;
use crate::com::google::gson::strictness::Strictness;

const DEFAULT_JSON_NON_EXECUTABLE: bool = false;
const DEFAULT_ESCAPE_HTML: bool = true;
const DEFAULT_SERIALIZE_NULLS: bool = false;

#[derive(Clone, Debug)]
pub struct GsonBuilder {
    serialize_nulls: bool,
    escape_html_chars: bool,
    generate_non_executable_json: bool,
    strictness: Option<Strictness>,
}

impl GsonBuilder {
    /// `new GsonBuilder()`: Gson's defaults.
    pub fn new() -> GsonBuilder {
        GsonBuilder {
            serialize_nulls: DEFAULT_SERIALIZE_NULLS,
            escape_html_chars: DEFAULT_ESCAPE_HTML,
            generate_non_executable_json: DEFAULT_JSON_NON_EXECUTABLE,
            strictness: None,
        }
    }

    /// Writes null members of objects, which are dropped by default.
    pub fn serialize_nulls(&mut self) -> &mut GsonBuilder {
        self.serialize_nulls = true;
        self
    }

    /// Prefixes the JSON with `)]}'` and a newline.
    pub fn generate_non_executable_json(&mut self) -> &mut GsonBuilder {
        self.generate_non_executable_json = true;
        self
    }

    /// Writes `<`, `>`, `&`, `=` and `'` in strings as they are, not as Unicode escapes.
    pub fn disable_html_escaping(&mut self) -> &mut GsonBuilder {
        self.escape_html_chars = false;
        self
    }

    /// `setLenient()`.
    pub fn set_lenient(&mut self) -> &mut GsonBuilder {
        self.set_strictness(Strictness::LENIENT)
    }

    pub fn set_strictness(&mut self, strictness: Strictness) -> &mut GsonBuilder {
        self.strictness = Some(strictness);
        self
    }

    pub fn create(&self) -> Gson {
        Gson {
            serialize_nulls: self.serialize_nulls,
            html_safe: self.escape_html_chars,
            generate_non_executable_json: self.generate_non_executable_json,
            strictness: self.strictness,
        }
    }
}

impl Default for GsonBuilder {
    fn default() -> GsonBuilder {
        GsonBuilder::new()
    }
}
