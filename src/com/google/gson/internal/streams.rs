//! com/google/gson/internal/Streams.java (Gson 2.14.0)
//!
//! Reading and writing a whole `JsonElement` through `JsonElementTypeAdapter`. `parse` reads an
//! empty document as JSON null, and wraps what the reader throws in `JsonSyntaxException`, Java's
//! `catch` clauses as a `match` on the class. `writerForAppendable` isn't ported: the writer writes
//! into a `String`. Neither is `JsonIOException`, which only wraps a failing `Reader` or `Writer`.

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

use crate::com::google::gson::internal::bind::json_element_type_adapter::JsonElementTypeAdapter;
use crate::com::google::gson::json_element::JsonElement;
use crate::com::google::gson::json_syntax_exception::JsonSyntaxException;
use crate::com::google::gson::stream::json_reader::JsonReader;
use crate::com::google::gson::stream::json_writer::JsonWriter;
use crate::com::google::gson::stream::malformed_json_exception::MalformedJsonException;
use crate::java::lang::number_format_exception::NumberFormatException;
use crate::java::lang::throwable::{EOF_EXCEPTION, Throwable};

pub struct Streams;

impl Streams {
    /// Takes a reader in any state and returns the next value as a `JsonElement`.
    pub fn parse(reader: &mut JsonReader) -> Result<JsonElement, Throwable> {
        let mut is_empty = true;
        let result = reader.peek().and_then(|_| {
            is_empty = false;
            JsonElementTypeAdapter::read(reader)
        });
        result.or_else(|error| match error.class_name() {
            // The stream ended before a value began: an empty document, which is JSON null.
            EOF_EXCEPTION if is_empty => Ok(JsonElement::Null),
            // The stream ended prematurely, so it's likely a syntax error.
            EOF_EXCEPTION
            | MalformedJsonException::CLASS_NAME
            | NumberFormatException::CLASS_NAME => Err(JsonSyntaxException::from_cause(error)),
            _ => Err(error),
        })
    }

    /// Writes the element to the writer.
    pub fn write(element: &JsonElement, writer: &mut JsonWriter) -> Result<(), Throwable> {
        JsonElementTypeAdapter::write(writer, element)
    }
}
