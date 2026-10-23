//! com/google/gson/stream/JsonReader.java (Gson 2.14.0)
//!
//! Reads a JSON value as a stream of tokens, in any of the three strictnesses (`Strictness`), with
//! Gson's nesting limit and lenient extensions. It reads a `&str` the way Java's reads a
//! `StringReader` of the string: its UTF-16 units, into the same 1024-unit buffer, refilled at the
//! same points. So positions, lines and columns in messages count UTF-16 units, and a token across
//! the buffer's end comes out as Java's does: a number of 1024 units or more is too long to be one,
//! and reads as an unquoted string where that's allowed. One quirk comes with the buffer: input
//! starting with two byte order marks whose lenient unquoted value runs past the first buffer
//! makes `fillBuffer` wait forever for a unit there's no room for, in Java and here.
//!
//! What Java throws is the `Err`, a `Throwable` with Java's class and message: a
//! `MalformedJsonException` (with the location and the troubleshooting URL), an `EOFException` at
//! the end of the input, an `IllegalStateException` for a token read the wrong way, a
//! `NumberFormatException` from `nextInt`, `nextLong` and `nextDouble`. A call that fails leaves
//! the reader as Java's leaves it.
//!
//! Strings it returns are Rust's, which can't hold half of a surrogate pair: an escape of a lone
//! surrogate (`"\ud800"`) reads as U+FFFD, like modified UTF-8's (PORTING.md). The deprecated
//! `setLenient` and `promoteNameToValue` (`JsonReaderInternalAccess`, for maps with complex keys)
//! aren't ported, and closing only closes the reader: there's no stream behind it.

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
use std::fmt::Write;

use crate::com::google::gson::internal::troubleshooting_guide::TroubleshootingGuide;
use crate::com::google::gson::stream::json_scope::{
    CLOSED, DANGLING_NAME, EMPTY_ARRAY, EMPTY_DOCUMENT, EMPTY_OBJECT, NONEMPTY_ARRAY,
    NONEMPTY_DOCUMENT, NONEMPTY_OBJECT,
};
use crate::com::google::gson::stream::json_token::JsonToken;
use crate::com::google::gson::stream::malformed_json_exception::MalformedJsonException;
use crate::com::google::gson::strictness::Strictness;
use crate::java::lang::double::Double;
use crate::java::lang::integer::Integer;
use crate::java::lang::long::Long;
use crate::java::lang::number_format_exception::NumberFormatException;
use crate::java::lang::throwable::{
    EOF_EXCEPTION, ILLEGAL_ARGUMENT_EXCEPTION, ILLEGAL_STATE_EXCEPTION, Throwable,
};

const MIN_INCOMPLETE_INTEGER: i64 = i64::MIN / 10;

const PEEKED_NONE: i32 = 0;
const PEEKED_BEGIN_OBJECT: i32 = 1;
const PEEKED_END_OBJECT: i32 = 2;
const PEEKED_BEGIN_ARRAY: i32 = 3;
const PEEKED_END_ARRAY: i32 = 4;
const PEEKED_TRUE: i32 = 5;
const PEEKED_FALSE: i32 = 6;
const PEEKED_NULL: i32 = 7;
const PEEKED_SINGLE_QUOTED: i32 = 8;
const PEEKED_DOUBLE_QUOTED: i32 = 9;
const PEEKED_UNQUOTED: i32 = 10;
/// When this is peeked, the string value is in `peeked_string`.
const PEEKED_BUFFERED: i32 = 11;
const PEEKED_SINGLE_QUOTED_NAME: i32 = 12;
const PEEKED_DOUBLE_QUOTED_NAME: i32 = 13;
const PEEKED_UNQUOTED_NAME: i32 = 14;
/// When this is peeked, the integer value is in `peeked_long`.
const PEEKED_LONG: i32 = 15;
const PEEKED_NUMBER: i32 = 16;
const PEEKED_EOF: i32 = 17;

// The state machine when parsing numbers.
const NUMBER_CHAR_NONE: i32 = 0;
const NUMBER_CHAR_SIGN: i32 = 1;
const NUMBER_CHAR_DIGIT: i32 = 2;
const NUMBER_CHAR_DECIMAL: i32 = 3;
const NUMBER_CHAR_FRACTION_DIGIT: i32 = 4;
const NUMBER_CHAR_EXP_E: i32 = 5;
const NUMBER_CHAR_EXP_SIGN: i32 = 6;
const NUMBER_CHAR_EXP_DIGIT: i32 = 7;

/// The default nesting limit, Moshi's.
pub const DEFAULT_NESTING_LIMIT: i32 = 255;

const BUFFER_SIZE: usize = 1024;

/// `c` as the UTF-16 unit Java's char literal is.
const fn unit(c: u8) -> u16 {
    c as u16
}

/// A string from UTF-16 units, a lone surrogate as U+FFFD.
fn string(units: &[u16]) -> String {
    String::from_utf16_lossy(units)
}

pub struct JsonReader {
    /// The input, Java's `StringReader`: the string's UTF-16 units, and how many it handed out.
    input: Vec<u16>,
    next: usize,

    strictness: Strictness,
    nesting_limit: i32,

    /// A manual buffer to read and unread upcoming characters, from which literals are decoded:
    /// at least as long as the longest token that can be reported as a number.
    buffer: Vec<u16>,
    pos: usize,
    limit: usize,

    line_number: i32,
    line_start: isize,

    peeked: i32,

    /// A peeked value of only digits, with an optional leading dash. Positive values may not have
    /// a leading 0.
    peeked_long: i64,

    /// The number of characters in a peeked number literal: `pos` moves by this after reading it.
    peeked_number_length: usize,

    /// A peeked string to parse on the next double, long or string, kept when parsing it fails.
    peeked_string: Option<String>,

    /// The nesting stack, Java's array and its size: what's past the size stays, and is read
    /// again when a scope there opens.
    stack: Vec<i32>,
    stack_size: usize,

    /// The path, parallel to the stack: an object's scope holds its current name, an array's its
    /// current index.
    path_names: Vec<Option<String>>,
    path_indices: Vec<i32>,
}

impl JsonReader {
    /// `new JsonReader(new StringReader(input))`, `LEGACY_STRICT` like Java's.
    pub fn new(input: &str) -> JsonReader {
        let mut reader = JsonReader {
            input: input.encode_utf16().collect(),
            next: 0,
            strictness: Strictness::LEGACY_STRICT,
            nesting_limit: DEFAULT_NESTING_LIMIT,
            buffer: vec![0; BUFFER_SIZE],
            pos: 0,
            limit: 0,
            line_number: 0,
            line_start: 0,
            peeked: PEEKED_NONE,
            peeked_long: 0,
            peeked_number_length: 0,
            peeked_string: None,
            stack: vec![0; 32],
            stack_size: 0,
            path_names: vec![None; 32],
            path_indices: vec![0; 32],
        };
        reader.stack[reader.stack_size] = EMPTY_DOCUMENT;
        reader.stack_size += 1;
        reader
    }

    /// Whether the strictness is `LENIENT`.
    pub fn is_lenient(&self) -> bool {
        self.strictness == Strictness::LENIENT
    }

    pub fn set_strictness(&mut self, strictness: Strictness) {
        self.strictness = strictness;
    }

    pub fn get_strictness(&self) -> Strictness {
        self.strictness
    }

    /// How many arrays or objects may be open at once. Java throws `IllegalArgumentException` for
    /// a negative limit, which panics here.
    pub fn set_nesting_limit(&mut self, limit: i32) {
        if limit < 0 {
            panic!("{ILLEGAL_ARGUMENT_EXCEPTION}: Invalid nesting limit: {limit}");
        }
        self.nesting_limit = limit;
    }

    pub fn get_nesting_limit(&self) -> i32 {
        self.nesting_limit
    }

    /// Consumes the beginning of an array.
    pub fn begin_array(&mut self) -> Result<(), Throwable> {
        let mut p = self.peeked;
        if p == PEEKED_NONE {
            p = self.do_peek()?;
        }
        if p == PEEKED_BEGIN_ARRAY {
            self.push(EMPTY_ARRAY)?;
            self.path_indices[self.stack_size - 1] = 0;
            self.peeked = PEEKED_NONE;
            Ok(())
        } else {
            Err(self.unexpected_token_error("BEGIN_ARRAY"))
        }
    }

    /// Consumes the end of the current array.
    pub fn end_array(&mut self) -> Result<(), Throwable> {
        let mut p = self.peeked;
        if p == PEEKED_NONE {
            p = self.do_peek()?;
        }
        if p == PEEKED_END_ARRAY {
            self.stack_size -= 1;
            self.path_indices[self.stack_size - 1] += 1;
            self.peeked = PEEKED_NONE;
            Ok(())
        } else {
            Err(self.unexpected_token_error("END_ARRAY"))
        }
    }

    /// Consumes the beginning of an object.
    pub fn begin_object(&mut self) -> Result<(), Throwable> {
        let mut p = self.peeked;
        if p == PEEKED_NONE {
            p = self.do_peek()?;
        }
        if p == PEEKED_BEGIN_OBJECT {
            self.push(EMPTY_OBJECT)?;
            self.peeked = PEEKED_NONE;
            Ok(())
        } else {
            Err(self.unexpected_token_error("BEGIN_OBJECT"))
        }
    }

    /// Consumes the end of the current object.
    pub fn end_object(&mut self) -> Result<(), Throwable> {
        let mut p = self.peeked;
        if p == PEEKED_NONE {
            p = self.do_peek()?;
        }
        if p == PEEKED_END_OBJECT {
            self.stack_size -= 1;
            // Java frees the last path name.
            self.path_names[self.stack_size] = None;
            self.path_indices[self.stack_size - 1] += 1;
            self.peeked = PEEKED_NONE;
            Ok(())
        } else {
            Err(self.unexpected_token_error("END_OBJECT"))
        }
    }

    /// Whether the current array or object has another element.
    pub fn has_next(&mut self) -> Result<bool, Throwable> {
        let mut p = self.peeked;
        if p == PEEKED_NONE {
            p = self.do_peek()?;
        }
        Ok(p != PEEKED_END_OBJECT && p != PEEKED_END_ARRAY && p != PEEKED_EOF)
    }

    /// The type of the next token, without consuming it.
    pub fn peek(&mut self) -> Result<JsonToken, Throwable> {
        let mut p = self.peeked;
        if p == PEEKED_NONE {
            p = self.do_peek()?;
        }

        Ok(match p {
            PEEKED_BEGIN_OBJECT => JsonToken::BEGIN_OBJECT,
            PEEKED_END_OBJECT => JsonToken::END_OBJECT,
            PEEKED_BEGIN_ARRAY => JsonToken::BEGIN_ARRAY,
            PEEKED_END_ARRAY => JsonToken::END_ARRAY,
            PEEKED_SINGLE_QUOTED_NAME | PEEKED_DOUBLE_QUOTED_NAME | PEEKED_UNQUOTED_NAME => {
                JsonToken::NAME
            }
            PEEKED_TRUE | PEEKED_FALSE => JsonToken::BOOLEAN,
            PEEKED_NULL => JsonToken::NULL,
            PEEKED_SINGLE_QUOTED | PEEKED_DOUBLE_QUOTED | PEEKED_UNQUOTED | PEEKED_BUFFERED => {
                JsonToken::STRING
            }
            PEEKED_LONG | PEEKED_NUMBER => JsonToken::NUMBER,
            PEEKED_EOF => JsonToken::END_DOCUMENT,
            _ => panic!("AssertionError"),
        })
    }

    fn do_peek(&mut self) -> Result<i32, Throwable> {
        let peek_stack = self.stack[self.stack_size - 1];
        if peek_stack == EMPTY_ARRAY {
            self.stack[self.stack_size - 1] = NONEMPTY_ARRAY;
        } else if peek_stack == NONEMPTY_ARRAY {
            // Look for a comma before the next element.
            let c = self.next_non_whitespace(true)?;
            match u8::try_from(c) {
                Ok(b']') => {
                    self.peeked = PEEKED_END_ARRAY;
                    return Ok(self.peeked);
                }
                Ok(b';') => self.check_lenient()?,
                Ok(b',') => {}
                _ => return Err(self.syntax_error("Unterminated array")),
            }
        } else if peek_stack == EMPTY_OBJECT || peek_stack == NONEMPTY_OBJECT {
            self.stack[self.stack_size - 1] = DANGLING_NAME;
            // Look for a comma before the next element.
            if peek_stack == NONEMPTY_OBJECT {
                let c = self.next_non_whitespace(true)?;
                match u8::try_from(c) {
                    Ok(b'}') => {
                        self.peeked = PEEKED_END_OBJECT;
                        return Ok(self.peeked);
                    }
                    Ok(b';') => self.check_lenient()?,
                    Ok(b',') => {}
                    _ => return Err(self.syntax_error("Unterminated object")),
                }
            }
            let c = self.next_non_whitespace(true)?;
            return match u8::try_from(c) {
                Ok(b'"') => {
                    self.peeked = PEEKED_DOUBLE_QUOTED_NAME;
                    Ok(self.peeked)
                }
                Ok(b'\'') => {
                    self.check_lenient()?;
                    self.peeked = PEEKED_SINGLE_QUOTED_NAME;
                    Ok(self.peeked)
                }
                Ok(b'}') => {
                    if peek_stack != NONEMPTY_OBJECT {
                        self.peeked = PEEKED_END_OBJECT;
                        Ok(self.peeked)
                    } else {
                        Err(self.syntax_error("Expected name"))
                    }
                }
                _ => {
                    self.check_lenient()?;
                    // Don't consume the first character of an unquoted string.
                    self.pos -= 1;
                    if self.is_literal(c as u16)? {
                        self.peeked = PEEKED_UNQUOTED_NAME;
                        Ok(self.peeked)
                    } else {
                        Err(self.syntax_error("Expected name"))
                    }
                }
            };
        } else if peek_stack == DANGLING_NAME {
            self.stack[self.stack_size - 1] = NONEMPTY_OBJECT;
            // Look for a colon before the value.
            let c = self.next_non_whitespace(true)?;
            match u8::try_from(c) {
                Ok(b':') => {}
                Ok(b'=') => {
                    self.check_lenient()?;
                    if (self.pos < self.limit || self.fill_buffer(1))
                        && self.buffer[self.pos] == unit(b'>')
                    {
                        self.pos += 1;
                    }
                }
                _ => return Err(self.syntax_error("Expected ':'")),
            }
        } else if peek_stack == EMPTY_DOCUMENT {
            if self.strictness == Strictness::LENIENT {
                self.consume_non_execute_prefix()?;
            }
            self.stack[self.stack_size - 1] = NONEMPTY_DOCUMENT;
        } else if peek_stack == NONEMPTY_DOCUMENT {
            let c = self.next_non_whitespace(false)?;
            if c == -1 {
                self.peeked = PEEKED_EOF;
                return Ok(self.peeked);
            } else {
                self.check_lenient()?;
                self.pos -= 1;
            }
        } else if peek_stack == CLOSED {
            return Err(Throwable::new(
                ILLEGAL_STATE_EXCEPTION,
                "JsonReader is closed",
            ));
        }

        let c = self.next_non_whitespace(true)?;
        match u8::try_from(c) {
            Ok(b']') if peek_stack == EMPTY_ARRAY => {
                self.peeked = PEEKED_END_ARRAY;
                return Ok(self.peeked);
            }
            // "]" falls through to handle ",]".
            Ok(b']' | b';' | b',') => {
                // In lenient mode, a 0-length literal in an array means 'null'.
                return if peek_stack == EMPTY_ARRAY || peek_stack == NONEMPTY_ARRAY {
                    self.check_lenient()?;
                    self.pos -= 1;
                    self.peeked = PEEKED_NULL;
                    Ok(self.peeked)
                } else {
                    Err(self.syntax_error("Unexpected value"))
                };
            }
            Ok(b'\'') => {
                self.check_lenient()?;
                self.peeked = PEEKED_SINGLE_QUOTED;
                return Ok(self.peeked);
            }
            Ok(b'"') => {
                self.peeked = PEEKED_DOUBLE_QUOTED;
                return Ok(self.peeked);
            }
            Ok(b'[') => {
                self.peeked = PEEKED_BEGIN_ARRAY;
                return Ok(self.peeked);
            }
            Ok(b'{') => {
                self.peeked = PEEKED_BEGIN_OBJECT;
                return Ok(self.peeked);
            }
            // Don't consume the first character of a literal value.
            _ => self.pos -= 1,
        }

        let result = self.peek_keyword()?;
        if result != PEEKED_NONE {
            return Ok(result);
        }

        let result = self.peek_number()?;
        if result != PEEKED_NONE {
            return Ok(result);
        }

        if !self.is_literal(self.buffer[self.pos])? {
            return Err(self.syntax_error("Expected value"));
        }

        self.check_lenient()?;
        self.peeked = PEEKED_UNQUOTED;
        Ok(self.peeked)
    }

    fn peek_keyword(&mut self) -> Result<i32, Throwable> {
        // Which keyword to match against, by its first character.
        let c = self.buffer[self.pos];
        let (keyword, keyword_upper, peeking) = match u8::try_from(c) {
            Ok(b't' | b'T') => ("true", "TRUE", PEEKED_TRUE),
            Ok(b'f' | b'F') => ("false", "FALSE", PEEKED_FALSE),
            Ok(b'n' | b'N') => ("null", "NULL", PEEKED_NULL),
            _ => return Ok(PEEKED_NONE),
        };

        // Upper-cased keywords are not allowed in STRICT mode.
        let allows_upper_cased = self.strictness != Strictness::STRICT;

        // Confirm that chars [0..length) match the keyword.
        let length = keyword.len();
        for i in 0..length {
            if self.pos + i >= self.limit && !self.fill_buffer(i + 1) {
                return Ok(PEEKED_NONE);
            }
            let c = self.buffer[self.pos + i];
            let matched = c == unit(keyword.as_bytes()[i])
                || (allows_upper_cased && c == unit(keyword_upper.as_bytes()[i]));
            if !matched {
                return Ok(PEEKED_NONE);
            }
        }

        if (self.pos + length < self.limit || self.fill_buffer(length + 1))
            && self.is_literal(self.buffer[self.pos + length])?
        {
            // Don't match trues, falsey or nullsoft!
            return Ok(PEEKED_NONE);
        }

        // The keyword, followed by the end of the input or a character that isn't a literal's.
        self.pos += length;
        self.peeked = peeking;
        Ok(self.peeked)
    }

    fn peek_number(&mut self) -> Result<i32, Throwable> {
        let mut p = self.pos;
        let mut l = self.limit;

        let mut value: i64 = 0; // Negative to accommodate Long.MIN_VALUE more easily.
        let mut negative = false;
        let mut fits_in_long = true;
        let mut last = NUMBER_CHAR_NONE;

        let mut i = 0;

        loop {
            if p + i == l {
                if i == BUFFER_SIZE {
                    // Though this looks like a well-formed number, it's too long to continue
                    // reading: the application handles it as an unquoted literal.
                    return Ok(PEEKED_NONE);
                }
                if !self.fill_buffer(i + 1) {
                    break;
                }
                p = self.pos;
                l = self.limit;
            }

            let c = self.buffer[p + i];
            match u8::try_from(c) {
                Ok(b'-') => {
                    if last == NUMBER_CHAR_NONE {
                        negative = true;
                        last = NUMBER_CHAR_SIGN;
                    } else if last == NUMBER_CHAR_EXP_E {
                        last = NUMBER_CHAR_EXP_SIGN;
                    } else {
                        return Ok(PEEKED_NONE);
                    }
                }
                Ok(b'+') => {
                    if last == NUMBER_CHAR_EXP_E {
                        last = NUMBER_CHAR_EXP_SIGN;
                    } else {
                        return Ok(PEEKED_NONE);
                    }
                }
                Ok(b'e' | b'E') => {
                    if last == NUMBER_CHAR_DIGIT || last == NUMBER_CHAR_FRACTION_DIGIT {
                        last = NUMBER_CHAR_EXP_E;
                    } else {
                        return Ok(PEEKED_NONE);
                    }
                }
                Ok(b'.') => {
                    if last == NUMBER_CHAR_DIGIT {
                        last = NUMBER_CHAR_DECIMAL;
                    } else {
                        return Ok(PEEKED_NONE);
                    }
                }
                _ => {
                    if c < unit(b'0') || c > unit(b'9') {
                        if !self.is_literal(c)? {
                            break;
                        }
                        return Ok(PEEKED_NONE);
                    }
                    let digit = i64::from(c - unit(b'0'));
                    if last == NUMBER_CHAR_SIGN || last == NUMBER_CHAR_NONE {
                        value = -digit;
                        last = NUMBER_CHAR_DIGIT;
                    } else if last == NUMBER_CHAR_DIGIT {
                        if value == 0 {
                            // A leading '0' prefix is not allowed (since it could be octal).
                            return Ok(PEEKED_NONE);
                        }
                        let new_value = value.wrapping_mul(10).wrapping_sub(digit);
                        fits_in_long &= value > MIN_INCOMPLETE_INTEGER
                            || (value == MIN_INCOMPLETE_INTEGER && new_value < value);
                        value = new_value;
                    } else if last == NUMBER_CHAR_DECIMAL {
                        last = NUMBER_CHAR_FRACTION_DIGIT;
                    } else if last == NUMBER_CHAR_EXP_E || last == NUMBER_CHAR_EXP_SIGN {
                        last = NUMBER_CHAR_EXP_DIGIT;
                    }
                }
            }
            i += 1;
        }

        // A complete number: a PEEKED_LONG or a PEEKED_NUMBER. -0 isn't stored as a long, since
        // it might be read as the double -0.0, and Long.MIN_VALUE can't be made positive.
        if last == NUMBER_CHAR_DIGIT
            && fits_in_long
            && (value != i64::MIN || negative)
            && (value != 0 || !negative)
        {
            self.peeked_long = if negative { value } else { -value };
            self.pos += i;
            self.peeked = PEEKED_LONG;
            Ok(self.peeked)
        } else if last == NUMBER_CHAR_DIGIT
            || last == NUMBER_CHAR_FRACTION_DIGIT
            || last == NUMBER_CHAR_EXP_DIGIT
        {
            self.peeked_number_length = i;
            self.peeked = PEEKED_NUMBER;
            Ok(self.peeked)
        } else {
            Ok(PEEKED_NONE)
        }
    }

    fn is_literal(&self, c: u16) -> Result<bool, Throwable> {
        match u8::try_from(c) {
            Ok(b'/' | b'\\' | b';' | b'#' | b'=') => {
                self.check_lenient()?;
                Ok(false)
            }
            Ok(b'{' | b'}' | b'[' | b']' | b':' | b',' | b' ' | b'\t' | 0x0C | b'\r' | b'\n') => {
                Ok(false)
            }
            _ => Ok(true),
        }
    }

    /// Consumes the next token, a property name, and returns it.
    pub fn next_name(&mut self) -> Result<String, Throwable> {
        let mut p = self.peeked;
        if p == PEEKED_NONE {
            p = self.do_peek()?;
        }
        let result = if p == PEEKED_UNQUOTED_NAME {
            self.next_unquoted_value()?
        } else if p == PEEKED_SINGLE_QUOTED_NAME {
            self.next_quoted_value(unit(b'\''))?
        } else if p == PEEKED_DOUBLE_QUOTED_NAME {
            self.next_quoted_value(unit(b'"'))?
        } else {
            return Err(self.unexpected_token_error("a name"));
        };
        self.peeked = PEEKED_NONE;
        self.path_names[self.stack_size - 1] = Some(result.clone());
        Ok(result)
    }

    /// Consumes the next token, a string, and returns it. A number is returned as its text.
    pub fn next_string(&mut self) -> Result<String, Throwable> {
        let mut p = self.peeked;
        if p == PEEKED_NONE {
            p = self.do_peek()?;
        }
        let result = if p == PEEKED_UNQUOTED {
            self.next_unquoted_value()?
        } else if p == PEEKED_SINGLE_QUOTED {
            self.next_quoted_value(unit(b'\''))?
        } else if p == PEEKED_DOUBLE_QUOTED {
            self.next_quoted_value(unit(b'"'))?
        } else if p == PEEKED_BUFFERED {
            self.peeked_string.take().expect("a buffered string")
        } else if p == PEEKED_LONG {
            self.peeked_long.to_string()
        } else if p == PEEKED_NUMBER {
            let result = string(&self.buffer[self.pos..self.pos + self.peeked_number_length]);
            self.pos += self.peeked_number_length;
            result
        } else {
            return Err(self.unexpected_token_error("a string"));
        };
        self.peeked = PEEKED_NONE;
        self.path_indices[self.stack_size - 1] += 1;
        Ok(result)
    }

    /// Consumes the next token, a boolean, and returns it.
    pub fn next_boolean(&mut self) -> Result<bool, Throwable> {
        let mut p = self.peeked;
        if p == PEEKED_NONE {
            p = self.do_peek()?;
        }
        if p == PEEKED_TRUE {
            self.peeked = PEEKED_NONE;
            self.path_indices[self.stack_size - 1] += 1;
            return Ok(true);
        } else if p == PEEKED_FALSE {
            self.peeked = PEEKED_NONE;
            self.path_indices[self.stack_size - 1] += 1;
            return Ok(false);
        }
        Err(self.unexpected_token_error("a boolean"))
    }

    /// Consumes the next token, a null.
    pub fn next_null(&mut self) -> Result<(), Throwable> {
        let mut p = self.peeked;
        if p == PEEKED_NONE {
            p = self.do_peek()?;
        }
        if p == PEEKED_NULL {
            self.peeked = PEEKED_NONE;
            self.path_indices[self.stack_size - 1] += 1;
            Ok(())
        } else {
            Err(self.unexpected_token_error("null"))
        }
    }

    /// Consumes the next token, a number or a string `Double.parseDouble` reads, and returns it.
    /// NaN and infinities are `MalformedJsonException`s unless the reader is lenient.
    pub fn next_double(&mut self) -> Result<f64, Throwable> {
        let mut p = self.peeked;
        if p == PEEKED_NONE {
            p = self.do_peek()?;
        }

        if p == PEEKED_LONG {
            self.peeked = PEEKED_NONE;
            self.path_indices[self.stack_size - 1] += 1;
            return Ok(self.peeked_long as f64);
        }

        if p == PEEKED_NUMBER {
            self.peeked_string = Some(string(
                &self.buffer[self.pos..self.pos + self.peeked_number_length],
            ));
            self.pos += self.peeked_number_length;
        } else if p == PEEKED_SINGLE_QUOTED || p == PEEKED_DOUBLE_QUOTED {
            let quote = if p == PEEKED_SINGLE_QUOTED {
                b'\''
            } else {
                b'"'
            };
            self.peeked_string = Some(self.next_quoted_value(unit(quote))?);
        } else if p == PEEKED_UNQUOTED {
            self.peeked_string = Some(self.next_unquoted_value()?);
        } else if p != PEEKED_BUFFERED {
            return Err(self.unexpected_token_error("a double"));
        }

        self.peeked = PEEKED_BUFFERED;
        // Java doesn't catch this NumberFormatException.
        let result = Double::parse_double(self.peeked_buffered())?;
        if self.strictness != Strictness::LENIENT && (result.is_nan() || result.is_infinite()) {
            return Err(self.syntax_error(&format!(
                "JSON forbids NaN and infinities: {}",
                Double::to_string(result)
            )));
        }
        self.peeked_string = None;
        self.peeked = PEEKED_NONE;
        self.path_indices[self.stack_size - 1] += 1;
        Ok(result)
    }

    fn peeked_buffered(&self) -> &str {
        self.peeked_string.as_deref().expect("a buffered string")
    }

    /// Consumes the next token, a number or a string, and returns it as a long, which it must be
    /// exactly.
    pub fn next_long(&mut self) -> Result<i64, Throwable> {
        let mut p = self.peeked;
        if p == PEEKED_NONE {
            p = self.do_peek()?;
        }

        if p == PEEKED_LONG {
            self.peeked = PEEKED_NONE;
            self.path_indices[self.stack_size - 1] += 1;
            return Ok(self.peeked_long);
        }

        if p == PEEKED_NUMBER {
            self.peeked_string = Some(string(
                &self.buffer[self.pos..self.pos + self.peeked_number_length],
            ));
            self.pos += self.peeked_number_length;
        } else if p == PEEKED_SINGLE_QUOTED || p == PEEKED_DOUBLE_QUOTED || p == PEEKED_UNQUOTED {
            if p == PEEKED_UNQUOTED {
                self.peeked_string = Some(self.next_unquoted_value()?);
            } else {
                let quote = if p == PEEKED_SINGLE_QUOTED {
                    b'\''
                } else {
                    b'"'
                };
                self.peeked_string = Some(self.next_quoted_value(unit(quote))?);
            }
            self.validate_ascii(self.peeked_buffered())?;
            if let Ok(result) = Long::parse_long(self.peeked_buffered()) {
                self.peeked = PEEKED_NONE;
                self.path_indices[self.stack_size - 1] += 1;
                return Ok(result);
            }
            // Falls back to parsing it as a double below.
        } else {
            return Err(self.unexpected_token_error("a long"));
        }

        self.peeked = PEEKED_BUFFERED;
        // Java doesn't catch this NumberFormatException.
        let as_double = Double::parse_double(self.peeked_buffered())?;
        let result = as_double as i64;
        if result as f64 != as_double {
            // Precision was lost casting to a long.
            return Err(NumberFormatException::new(format!(
                "Expected a long but was {}{}",
                self.peeked_buffered(),
                self.location_string()
            ))
            .into());
        }
        self.peeked_string = None;
        self.peeked = PEEKED_NONE;
        self.path_indices[self.stack_size - 1] += 1;
        Ok(result)
    }

    /// The string up to `quote`, unescaping escapes along the way. The opening quote has been
    /// read; this consumes the closing one.
    fn next_quoted_value(&mut self, quote: u16) -> Result<String, Throwable> {
        let mut builder: Option<Vec<u16>> = None;
        loop {
            let mut p = self.pos;
            let mut l = self.limit;
            // The index of the first character not yet appended to the builder.
            let mut start = p;
            while p < l {
                let c = self.buffer[p];
                p += 1;

                // In strict mode, unescaped control characters (U+0000 through U+001F) are
                // errors.
                if self.strictness == Strictness::STRICT && c < 0x20 {
                    return Err(self.syntax_error(
                        "Unescaped control characters (\\u0000-\\u001F) are not allowed in strict mode",
                    ));
                } else if c == quote {
                    self.pos = p;
                    let len = p - start - 1;
                    return Ok(match builder {
                        None => string(&self.buffer[start..start + len]),
                        Some(mut builder) => {
                            builder.extend_from_slice(&self.buffer[start..start + len]);
                            string(&builder)
                        }
                    });
                } else if c == unit(b'\\') {
                    self.pos = p;
                    let len = p - start - 1;
                    builder
                        .get_or_insert_with(Vec::new)
                        .extend_from_slice(&self.buffer[start..start + len]);
                    let escaped = self.read_escape_character()?;
                    builder.get_or_insert_with(Vec::new).push(escaped);
                    p = self.pos;
                    l = self.limit;
                    start = p;
                } else if c == unit(b'\n') {
                    self.line_number += 1;
                    self.line_start = p as isize;
                }
            }

            builder
                .get_or_insert_with(Vec::new)
                .extend_from_slice(&self.buffer[start..p]);
            self.pos = p;
            if !self.fill_buffer(1) {
                return Err(self.syntax_error("Unterminated string"));
            }
        }
    }

    /// An unquoted value, as a string.
    fn next_unquoted_value(&mut self) -> Result<String, Throwable> {
        let mut builder: Option<Vec<u16>> = None;
        let mut i = 0;

        'find_non_literal_character: loop {
            while self.pos + i < self.limit {
                match u8::try_from(self.buffer[self.pos + i]) {
                    Ok(b'/' | b'\\' | b';' | b'#' | b'=') => {
                        self.check_lenient()?;
                        break 'find_non_literal_character;
                    }
                    Ok(
                        b'{' | b'}' | b'[' | b']' | b':' | b',' | b' ' | b'\t' | 0x0C | b'\r'
                        | b'\n',
                    ) => break 'find_non_literal_character,
                    // The character is part of the value.
                    _ => {}
                }
                i += 1;
            }

            // Try to load the entire literal into the buffer at once.
            if i < BUFFER_SIZE {
                if self.fill_buffer(i + 1) {
                    continue;
                } else {
                    break;
                }
            }

            // A builder when the value is too long. This is too long to be a number!
            builder
                .get_or_insert_with(Vec::new)
                .extend_from_slice(&self.buffer[self.pos..self.pos + i]);
            self.pos += i;
            i = 0;
            if !self.fill_buffer(1) {
                break;
            }
        }

        let result = match builder {
            None => string(&self.buffer[self.pos..self.pos + i]),
            Some(mut builder) => {
                builder.extend_from_slice(&self.buffer[self.pos..self.pos + i]);
                string(&builder)
            }
        };
        self.pos += i;
        Ok(result)
    }

    fn skip_quoted_value(&mut self, quote: u16) -> Result<(), Throwable> {
        loop {
            let mut p = self.pos;
            let mut l = self.limit;
            while p < l {
                let c = self.buffer[p];
                p += 1;
                if c == quote {
                    self.pos = p;
                    return Ok(());
                } else if c == unit(b'\\') {
                    self.pos = p;
                    self.read_escape_character()?;
                    p = self.pos;
                    l = self.limit;
                } else if c == unit(b'\n') {
                    self.line_number += 1;
                    self.line_start = p as isize;
                }
            }
            self.pos = p;
            if !self.fill_buffer(1) {
                break;
            }
        }
        Err(self.syntax_error("Unterminated string"))
    }

    fn skip_unquoted_value(&mut self) -> Result<(), Throwable> {
        loop {
            let mut i = 0;
            while self.pos + i < self.limit {
                match u8::try_from(self.buffer[self.pos + i]) {
                    Ok(b'/' | b'\\' | b';' | b'#' | b'=') => {
                        self.check_lenient()?;
                        self.pos += i;
                        return Ok(());
                    }
                    Ok(
                        b'{' | b'}' | b'[' | b']' | b':' | b',' | b' ' | b'\t' | 0x0C | b'\r'
                        | b'\n',
                    ) => {
                        self.pos += i;
                        return Ok(());
                    }
                    // Skip the character.
                    _ => {}
                }
                i += 1;
            }
            self.pos += i;
            if !self.fill_buffer(1) {
                return Ok(());
            }
        }
    }

    /// Consumes the next token, a number or a string, and returns it as an int, which it must be
    /// exactly.
    pub fn next_int(&mut self) -> Result<i32, Throwable> {
        let mut p = self.peeked;
        if p == PEEKED_NONE {
            p = self.do_peek()?;
        }

        if p == PEEKED_LONG {
            let result = self.peeked_long as i32;
            if self.peeked_long != i64::from(result) {
                // Precision was lost casting to an int.
                return Err(NumberFormatException::new(format!(
                    "Expected an int but was {}{}",
                    self.peeked_long,
                    self.location_string()
                ))
                .into());
            }
            self.peeked = PEEKED_NONE;
            self.path_indices[self.stack_size - 1] += 1;
            return Ok(result);
        }

        if p == PEEKED_NUMBER {
            self.peeked_string = Some(string(
                &self.buffer[self.pos..self.pos + self.peeked_number_length],
            ));
            self.pos += self.peeked_number_length;
        } else if p == PEEKED_SINGLE_QUOTED || p == PEEKED_DOUBLE_QUOTED || p == PEEKED_UNQUOTED {
            if p == PEEKED_UNQUOTED {
                self.peeked_string = Some(self.next_unquoted_value()?);
            } else {
                let quote = if p == PEEKED_SINGLE_QUOTED {
                    b'\''
                } else {
                    b'"'
                };
                self.peeked_string = Some(self.next_quoted_value(unit(quote))?);
            }
            self.validate_ascii(self.peeked_buffered())?;
            if let Ok(result) = Integer::parse_int(self.peeked_buffered()) {
                self.peeked = PEEKED_NONE;
                self.path_indices[self.stack_size - 1] += 1;
                return Ok(result);
            }
            // Falls back to parsing it as a double below.
        } else {
            return Err(self.unexpected_token_error("an int"));
        }

        self.peeked = PEEKED_BUFFERED;
        // Java doesn't catch this NumberFormatException.
        let as_double = Double::parse_double(self.peeked_buffered())?;
        let result = as_double as i32;
        if f64::from(result) != as_double {
            // Precision was lost casting to an int.
            return Err(NumberFormatException::new(format!(
                "Expected an int but was {}{}",
                self.peeked_buffered(),
                self.location_string()
            ))
            .into());
        }
        self.peeked_string = None;
        self.peeked = PEEKED_NONE;
        self.path_indices[self.stack_size - 1] += 1;
        Ok(result)
    }

    /// Closes the reader. Using it after throws `IllegalStateException` in most cases.
    pub fn close(&mut self) {
        self.peeked = PEEKED_NONE;
        self.stack[0] = CLOSED;
        self.stack_size = 1;
    }

    /// Skips the next value recursively: an array or object and everything in it, a primitive, or
    /// only a name (then the value needs skipping too), or an end token. At the end of the
    /// document it does nothing.
    pub fn skip_value(&mut self) -> Result<(), Throwable> {
        let mut count = 0;
        loop {
            let mut p = self.peeked;
            if p == PEEKED_NONE {
                p = self.do_peek()?;
            }

            match p {
                PEEKED_BEGIN_ARRAY => {
                    self.push(EMPTY_ARRAY)?;
                    count += 1;
                }
                PEEKED_BEGIN_OBJECT => {
                    self.push(EMPTY_OBJECT)?;
                    count += 1;
                }
                PEEKED_END_ARRAY => {
                    self.stack_size -= 1;
                    count -= 1;
                }
                PEEKED_END_OBJECT => {
                    // Only when the end of the object is skipped itself: otherwise the stack isn't
                    // read.
                    if count == 0 {
                        self.path_names[self.stack_size - 1] = None;
                    }
                    self.stack_size -= 1;
                    count -= 1;
                }
                PEEKED_UNQUOTED => self.skip_unquoted_value()?,
                PEEKED_SINGLE_QUOTED => self.skip_quoted_value(unit(b'\''))?,
                PEEKED_DOUBLE_QUOTED => self.skip_quoted_value(unit(b'"'))?,
                PEEKED_UNQUOTED_NAME | PEEKED_SINGLE_QUOTED_NAME | PEEKED_DOUBLE_QUOTED_NAME => {
                    match p {
                        PEEKED_UNQUOTED_NAME => self.skip_unquoted_value()?,
                        PEEKED_SINGLE_QUOTED_NAME => self.skip_quoted_value(unit(b'\''))?,
                        _ => self.skip_quoted_value(unit(b'"'))?,
                    }
                    // Only when the name is skipped itself.
                    if count == 0 {
                        self.path_names[self.stack_size - 1] = Some("<skipped>".to_owned());
                    }
                }
                PEEKED_NUMBER => self.pos += self.peeked_number_length,
                PEEKED_EOF => return Ok(()),
                // Every other token has been consumed from the input already.
                _ => {}
            }
            self.peeked = PEEKED_NONE;
            if count <= 0 {
                break;
            }
        }

        self.path_indices[self.stack_size - 1] += 1;
        Ok(())
    }

    fn push(&mut self, new_top: i32) -> Result<(), Throwable> {
        // Java's `stackSize - 1 >= nestingLimit`: minus 1, since the stack's first element is
        // EMPTY_DOCUMENT or NONEMPTY_DOCUMENT.
        if self.stack_size as i32 > self.nesting_limit {
            return Err(MalformedJsonException::new(format!(
                "Nesting limit {} reached{}",
                self.nesting_limit,
                self.location_string()
            )));
        }

        if self.stack_size == self.stack.len() {
            let new_length = self.stack_size * 2;
            self.stack.resize(new_length, 0);
            self.path_indices.resize(new_length, 0);
            self.path_names.resize(new_length, None);
        }
        self.stack[self.stack_size] = new_top;
        self.stack_size += 1;
        Ok(())
    }

    /// `in.read(buffer, limit, buffer.length - limit)`, `StringReader`'s: up to the room left, -1
    /// at the end of the input, and 0 when there's no room, even at the end.
    fn read(&mut self) -> i32 {
        let len = BUFFER_SIZE - self.limit;
        if len == 0 {
            return 0;
        }
        if self.next >= self.input.len() {
            return -1;
        }
        let n = (self.input.len() - self.next).min(len);
        self.buffer[self.limit..self.limit + n]
            .copy_from_slice(&self.input[self.next..self.next + n]);
        self.next += n;
        n as i32
    }

    /// Whether `limit - pos >= minimum` once it's read what it can: false when the input runs out
    /// first.
    fn fill_buffer(&mut self, mut minimum: usize) -> bool {
        self.line_start -= self.pos as isize;
        if self.limit != self.pos {
            self.limit -= self.pos;
            self.buffer.copy_within(self.pos..self.pos + self.limit, 0);
        } else {
            self.limit = 0;
        }

        self.pos = 0;
        loop {
            let total = self.read();
            if total == -1 {
                return false;
            }
            self.limit += total as usize;

            // On the first read, consume a byte order mark if there is one.
            if self.line_number == 0
                && self.line_start == 0
                && self.limit > 0
                && self.buffer[0] == 0xfeff
            {
                self.pos += 1;
                self.line_start += 1;
                minimum += 1;
            }

            if self.limit >= minimum {
                return true;
            }
        }
    }

    /// The next character that's neither whitespace nor part of a comment, or -1 at the end of
    /// the input when that's not an error. It's always at `buffer[pos - 1]` after, so a caller
    /// can push it back by decrementing `pos`.
    fn next_non_whitespace(&mut self, throw_on_eof: bool) -> Result<i32, Throwable> {
        // Java's locals for `pos` and `limit`, which have to be flushed to the fields before
        // anything that can call fill_buffer, and read again after.
        let mut p = self.pos;
        let mut l = self.limit;
        loop {
            if p == l {
                self.pos = p;
                if !self.fill_buffer(1) {
                    break;
                }
                p = self.pos;
                l = self.limit;
            }

            let c = self.buffer[p];
            p += 1;
            if c == unit(b'\n') {
                self.line_number += 1;
                self.line_start = p as isize;
                continue;
            } else if c == unit(b' ') || c == unit(b'\r') || c == unit(b'\t') {
                continue;
            }

            if c == unit(b'/') {
                self.pos = p;
                if p == l {
                    // Push back '/' so it's still in the buffer when this returns.
                    self.pos -= 1;
                    let chars_loaded = self.fill_buffer(2);
                    // Consume the '/' again.
                    self.pos += 1;
                    if !chars_loaded {
                        return Ok(i32::from(c));
                    }
                }

                self.check_lenient()?;
                let peek = self.buffer[self.pos];
                match u8::try_from(peek) {
                    Ok(b'*') => {
                        // Skip a /* c-style comment */.
                        self.pos += 1;
                        if !self.skip_to("*/") {
                            return Err(self.syntax_error("Unterminated comment"));
                        }
                        p = self.pos + 2;
                        l = self.limit;
                        continue;
                    }
                    Ok(b'/') => {
                        // Skip a // end-of-line comment.
                        self.pos += 1;
                        self.skip_to_end_of_line();
                        p = self.pos;
                        l = self.limit;
                        continue;
                    }
                    _ => return Ok(i32::from(c)),
                }
            } else if c == unit(b'#') {
                self.pos = p;
                // Skip a # hash end-of-line comment, which isn't JSON but existing documents
                // have.
                self.check_lenient()?;
                self.skip_to_end_of_line();
                p = self.pos;
                l = self.limit;
            } else {
                self.pos = p;
                return Ok(i32::from(c));
            }
        }
        if throw_on_eof {
            Err(Throwable::new(
                EOF_EXCEPTION,
                format!("End of input{}", self.location_string()),
            ))
        } else {
            Ok(-1)
        }
    }

    fn check_lenient(&self) -> Result<(), Throwable> {
        if self.strictness != Strictness::LENIENT {
            return Err(self.syntax_error(
                "Use JsonReader.setStrictness(Strictness.LENIENT) to accept malformed JSON",
            ));
        }
        Ok(())
    }

    /// Advances past the next newline. A line ended by "\r\n" leaves the '\n' for the caller to
    /// consume as whitespace.
    fn skip_to_end_of_line(&mut self) {
        while self.pos < self.limit || self.fill_buffer(1) {
            let c = self.buffer[self.pos];
            self.pos += 1;
            if c == unit(b'\n') {
                self.line_number += 1;
                self.line_start = self.pos as isize;
                break;
            } else if c == unit(b'\r') {
                break;
            }
        }
    }

    /// Advances to the start of `to_find`, which mustn't hold a newline.
    fn skip_to(&mut self, to_find: &str) -> bool {
        let to_find = to_find.as_bytes();
        let length = to_find.len();
        'outer: while self.pos + length <= self.limit || self.fill_buffer(length) {
            if self.buffer[self.pos] == unit(b'\n') {
                self.line_number += 1;
                self.line_start = self.pos as isize + 1;
                self.pos += 1;
                continue;
            }
            for (c, &expected) in to_find.iter().enumerate() {
                if self.buffer[self.pos + c] != unit(expected) {
                    self.pos += 1;
                    continue 'outer;
                }
            }
            return true;
        }
        false
    }

    fn location_string(&self) -> String {
        let line = self.line_number + 1;
        let column = self.pos as isize - self.line_start + 1;
        format!(" at line {line} column {column} path {}", self.get_path())
    }

    fn get_path_with(&self, use_previous_path: bool) -> String {
        let mut result = String::from("$");
        for i in 0..self.stack_size {
            let scope = self.stack[i];
            match scope {
                EMPTY_ARRAY | NONEMPTY_ARRAY => {
                    let mut path_index = self.path_indices[i];
                    // The last index points to the next element; the previous one is before it.
                    if use_previous_path && path_index > 0 && i == self.stack_size - 1 {
                        path_index -= 1;
                    }
                    write!(result, "[{path_index}]").expect("a String takes any write");
                }
                EMPTY_OBJECT | DANGLING_NAME | NONEMPTY_OBJECT => {
                    result.push('.');
                    if let Some(name) = &self.path_names[i] {
                        result.push_str(name);
                    }
                }
                NONEMPTY_DOCUMENT | EMPTY_DOCUMENT | CLOSED => {}
                _ => panic!("AssertionError: Unknown scope value: {scope}"),
            }
        }
        result
    }

    /// A JSONPath in dot notation to the next (or current) location: in an array, the next
    /// element's index; in an object, the last property, or the current one if its name has been
    /// consumed.
    pub fn get_path(&self) -> String {
        self.get_path_with(false)
    }

    /// Like `get_path`, but in an array the previous element's index (0 before any).
    pub fn get_previous_path(&self) -> String {
        self.get_path_with(true)
    }

    /// Unescapes the escape after a backslash, which has been read: Unicode escapes ("u000A") and
    /// two-character ones ("\n").
    fn read_escape_character(&mut self) -> Result<u16, Throwable> {
        if self.pos == self.limit && !self.fill_buffer(1) {
            return Err(self.syntax_error("Unterminated escape sequence"));
        }

        let escaped = self.buffer[self.pos];
        self.pos += 1;
        match u8::try_from(escaped) {
            Ok(b'u') => {
                if self.pos + 4 > self.limit && !self.fill_buffer(4) {
                    return Err(self.syntax_error("Unterminated escape sequence"));
                }
                // Equivalent to Integer.parseInt(stringPool.get(buffer, pos, 4), 16).
                let mut result: u16 = 0;
                for i in self.pos..self.pos + 4 {
                    let c = self.buffer[i];
                    result <<= 4;
                    if (unit(b'0')..=unit(b'9')).contains(&c) {
                        result += c - unit(b'0');
                    } else if (unit(b'a')..=unit(b'f')).contains(&c) {
                        result += c - unit(b'a') + 10;
                    } else if (unit(b'A')..=unit(b'F')).contains(&c) {
                        result += c - unit(b'A') + 10;
                    } else {
                        return Err(self.syntax_error(&format!(
                            "Malformed Unicode escape \\u{}",
                            string(&self.buffer[self.pos..self.pos + 4])
                        )));
                    }
                }
                self.pos += 4;
                Ok(result)
            }
            Ok(b't') => Ok(unit(b'\t')),
            Ok(b'b') => Ok(0x08),
            Ok(b'n') => Ok(unit(b'\n')),
            Ok(b'r') => Ok(unit(b'\r')),
            Ok(b'f') => Ok(0x0C),
            Ok(b'\n') => {
                if self.strictness == Strictness::STRICT {
                    return Err(
                        self.syntax_error("Cannot escape a newline character in strict mode")
                    );
                }
                self.line_number += 1;
                self.line_start = self.pos as isize;
                // Java falls through to the '\'' case, which strict mode has already left.
                Ok(escaped)
            }
            Ok(b'\'') => {
                if self.strictness == Strictness::STRICT {
                    return Err(self.syntax_error("Invalid escaped character \"'\" in strict mode"));
                }
                Ok(escaped)
            }
            Ok(b'"' | b'\\' | b'/') => Ok(escaped),
            _ => Err(self.syntax_error("Invalid escape sequence")),
        }
    }

    /// A `MalformedJsonException` with the message, the location and the troubleshooting URL.
    fn syntax_error(&self, message: &str) -> Throwable {
        MalformedJsonException::new(format!(
            "{message}{}\nSee {}",
            self.location_string(),
            TroubleshootingGuide::create_url("malformed-json")
        ))
    }

    /// An `IllegalStateException` naming the token that was there, or what peeking it threw.
    fn unexpected_token_error(&mut self, expected: &str) -> Throwable {
        let peeked = match self.peek() {
            Ok(peeked) => peeked,
            Err(error) => return error,
        };
        let troubleshooting_id = if peeked == JsonToken::NULL {
            "adapter-not-null-safe"
        } else {
            "unexpected-json-structure"
        };
        Throwable::new(
            ILLEGAL_STATE_EXCEPTION,
            format!(
                "Expected {expected} but was {peeked}{}\nSee {}",
                self.location_string(),
                TroubleshootingGuide::create_url(troubleshooting_id)
            ),
        )
    }

    /// Consumes the non-execute prefix `)]}'` and a newline, if it's there.
    fn consume_non_execute_prefix(&mut self) -> Result<(), Throwable> {
        // Fast-forward through the leading whitespace.
        self.next_non_whitespace(true)?;
        self.pos -= 1;

        if self.pos + 5 > self.limit && !self.fill_buffer(5) {
            return Ok(());
        }

        let p = self.pos;
        let buf = &self.buffer;
        if buf[p] != unit(b')')
            || buf[p + 1] != unit(b']')
            || buf[p + 2] != unit(b'}')
            || buf[p + 3] != unit(b'\'')
            || buf[p + 4] != unit(b'\n')
        {
            // Not a security token!
            return Ok(());
        }

        // A security token was consumed!
        self.pos += 5;
        Ok(())
    }

    fn validate_ascii(&self, s: &str) -> Result<(), Throwable> {
        if !s.is_ascii() {
            return Err(self.syntax_error(&format!("String contains non-ASCII characters: {s}")));
        }
        Ok(())
    }
}

/// `toString()`: the class's simple name and the location.
impl fmt::Display for JsonReader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JsonReader{}", self.location_string())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::JsonReader;
    use crate::com::google::gson::strictness::Strictness;
    use crate::golden::{Encode, Golden};
    use crate::java::lang::throwable::Throwable;

    /// Programs reading inputs of every kind in each strictness: each op's output, with the path
    /// and the previous path after it, or what it threw, which ends the program.
    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("com.google.gson.stream.JsonReader");
        golden.check(
            "programs",
            |strictness: String, nesting_limit: i32, input: String, ops: Vec<String>| {
                let mut reader = JsonReader::new(&input);
                reader.set_strictness(match strictness.as_str() {
                    "LENIENT" => Strictness::LENIENT,
                    "LEGACY_STRICT" => Strictness::LEGACY_STRICT,
                    _ => Strictness::STRICT,
                });
                reader.set_nesting_limit(nesting_limit);
                let mut outputs: Vec<Box<dyn Encode>> = Vec::new();
                for op in &ops {
                    let output: Result<Box<dyn Encode>, Throwable> = match op.as_str() {
                        "peek" => reader
                            .peek()
                            .map(|token| Box::new(token.name().to_owned()) as Box<dyn Encode>),
                        "hasNext" => reader
                            .has_next()
                            .map(|next| Box::new(next) as Box<dyn Encode>),
                        "toString" => Ok(Box::new(reader.to_string())),
                        "beginArray" => reader.begin_array().map(|()| Box::new(Value::Null) as _),
                        "endArray" => reader.end_array().map(|()| Box::new(Value::Null) as _),
                        "beginObject" => reader.begin_object().map(|()| Box::new(Value::Null) as _),
                        "endObject" => reader.end_object().map(|()| Box::new(Value::Null) as _),
                        "skipValue" => reader.skip_value().map(|()| Box::new(Value::Null) as _),
                        "nextNull" => reader.next_null().map(|()| Box::new(Value::Null) as _),
                        "nextName" => reader.next_name().map(|name| Box::new(name) as _),
                        "nextString" => reader.next_string().map(|string| Box::new(string) as _),
                        "nextBoolean" => reader.next_boolean().map(|value| Box::new(value) as _),
                        "nextDouble" => reader.next_double().map(|value| Box::new(value) as _),
                        "nextLong" => reader.next_long().map(|value| Box::new(value) as _),
                        "nextInt" => reader.next_int().map(|value| Box::new(value) as _),
                        other => panic!("no such op: {other}"),
                    };
                    match output {
                        Ok(output) => outputs.push(Box::new((
                            output,
                            reader.get_path(),
                            reader.get_previous_path(),
                        ))),
                        Err(error) => {
                            outputs.push(Box::new(json!({
                                "exception": error.class_name(),
                                "message": error.get_message(),
                            })));
                            break;
                        }
                    }
                }
                outputs
            },
        );
        golden.finish();
    }
}
