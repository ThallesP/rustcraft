//! com/google/gson/internal/bind/JsonElementTypeAdapter.java (Gson 2.14.0)
//!
//! Reads a `JsonElement` from a `JsonReader` and writes one to a `JsonWriter`. Reading is Java's
//! loop over an explicit stack of the arrays and objects being read, so deep nesting can't
//! overflow the call stack. Java adds a nested array or object to its parent when it begins and
//! fills it in after; here it's added when it ends, which is the same, since nothing else goes into
//! the parent meanwhile: a later member of the same name still replaces it in its place. Writing
//! recurses, like Java's. `JsonTreeReader`'s shortcut isn't ported.

/*
 * Copyright (C) 2024 Google Inc.
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

use crate::com::google::gson::internal::lazily_parsed_number::LazilyParsedNumber;
use crate::com::google::gson::json_element::JsonElement;
use crate::com::google::gson::json_object::JsonObject;
use crate::com::google::gson::json_primitive::JsonPrimitive;
use crate::com::google::gson::stream::json_reader::JsonReader;
use crate::com::google::gson::stream::json_token::JsonToken;
use crate::com::google::gson::stream::json_writer::JsonWriter;
use crate::java::lang::number::Number;
use crate::java::lang::throwable::{ILLEGAL_STATE_EXCEPTION, Throwable};

pub struct JsonElementTypeAdapter;

impl JsonElementTypeAdapter {
    /// Begins reading an array or an object, or `None` if the next element is neither.
    fn try_begin_nesting(
        reader: &mut JsonReader,
        peeked: JsonToken,
    ) -> Result<Option<JsonElement>, Throwable> {
        match peeked {
            JsonToken::BEGIN_ARRAY => {
                reader.begin_array()?;
                Ok(Some(JsonElement::Array(Vec::new())))
            }
            JsonToken::BEGIN_OBJECT => {
                reader.begin_object()?;
                Ok(Some(JsonElement::Object(JsonObject::new())))
            }
            _ => Ok(None),
        }
    }

    /// Reads an element that can't have nested elements. A number is its text.
    fn read_terminal(reader: &mut JsonReader, peeked: JsonToken) -> Result<JsonElement, Throwable> {
        match peeked {
            JsonToken::STRING => Ok(JsonElement::Primitive(JsonPrimitive::String(
                reader.next_string()?,
            ))),
            JsonToken::NUMBER => {
                let number = reader.next_string()?;
                Ok(JsonElement::Primitive(JsonPrimitive::Number(
                    Number::LazilyParsedNumber(LazilyParsedNumber::new(number)),
                )))
            }
            JsonToken::BOOLEAN => Ok(JsonElement::Primitive(JsonPrimitive::Boolean(
                reader.next_boolean()?,
            ))),
            JsonToken::NULL => {
                reader.next_null()?;
                Ok(JsonElement::Null)
            }
            // The reader is in an invalid state.
            _ => Err(Throwable::new(
                ILLEGAL_STATE_EXCEPTION,
                format!("Unexpected token: {peeked}"),
            )),
        }
    }

    pub fn read(reader: &mut JsonReader) -> Result<JsonElement, Throwable> {
        // Either an array or an object.
        let peeked = reader.peek()?;
        let Some(mut current) = JsonElementTypeAdapter::try_begin_nesting(reader, peeked)? else {
            return JsonElementTypeAdapter::read_terminal(reader, peeked);
        };
        // The name `current` goes under in its parent, which is in `stack` with its own.
        let mut current_name = None;
        let mut stack: Vec<(JsonElement, Option<String>)> = Vec::new();

        loop {
            while reader.has_next()? {
                // A name only for an object's members.
                let name = match current {
                    JsonElement::Object(_) => Some(reader.next_name()?),
                    _ => None,
                };

                let peeked = reader.peek()?;
                match JsonElementTypeAdapter::try_begin_nesting(reader, peeked)? {
                    Some(value) => {
                        stack.push((current, current_name));
                        current = value;
                        current_name = name;
                    }
                    None => {
                        let value = JsonElementTypeAdapter::read_terminal(reader, peeked)?;
                        add(&mut current, name, value);
                    }
                }
            }

            // End the current element.
            match current {
                JsonElement::Array(_) => reader.end_array()?,
                _ => reader.end_object()?,
            }

            match stack.pop() {
                None => return Ok(current),
                // Continue with the enclosing element.
                Some((mut parent, parent_name)) => {
                    add(&mut parent, current_name, current);
                    current = parent;
                    current_name = parent_name;
                }
            }
        }
    }

    pub fn write(out: &mut JsonWriter, value: &JsonElement) -> Result<(), Throwable> {
        match value {
            JsonElement::Null => out.null_value(),
            JsonElement::Primitive(JsonPrimitive::Number(number)) => out.value_number(number),
            JsonElement::Primitive(JsonPrimitive::Boolean(value)) => out.value_bool(*value),
            JsonElement::Primitive(JsonPrimitive::String(value)) => out.value_str(value),
            JsonElement::Array(elements) => {
                out.begin_array()?;
                for element in elements {
                    JsonElementTypeAdapter::write(out, element)?;
                }
                out.end_array()
            }
            JsonElement::Object(object) => {
                out.begin_object()?;
                for (name, value) in object.entry_set() {
                    out.name(name)?;
                    JsonElementTypeAdapter::write(out, value)?;
                }
                out.end_object()
            }
        }
    }
}

/// `JsonArray.add(value)`, or `JsonObject.add(name, value)` for an object.
fn add(element: &mut JsonElement, name: Option<String>, value: JsonElement) {
    match element {
        JsonElement::Array(elements) => elements.push(value),
        JsonElement::Object(object) => object.add(name.expect("a member's name"), value),
        _ => unreachable!("only arrays and objects nest"),
    }
}
