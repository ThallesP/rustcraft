//! com/google/gson/stream/JsonWriter.java (Gson 2.14.0)
//!
//! Writes JSON one token at a time into a `String`, Java's `StringBuilder` behind
//! `Streams.writerForAppendable`, which `Gson.toJson` and `JsonElement.toString()` write into. Only
//! the compact format is ported: `setIndent` and `setFormattingStyle` (pretty printing), which
//! nothing the game writes uses, aren't, so `newline()` and the spaces after separators never
//! write anything. Neither is the deprecated `setLenient`.
//!
//! What Java throws is the `Err`: `IllegalStateException` for tokens in the wrong place,
//! `IllegalArgumentException` for a number that isn't JSON (NaN and infinities unless lenient, a
//! `LazilyParsedNumber` whose text isn't a JSON number, which the strictness doesn't excuse), and
//! `IOException` from `close` on an incomplete document. Writing to a `String` can't fail.

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

use crate::com::google::gson::stream::json_scope::{
    DANGLING_NAME, EMPTY_ARRAY, EMPTY_DOCUMENT, EMPTY_OBJECT, NONEMPTY_ARRAY, NONEMPTY_DOCUMENT,
    NONEMPTY_OBJECT,
};
use crate::com::google::gson::strictness::Strictness;
use crate::java::lang::double::Double;
use crate::java::lang::float::Float;
use crate::java::lang::number::Number;
use crate::java::lang::throwable::{
    ILLEGAL_ARGUMENT_EXCEPTION, ILLEGAL_STATE_EXCEPTION, IO_EXCEPTION, Throwable,
};

/// How each ASCII character is written in a string: escaped, or `None` for as it is. U+2028 and
/// U+2029, which JavaScript reads as newlines, are escaped too.
const REPLACEMENT_CHARS: [Option<&str>; 128] = replacement_chars(false);
/// The same, with HTML's special characters escaped, so the JSON can go in an HTML page.
const HTML_SAFE_REPLACEMENT_CHARS: [Option<&str>; 128] = replacement_chars(true);

const fn replacement_chars(html_safe: bool) -> [Option<&'static str>; 128] {
    // `String.format("\\u%04x", i)` for each control character.
    const CONTROLS: [&str; 32] = [
        "\\u0000", "\\u0001", "\\u0002", "\\u0003", "\\u0004", "\\u0005", "\\u0006", "\\u0007",
        "\\u0008", "\\u0009", "\\u000a", "\\u000b", "\\u000c", "\\u000d", "\\u000e", "\\u000f",
        "\\u0010", "\\u0011", "\\u0012", "\\u0013", "\\u0014", "\\u0015", "\\u0016", "\\u0017",
        "\\u0018", "\\u0019", "\\u001a", "\\u001b", "\\u001c", "\\u001d", "\\u001e", "\\u001f",
    ];
    let mut replacements = [None; 128];
    let mut i = 0;
    while i <= 0x1f {
        replacements[i] = Some(CONTROLS[i]);
        i += 1;
    }
    replacements[b'"' as usize] = Some("\\\"");
    replacements[b'\\' as usize] = Some("\\\\");
    replacements[b'\t' as usize] = Some("\\t");
    replacements[0x08] = Some("\\b");
    replacements[b'\n' as usize] = Some("\\n");
    replacements[b'\r' as usize] = Some("\\r");
    replacements[0x0c] = Some("\\f");
    if html_safe {
        replacements[b'<' as usize] = Some("\\u003c");
        replacements[b'>' as usize] = Some("\\u003e");
        replacements[b'&' as usize] = Some("\\u0026");
        replacements[b'=' as usize] = Some("\\u003d");
        replacements[b'\'' as usize] = Some("\\u0027");
    }
    replacements
}

pub struct JsonWriter<'a> {
    out: &'a mut String,

    /// The nesting stack, the kind of each open scope.
    stack: Vec<i32>,

    strictness: Strictness,
    html_safe: bool,
    /// A name written but not yet put out, which waits for its value (a null one drops it).
    deferred_name: Option<String>,
    serialize_nulls: bool,
}

impl<'a> JsonWriter<'a> {
    /// `new JsonWriter(out)`: `LEGACY_STRICT`, not HTML-safe, writing nulls.
    pub fn new(out: &'a mut String) -> JsonWriter<'a> {
        JsonWriter {
            out,
            stack: vec![EMPTY_DOCUMENT],
            strictness: Strictness::LEGACY_STRICT,
            html_safe: false,
            deferred_name: None,
            serialize_nulls: true,
        }
    }

    /// Whether the strictness is `LENIENT`.
    pub fn is_lenient(&self) -> bool {
        self.strictness == Strictness::LENIENT
    }

    /// `LENIENT` writes NaN and infinities and several top-level values; the others don't.
    pub fn set_strictness(&mut self, strictness: Strictness) {
        self.strictness = strictness;
    }

    pub fn get_strictness(&self) -> Strictness {
        self.strictness
    }

    /// Whether strings escape `<`, `>`, `&`, `=` and `'`.
    pub fn set_html_safe(&mut self, html_safe: bool) {
        self.html_safe = html_safe;
    }

    pub fn is_html_safe(&self) -> bool {
        self.html_safe
    }

    /// Whether an object's members with null values are written, or dropped with their names.
    pub fn set_serialize_nulls(&mut self, serialize_nulls: bool) {
        self.serialize_nulls = serialize_nulls;
    }

    pub fn get_serialize_nulls(&self) -> bool {
        self.serialize_nulls
    }

    pub fn begin_array(&mut self) -> Result<(), Throwable> {
        self.write_deferred_name()?;
        self.open_scope(EMPTY_ARRAY, '[')
    }

    pub fn end_array(&mut self) -> Result<(), Throwable> {
        self.close_scope(EMPTY_ARRAY, NONEMPTY_ARRAY, ']')
    }

    pub fn begin_object(&mut self) -> Result<(), Throwable> {
        self.write_deferred_name()?;
        self.open_scope(EMPTY_OBJECT, '{')
    }

    pub fn end_object(&mut self) -> Result<(), Throwable> {
        self.close_scope(EMPTY_OBJECT, NONEMPTY_OBJECT, '}')
    }

    fn open_scope(&mut self, empty: i32, open_bracket: char) -> Result<(), Throwable> {
        self.before_value()?;
        self.stack.push(empty);
        self.out.push(open_bracket);
        Ok(())
    }

    fn close_scope(
        &mut self,
        empty: i32,
        nonempty: i32,
        close_bracket: char,
    ) -> Result<(), Throwable> {
        let context = self.peek()?;
        if context != nonempty && context != empty {
            return Err(Throwable::new(ILLEGAL_STATE_EXCEPTION, "Nesting problem."));
        }
        if let Some(deferred_name) = &self.deferred_name {
            return Err(Throwable::new(
                ILLEGAL_STATE_EXCEPTION,
                format!("Dangling name: {deferred_name}"),
            ));
        }

        self.stack.pop();
        self.out.push(close_bracket);
        Ok(())
    }

    /// The value on the top of the stack.
    fn peek(&self) -> Result<i32, Throwable> {
        self.stack
            .last()
            .copied()
            .ok_or_else(|| Throwable::new(ILLEGAL_STATE_EXCEPTION, "JsonWriter is closed."))
    }

    /// Replaces the value on the top of the stack.
    fn replace_top(&mut self, top_of_stack: i32) {
        *self.stack.last_mut().expect("an open scope") = top_of_stack;
    }

    /// The name of an object's member, which the next value goes with.
    pub fn name(&mut self, name: &str) -> Result<(), Throwable> {
        if self.deferred_name.is_some() {
            return Err(Throwable::new(
                ILLEGAL_STATE_EXCEPTION,
                "Already wrote a name, expecting a value.",
            ));
        }
        let context = self.peek()?;
        if context != EMPTY_OBJECT && context != NONEMPTY_OBJECT {
            return Err(Throwable::new(
                ILLEGAL_STATE_EXCEPTION,
                "Please begin an object before writing a name.",
            ));
        }
        self.deferred_name = Some(name.to_owned());
        Ok(())
    }

    fn write_deferred_name(&mut self) -> Result<(), Throwable> {
        if let Some(deferred_name) = self.deferred_name.clone() {
            self.before_name()?;
            self.string(&deferred_name);
            self.deferred_name = None;
        }
        Ok(())
    }

    /// `value(String)`: a string, escaped.
    pub fn value_str(&mut self, value: &str) -> Result<(), Throwable> {
        self.write_deferred_name()?;
        self.before_value()?;
        self.string(value);
        Ok(())
    }

    /// `value(boolean)`.
    pub fn value_bool(&mut self, value: bool) -> Result<(), Throwable> {
        self.write_deferred_name()?;
        self.before_value()?;
        self.out.push_str(if value { "true" } else { "false" });
        Ok(())
    }

    /// `value(float)`: `Float.toString`, NaN and infinities only when lenient.
    pub fn value_f32(&mut self, value: f32) -> Result<(), Throwable> {
        self.write_deferred_name()?;
        if self.strictness != Strictness::LENIENT && (value.is_nan() || value.is_infinite()) {
            return Err(Throwable::new(
                ILLEGAL_ARGUMENT_EXCEPTION,
                format!(
                    "Numeric values must be finite, but was {}",
                    Float::to_string(value)
                ),
            ));
        }
        self.before_value()?;
        self.out.push_str(&Float::to_string(value));
        Ok(())
    }

    /// `value(double)`: `Double.toString`, NaN and infinities only when lenient.
    pub fn value_f64(&mut self, value: f64) -> Result<(), Throwable> {
        self.write_deferred_name()?;
        if self.strictness != Strictness::LENIENT && (value.is_nan() || value.is_infinite()) {
            return Err(Throwable::new(
                ILLEGAL_ARGUMENT_EXCEPTION,
                format!(
                    "Numeric values must be finite, but was {}",
                    Double::to_string(value)
                ),
            ));
        }
        self.before_value()?;
        self.out.push_str(&Double::to_string(value));
        Ok(())
    }

    /// `value(long)`.
    pub fn value_i64(&mut self, value: i64) -> Result<(), Throwable> {
        self.write_deferred_name()?;
        self.before_value()?;
        self.out.push_str(&value.to_string());
        Ok(())
    }

    /// `value(Number)`: the number's `toString()`. NaN and infinities are written only when
    /// lenient; any other text from a class that can make one that isn't JSON (a
    /// `LazilyParsedNumber`; floats and doubles only make those) must be a JSON number.
    pub fn value_number(&mut self, value: &Number) -> Result<(), Throwable> {
        self.write_deferred_name()?;
        let string = value.to_string();

        if !always_creates_valid_json_number(value) {
            // The string is checked before it goes into the JSON.
            if string == "-Infinity" || string == "Infinity" || string == "NaN" {
                if self.strictness != Strictness::LENIENT {
                    return Err(Throwable::new(
                        ILLEGAL_ARGUMENT_EXCEPTION,
                        format!("Numeric values must be finite, but was {string}"),
                    ));
                }
            } else if !matches!(value, Number::Float(_) | Number::Double(_))
                && !is_valid_json_number(&string)
            {
                return Err(Throwable::new(
                    ILLEGAL_ARGUMENT_EXCEPTION,
                    format!(
                        "String created by class {} is not a valid JSON number: {string}",
                        value.class_name()
                    ),
                ));
            }
        }

        self.before_value()?;
        self.out.push_str(&string);
        Ok(())
    }

    /// `nullValue()`: null, or nothing at all for an object's member when nulls aren't written.
    pub fn null_value(&mut self) -> Result<(), Throwable> {
        if self.deferred_name.is_some() {
            if self.serialize_nulls {
                self.write_deferred_name()?;
            } else {
                // Skip the name and the value.
                self.deferred_name = None;
                return Ok(());
            }
        }
        self.before_value()?;
        self.out.push_str("null");
        Ok(())
    }

    /// `jsonValue(String)`: the text as it is, which had better be JSON.
    pub fn json_value(&mut self, value: &str) -> Result<(), Throwable> {
        self.write_deferred_name()?;
        self.before_value()?;
        self.out.push_str(value);
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), Throwable> {
        if self.stack.is_empty() {
            return Err(Throwable::new(
                ILLEGAL_STATE_EXCEPTION,
                "JsonWriter is closed.",
            ));
        }
        Ok(())
    }

    /// Closes the writer, which throws when the document isn't complete.
    pub fn close(&mut self) -> Result<(), Throwable> {
        let size = self.stack.len();
        if size > 1 || (size == 1 && self.stack[size - 1] != NONEMPTY_DOCUMENT) {
            return Err(Throwable::new(IO_EXCEPTION, "Incomplete document"));
        }
        self.stack.clear();
        Ok(())
    }

    fn string(&mut self, value: &str) {
        let replacements = if self.html_safe {
            &HTML_SAFE_REPLACEMENT_CHARS
        } else {
            &REPLACEMENT_CHARS
        };
        self.out.push('"');
        // Java walks UTF-16 units; a supplementary character's two are never replaced, so walking
        // characters writes the same.
        for c in value.chars() {
            let replacement = match c {
                '\u{0}'..='\u{7f}' => replacements[c as usize],
                '\u{2028}' => Some("\\u2028"),
                '\u{2029}' => Some("\\u2029"),
                _ => None,
            };
            match replacement {
                Some(replacement) => self.out.push_str(replacement),
                None => self.out.push(c),
            }
        }
        self.out.push('"');
    }

    fn before_name(&mut self) -> Result<(), Throwable> {
        let context = self.peek()?;
        if context == NONEMPTY_OBJECT {
            // Not the first in the object.
            self.out.push(',');
        } else if context != EMPTY_OBJECT {
            // Not in an object!
            return Err(Throwable::new(ILLEGAL_STATE_EXCEPTION, "Nesting problem."));
        }
        self.replace_top(DANGLING_NAME);
        Ok(())
    }

    /// Readies the stack for a value: a separator before it, and its scope's state after.
    fn before_value(&mut self) -> Result<(), Throwable> {
        match self.peek()? {
            NONEMPTY_DOCUMENT => {
                if self.strictness != Strictness::LENIENT {
                    return Err(Throwable::new(
                        ILLEGAL_STATE_EXCEPTION,
                        "JSON must have only one top-level value.",
                    ));
                }
                self.replace_top(NONEMPTY_DOCUMENT);
            }
            // The first in the document.
            EMPTY_DOCUMENT => self.replace_top(NONEMPTY_DOCUMENT),
            // The first in the array.
            EMPTY_ARRAY => self.replace_top(NONEMPTY_ARRAY),
            // Another in the array.
            NONEMPTY_ARRAY => self.out.push(','),
            // The value for a name.
            DANGLING_NAME => {
                self.out.push(':');
                self.replace_top(NONEMPTY_OBJECT);
            }
            _ => return Err(Throwable::new(ILLEGAL_STATE_EXCEPTION, "Nesting problem.")),
        }
        Ok(())
    }
}

/// The classes whose `toString()` is always a JSON number: not `Float` or `Double` (NaN and
/// infinities) or `LazilyParsedNumber` (any text).
fn always_creates_valid_json_number(value: &Number) -> bool {
    matches!(
        value,
        Number::Integer(_)
            | Number::Long(_)
            | Number::Byte(_)
            | Number::Short(_)
            | Number::BigDecimal(_)
    )
}

/// `VALID_JSON_NUMBER_PATTERN`, RFC 8259's number:
/// `-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?(?:[eE][-+]?[0-9]+)?`, matching the whole string.
fn is_valid_json_number(string: &str) -> bool {
    let bytes = string.as_bytes();
    let mut i = 0;
    let digits = |i: &mut usize| {
        let start = *i;
        while bytes.get(*i).is_some_and(u8::is_ascii_digit) {
            *i += 1;
        }
        *i - start
    };
    if bytes.get(i) == Some(&b'-') {
        i += 1;
    }
    match bytes.get(i) {
        Some(b'0') => i += 1,
        Some(b'1'..=b'9') => {
            digits(&mut i);
        }
        _ => return false,
    }
    if bytes.get(i) == Some(&b'.') {
        i += 1;
        if digits(&mut i) == 0 {
            return false;
        }
    }
    if let Some(b'e' | b'E') = bytes.get(i) {
        i += 1;
        if let Some(b'-' | b'+') = bytes.get(i) {
            i += 1;
        }
        if digits(&mut i) == 0 {
            return false;
        }
    }
    i == bytes.len()
}
