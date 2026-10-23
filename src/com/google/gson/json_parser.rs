//! com/google/gson/JsonParser.java (Gson 2.14.0)
//!
//! Parses JSON into a `JsonElement`. `parseString` is `parseReader(new StringReader(json))`: a
//! reader of the default strictness, which `parseReader(JsonReader)` makes lenient while it reads
//! the value, then a peek in the default strictness that the document ends there, so anything
//! after the value but whitespace is an error (`MalformedJsonException`'s "Use
//! JsonReader.setStrictness(Strictness.LENIENT)…", before the "Did not consume the entire
//! document." check can see it). A JSON null skips that check, and an empty document is JSON null.
//!
//! What Java throws is the `Err`, a `JsonSyntaxException` wrapping the reader's exception (its
//! message is that exception's `toString()`). `JsonIOException`, which wraps a `Reader` that
//! fails, and `JsonParseException`, which wraps a `StackOverflowError`, can't come from a string
//! read by a loop, and aren't ported. The deprecated instance methods aren't either.

/*
 * Copyright (C) 2009 Google Inc.
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

use crate::com::google::gson::internal::streams::Streams;
use crate::com::google::gson::json_element::JsonElement;
use crate::com::google::gson::json_syntax_exception::JsonSyntaxException;
use crate::com::google::gson::stream::json_reader::JsonReader;
use crate::com::google::gson::stream::json_token::JsonToken;
use crate::com::google::gson::stream::malformed_json_exception::MalformedJsonException;
use crate::com::google::gson::strictness::Strictness;
use crate::java::lang::number_format_exception::NumberFormatException;
use crate::java::lang::throwable::Throwable;
use crate::net::minecraft::util::strict_json_parser::StrictJsonParser;

pub struct JsonParser;

impl JsonParser {
    /// `parseString(json)`, the body of `parseReader(Reader)` over a `StringReader` of it: the
    /// value, leniently, and nothing after it.
    pub fn parse_string(json: &str) -> Result<JsonElement, Throwable> {
        let mut json_reader = JsonReader::new(json);
        let result = JsonParser::parse_reader(&mut json_reader).and_then(|element| {
            if !element.is_json_null() && json_reader.peek()? != JsonToken::END_DOCUMENT {
                Err(JsonSyntaxException::new(
                    "Did not consume the entire document.",
                ))
            } else {
                Ok(element)
            }
        });
        result.map_err(|error| match error.class_name() {
            MalformedJsonException::CLASS_NAME | NumberFormatException::CLASS_NAME => {
                JsonSyntaxException::from_cause(error)
            }
            _ => error,
        })
    }

    /// `parseReader(JsonReader)`: the reader's next value, read leniently if its strictness is
    /// the default `LEGACY_STRICT` (then put back), and as strictly as it's set otherwise.
    pub fn parse_reader(reader: &mut JsonReader) -> Result<JsonElement, Throwable> {
        let strictness = reader.get_strictness();
        if strictness == Strictness::LEGACY_STRICT {
            // For backward compatibility, a reader with the default strictness becomes lenient.
            reader.set_strictness(Strictness::LENIENT);
        }
        let result = Streams::parse(reader);
        reader.set_strictness(strictness);
        result
    }

    /// `StrictJsonParser.parse(String)`, how block models and block states are read, with its
    /// exception's `toString()` as the `Err`, which is what the render track's callers pass on.
    pub fn parse_strict(text: &str) -> Result<JsonElement, String> {
        StrictJsonParser::parse(text).map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::JsonParser;
    use crate::com::google::gson::json_element::JsonElement;
    use crate::golden::{Encode, Golden};
    use crate::java::lang::throwable::Throwable;
    use crate::net::minecraft::util::strict_json_parser::StrictJsonParser;

    /// GsonSuites' deepest element encoded whole: serde_json's recursion limit is 128.
    const MAX_ENCODED_DEPTH: usize = 40;

    /// A parsed element, encoded like any shallow enough, and as `["deep", depth, toString]`
    /// otherwise.
    struct Parsed(JsonElement);

    fn depth(element: &JsonElement) -> usize {
        match element {
            JsonElement::Array(elements) => 1 + elements.iter().map(depth).max().unwrap_or(0),
            JsonElement::Object(object) => {
                1 + object
                    .entry_set()
                    .map(|(_, value)| depth(value))
                    .max()
                    .unwrap_or(0)
            }
            _ => 0,
        }
    }

    impl Encode for Parsed {
        fn encode(&self) -> Value {
            let depth = depth(&self.0);
            if depth <= MAX_ENCODED_DEPTH {
                self.0.encode()
            } else {
                Value::from(vec![
                    Value::from("deep"),
                    Value::from(depth),
                    Value::from(self.0.to_string()),
                ])
            }
        }
    }

    fn parsed(result: Result<JsonElement, Throwable>) -> Parsed {
        Parsed(result.unwrap_or_else(|error| panic!("{error}")))
    }

    fn thrown(result: Result<JsonElement, Throwable>) -> (String, String) {
        let error = result.expect_err("vanilla threw");
        let message = error.get_message().unwrap_or("null").to_owned();
        (error.class_name().to_owned(), message)
    }

    /// Valid JSON, every lenient extension, malformed input, nesting past the limit, tokens
    /// across the reader's buffer, random documents and mutations of them: read leniently by
    /// `parseString`, and strictly by `StrictJsonParser`.
    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("com.google.gson.JsonParser");
        golden.check(
            "parseString(Ljava/lang/String;)Lcom/google/gson/JsonElement;",
            |json: String| parsed(JsonParser::parse_string(&json)),
        );
        golden.check(
            "parseString(Ljava/lang/String;)Lcom/google/gson/JsonElement; message",
            |json: String| thrown(JsonParser::parse_string(&json)),
        );
        golden.check(
            "StrictJsonParser.parse(Ljava/lang/String;)Lcom/google/gson/JsonElement;",
            |json: String| parsed(StrictJsonParser::parse(&json)),
        );
        golden.check(
            "StrictJsonParser.parse(Ljava/lang/String;)Lcom/google/gson/JsonElement; message",
            |json: String| thrown(StrictJsonParser::parse(&json)),
        );
        golden.finish();
    }

    #[test]
    fn keeps_numbers_as_text_and_members_in_order() {
        let element = JsonParser::parse_strict(r#"{"b": [1.50, -0, 1e3], "a": "é", "b": true}"#)
            .expect("valid JSON");
        let object = element.get_as_json_object();
        let names: Vec<_> = object.entry_set().map(|(name, _)| name.clone()).collect();
        assert_eq!(names, ["b", "a"]);
        assert_eq!(
            object.get("b"),
            Some(&JsonParser::parse_strict("true").unwrap())
        );
        assert_eq!(object.get("a").unwrap().get_as_string(), "é");
        let numbers = JsonParser::parse_strict("[1.50, -0, 1e3]").unwrap();
        let JsonElement::Array(numbers) = numbers else {
            panic!()
        };
        let texts: Vec<_> = numbers.iter().map(JsonElement::get_as_string).collect();
        assert_eq!(texts, ["1.50", "-0", "1e3"]);
        assert_eq!(numbers[1].get_as_float().to_bits(), (-0.0f32).to_bits());
    }

    #[test]
    fn refuses_what_strict_json_refuses() {
        for text in ["{a: 1}", "[1,]", "01", "'x'", "[1] 2", "NaN", "// x\n1"] {
            assert!(JsonParser::parse_strict(text).is_err(), "{text}");
        }
        // Like Java's, an empty document is JSON null.
        assert_eq!(JsonParser::parse_strict(""), Ok(JsonElement::Null));
    }
}
