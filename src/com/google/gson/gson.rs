//! com/google/gson/Gson.java (Gson 2.14.0)
//!
//! Only what `toJson(JsonElement)` reads of a `Gson`: whether it escapes HTML, writes null
//! members, prefixes the non-execute prefix, and its strictness. Gson's type adapters, reflection
//! and `fromJson` aren't ported: the game reads JSON with `JsonParser` and its codecs. A `Gson` is
//! made with `GsonBuilder`; `new Gson()` is `Gson::new()`.

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

use crate::com::google::gson::gson_builder::GsonBuilder;
use crate::com::google::gson::internal::streams::Streams;
use crate::com::google::gson::json_element::JsonElement;
use crate::com::google::gson::stream::json_writer::JsonWriter;
use crate::com::google::gson::strictness::Strictness;
use crate::java::lang::throwable::Throwable;

const JSON_NON_EXECUTABLE_PREFIX: &str = ")]}'\n";

#[derive(Clone, Debug)]
pub struct Gson {
    pub(super) serialize_nulls: bool,
    pub(super) html_safe: bool,
    pub(super) generate_non_executable_json: bool,
    /// `None` leaves the writer's strictness, `LEGACY_STRICT` becoming `LENIENT` for compatibility.
    pub(super) strictness: Option<Strictness>,
}

impl Gson {
    /// `new Gson()`: HTML-safe, dropping null members, with the default strictness.
    pub fn new() -> Gson {
        GsonBuilder::new().create()
    }

    /// `toJson(JsonElement)`: the element's JSON. What the writer throws is the `Err` (an
    /// `IllegalArgumentException` for a `LazilyParsedNumber` whose text isn't a JSON number, or
    /// for NaN and infinities when the strictness is set and isn't `LENIENT`).
    pub fn to_json(&self, json_element: &JsonElement) -> Result<String, Throwable> {
        let mut writer = String::new();
        let mut json_writer = self.new_json_writer(&mut writer);
        self.to_json_writer(json_element, &mut json_writer)?;
        Ok(writer)
    }

    /// `toJson(JsonElement, JsonWriter)`: writes the element with this Gson's settings, then puts
    /// the writer's back.
    pub fn to_json_writer(
        &self,
        json_element: &JsonElement,
        writer: &mut JsonWriter,
    ) -> Result<(), Throwable> {
        let old_strictness = writer.get_strictness();
        let old_html_safe = writer.is_html_safe();
        let old_serialize_nulls = writer.get_serialize_nulls();

        writer.set_html_safe(self.html_safe);
        writer.set_serialize_nulls(self.serialize_nulls);

        if let Some(strictness) = self.strictness {
            writer.set_strictness(strictness);
        } else if writer.get_strictness() == Strictness::LEGACY_STRICT {
            // For backward compatibility, a writer with the default strictness becomes lenient.
            writer.set_strictness(Strictness::LENIENT);
        }

        let result = Streams::write(json_element, writer);
        writer.set_strictness(old_strictness);
        writer.set_html_safe(old_html_safe);
        writer.set_serialize_nulls(old_serialize_nulls);
        result
    }

    /// `newJsonWriter(Writer)`: a writer configured like this Gson, after the non-execute prefix
    /// when it writes one.
    pub fn new_json_writer<'a>(&self, writer: &'a mut String) -> JsonWriter<'a> {
        if self.generate_non_executable_json {
            writer.push_str(JSON_NON_EXECUTABLE_PREFIX);
        }
        let mut json_writer = JsonWriter::new(writer);
        json_writer.set_html_safe(self.html_safe);
        json_writer.set_strictness(self.strictness.unwrap_or(Strictness::LEGACY_STRICT));
        json_writer.set_serialize_nulls(self.serialize_nulls);
        json_writer
    }
}

impl Default for Gson {
    fn default() -> Gson {
        Gson::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::com::google::gson::gson::Gson;
    use crate::com::google::gson::gson_builder::GsonBuilder;
    use crate::com::google::gson::json_element::JsonElement;
    use crate::com::google::gson::strictness::Strictness;
    use crate::golden::Golden;

    fn thrown(result: Result<String, crate::java::lang::throwable::Throwable>) -> (String, String) {
        let error = result.expect_err("vanilla threw");
        let message = error.get_message().unwrap_or("null").to_owned();
        (error.class_name().to_owned(), message)
    }

    /// `toJson` of every element GsonSuites writes, by `ByteBufCodecs.lenientJson`'s Gson (HTML
    /// escaping off) and a default one, a third of them by one that writes nulls after the
    /// non-execute prefix, and every primitive by a strict one.
    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("com.google.gson.Gson");
        let lenient_json = GsonBuilder::new().disable_html_escaping().create();
        let default = Gson::new();
        let to_json = "toJson(Lcom/google/gson/JsonElement;)Ljava/lang/String;";
        golden.check(to_json, |element: JsonElement| {
            lenient_json
                .to_json(&element)
                .unwrap_or_else(|error| panic!("{error}"))
        });
        golden.check(&format!("{to_json} message"), |element: JsonElement| {
            thrown(lenient_json.to_json(&element))
        });
        golden.check(&format!("new Gson().{to_json}"), |element: JsonElement| {
            default
                .to_json(&element)
                .unwrap_or_else(|error| panic!("{error}"))
        });
        golden.check(
            &format!("new Gson().{to_json} message"),
            |element: JsonElement| thrown(default.to_json(&element)),
        );
        let nulls = GsonBuilder::new()
            .serialize_nulls()
            .generate_non_executable_json()
            .create();
        golden.check(
            &format!("[serializeNulls, generateNonExecutableJson]{to_json}"),
            |element: JsonElement| {
                nulls
                    .to_json(&element)
                    .unwrap_or_else(|error| panic!("{error}"))
            },
        );
        golden.check(
            &format!("[serializeNulls, generateNonExecutableJson]{to_json} message"),
            |element: JsonElement| thrown(nulls.to_json(&element)),
        );
        let strict = GsonBuilder::new()
            .set_strictness(Strictness::STRICT)
            .create();
        golden.check(&format!("[STRICT]{to_json}"), |element: JsonElement| {
            strict
                .to_json(&element)
                .unwrap_or_else(|error| panic!("{error}"))
        });
        golden.check(
            &format!("[STRICT]{to_json} message"),
            |element: JsonElement| thrown(strict.to_json(&element)),
        );
        golden.finish();
    }
}
