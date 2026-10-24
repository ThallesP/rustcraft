//! com/mojang/brigadier/StringReader.java (Brigadier 1.3.11), with `ImmutableStringReader`'s
//! methods, its only implementation here.
//!
//! A cursor over Java's string: positions and lengths count UTF-16 units (PORTING.md, "Strings"),
//! and `peek` and `read` hand out units. A copy shares the string, like Java's. Its errors are the
//! game's (`BrigadierExceptions`, which Java's `CommandSyntaxException.BUILT_IN_EXCEPTIONS` is in
//! the game).

use std::sync::Arc;

use crate::com::mojang::brigadier::exceptions::command_syntax_exception::CommandSyntaxException;
use crate::java::lang::character::Character;
use crate::java::lang::double::Double;
use crate::java::lang::float::Float;
use crate::java::lang::integer::Integer;
use crate::java::lang::long::Long;
use crate::net::minecraft::commands::brigadier_exceptions::BrigadierExceptions;

const SYNTAX_ESCAPE: u16 = b'\\' as u16;
const SYNTAX_DOUBLE_QUOTE: u16 = b'"' as u16;
const SYNTAX_SINGLE_QUOTE: u16 = b'\'' as u16;

/// A string being read, and where in it.
#[derive(Clone, Debug)]
pub struct StringReader {
    string: Arc<str>,
    units: Arc<[u16]>,
    cursor: i32,
}

impl StringReader {
    pub fn new(string: &str) -> StringReader {
        StringReader {
            string: Arc::from(string),
            units: string.encode_utf16().collect(),
            cursor: 0,
        }
    }

    pub fn get_string(&self) -> &str {
        &self.string
    }

    pub fn set_cursor(&mut self, cursor: i32) {
        self.cursor = cursor;
    }

    pub fn get_remaining_length(&self) -> i32 {
        self.get_total_length() - self.cursor
    }

    pub fn get_total_length(&self) -> i32 {
        self.units.len() as i32
    }

    pub fn get_cursor(&self) -> i32 {
        self.cursor
    }

    pub fn get_read(&self) -> String {
        self.substring(0, self.cursor)
    }

    pub fn get_remaining(&self) -> String {
        self.substring(self.cursor, self.get_total_length())
    }

    pub fn can_read_length(&self, length: i32) -> bool {
        self.cursor + length <= self.get_total_length()
    }

    pub fn can_read(&self) -> bool {
        self.can_read_length(1)
    }

    pub fn peek(&self) -> u16 {
        self.char_at(self.cursor)
    }

    pub fn peek_offset(&self, offset: i32) -> u16 {
        self.char_at(self.cursor + offset)
    }

    pub fn read(&mut self) -> u16 {
        let c = self.char_at(self.cursor);
        self.cursor += 1;
        c
    }

    pub fn skip(&mut self) {
        self.cursor += 1;
    }

    pub fn is_allowed_number(c: u16) -> bool {
        (u16::from(b'0')..=u16::from(b'9')).contains(&c)
            || c == u16::from(b'.')
            || c == u16::from(b'-')
    }

    pub fn is_quoted_string_start(c: u16) -> bool {
        c == SYNTAX_DOUBLE_QUOTE || c == SYNTAX_SINGLE_QUOTE
    }

    pub fn skip_whitespace(&mut self) {
        while self.can_read() && Character::is_whitespace(self.peek()) {
            self.skip();
        }
    }

    // The run of number characters from the cursor, which the cursor moves past.
    fn read_number(&mut self) -> (i32, String) {
        let start = self.cursor;
        while self.can_read() && StringReader::is_allowed_number(self.peek()) {
            self.skip();
        }
        (start, self.substring(start, self.cursor))
    }

    pub fn read_int(&mut self) -> Result<i32, CommandSyntaxException> {
        let (start, number) = self.read_number();
        if number.is_empty() {
            return Err(BrigadierExceptions::reader_expected_int().create_with_context(self));
        }
        Integer::parse_int(&number).map_err(|_| {
            self.cursor = start;
            BrigadierExceptions::reader_invalid_int().create_with_context(self, number)
        })
    }

    pub fn read_long(&mut self) -> Result<i64, CommandSyntaxException> {
        let (start, number) = self.read_number();
        if number.is_empty() {
            return Err(BrigadierExceptions::reader_expected_long().create_with_context(self));
        }
        Long::parse_long(&number).map_err(|_| {
            self.cursor = start;
            BrigadierExceptions::reader_invalid_long().create_with_context(self, number)
        })
    }

    pub fn read_double(&mut self) -> Result<f64, CommandSyntaxException> {
        let (start, number) = self.read_number();
        if number.is_empty() {
            return Err(BrigadierExceptions::reader_expected_double().create_with_context(self));
        }
        Double::parse_double(&number).map_err(|_| {
            self.cursor = start;
            BrigadierExceptions::reader_invalid_double().create_with_context(self, number)
        })
    }

    pub fn read_float(&mut self) -> Result<f32, CommandSyntaxException> {
        let (start, number) = self.read_number();
        if number.is_empty() {
            return Err(BrigadierExceptions::reader_expected_float().create_with_context(self));
        }
        Float::parse_float(&number).map_err(|_| {
            self.cursor = start;
            BrigadierExceptions::reader_invalid_float().create_with_context(self, number)
        })
    }

    pub fn is_allowed_in_unquoted_string(c: u16) -> bool {
        (u16::from(b'0')..=u16::from(b'9')).contains(&c)
            || (u16::from(b'A')..=u16::from(b'Z')).contains(&c)
            || (u16::from(b'a')..=u16::from(b'z')).contains(&c)
            || b"_-.+".iter().any(|&allowed| c == u16::from(allowed))
    }

    pub fn read_unquoted_string(&mut self) -> String {
        let start = self.cursor;
        while self.can_read() && StringReader::is_allowed_in_unquoted_string(self.peek()) {
            self.skip();
        }
        self.substring(start, self.cursor)
    }

    pub fn read_quoted_string(&mut self) -> Result<String, CommandSyntaxException> {
        if !self.can_read() {
            return Ok(String::new());
        }
        let next = self.peek();
        if !StringReader::is_quoted_string_start(next) {
            return Err(
                BrigadierExceptions::reader_expected_start_of_quote().create_with_context(self)
            );
        }
        self.skip();
        self.read_string_until(next)
    }

    pub fn read_string_until(&mut self, terminator: u16) -> Result<String, CommandSyntaxException> {
        let mut result: Vec<u16> = Vec::new();
        let mut escaped = false;
        while self.can_read() {
            let c = self.read();
            if escaped {
                if c != terminator && c != SYNTAX_ESCAPE {
                    self.set_cursor(self.get_cursor() - 1);
                    return Err(BrigadierExceptions::reader_invalid_escape()
                        .create_with_context(self, String::from_utf16_lossy(&[c])));
                }
                result.push(c);
                escaped = false;
            } else if c == SYNTAX_ESCAPE {
                escaped = true;
            } else {
                if c == terminator {
                    return Ok(String::from_utf16_lossy(&result));
                }
                result.push(c);
            }
        }
        Err(BrigadierExceptions::reader_expected_end_of_quote().create_with_context(self))
    }

    pub fn read_string(&mut self) -> Result<String, CommandSyntaxException> {
        if !self.can_read() {
            return Ok(String::new());
        }
        let next = self.peek();
        if StringReader::is_quoted_string_start(next) {
            self.skip();
            self.read_string_until(next)
        } else {
            Ok(self.read_unquoted_string())
        }
    }

    pub fn read_boolean(&mut self) -> Result<bool, CommandSyntaxException> {
        let start = self.cursor;
        let value = self.read_string()?;
        if value.is_empty() {
            return Err(BrigadierExceptions::reader_expected_bool().create_with_context(self));
        }
        match value.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => {
                self.cursor = start;
                Err(BrigadierExceptions::reader_invalid_bool().create_with_context(self, value))
            }
        }
    }

    pub fn expect(&mut self, c: u16) -> Result<(), CommandSyntaxException> {
        if self.can_read() && self.peek() == c {
            self.skip();
            Ok(())
        } else {
            Err(BrigadierExceptions::reader_expected_symbol()
                .create_with_context(self, String::from_utf16_lossy(&[c])))
        }
    }

    /// `string.substring(start, end)`, in UTF-16 units; panics where Java throws
    /// `StringIndexOutOfBoundsException`.
    pub fn substring(&self, start: i32, end: i32) -> String {
        assert!(
            0 <= start && start <= end && end <= self.get_total_length(),
            "StringIndexOutOfBoundsException: begin {start}, end {end}, length {}",
            self.get_total_length()
        );
        String::from_utf16_lossy(&self.units[start as usize..end as usize])
    }

    /// `getString().charAt(index)`, which the parse rules over a reader read the string with;
    /// panics where Java throws `StringIndexOutOfBoundsException`.
    pub fn char_at(&self, index: i32) -> u16 {
        assert!(
            0 <= index && index < self.get_total_length(),
            "StringIndexOutOfBoundsException: Index {index} out of bounds for length {}",
            self.get_total_length()
        );
        self.units[index as usize]
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::StringReader;
    use crate::golden::Golden;

    // One op of a program: its output, or the error it gave.
    fn apply(reader: &mut StringReader, op: &Value) -> Result<Value, String> {
        let (name, argument) = match op {
            Value::Array(op) => (op[0].as_str().unwrap(), op[1].as_i64().unwrap() as i32),
            op => (op.as_str().unwrap(), 0),
        };
        let error = |exception: super::CommandSyntaxException| exception.get_message();
        Ok(match name {
            "readInt" => json!(reader.read_int().map_err(error)?),
            "readLong" => json!(reader.read_long().map_err(error)?),
            "readDouble" => json!(format!(
                "{:016x}",
                reader.read_double().map_err(error)?.to_bits()
            )),
            "readFloat" => json!(format!(
                "{:08x}",
                reader.read_float().map_err(error)?.to_bits()
            )),
            "readUnquotedString" => json!(reader.read_unquoted_string()),
            "readQuotedString" => json!(reader.read_quoted_string().map_err(error)?),
            "readString" => json!(reader.read_string().map_err(error)?),
            "readStringUntil" => json!(reader.read_string_until(argument as u16).map_err(error)?),
            "readBoolean" => json!(reader.read_boolean().map_err(error)?),
            "skipWhitespace" => {
                reader.skip_whitespace();
                Value::Null
            }
            "skip" => {
                reader.skip();
                Value::Null
            }
            "setCursor" => {
                reader.set_cursor(argument);
                Value::Null
            }
            "expect" => {
                reader.expect(argument as u16).map_err(error)?;
                Value::Null
            }
            "canRead" if op.is_array() => json!(reader.can_read_length(argument)),
            "canRead" => json!(reader.can_read()),
            "peek" if op.is_array() => json!(reader.peek_offset(argument)),
            "peek" => json!(reader.peek()),
            "read" => json!(reader.read()),
            "getRead" => json!(reader.get_read()),
            "getRemaining" => json!(reader.get_remaining()),
            "getRemainingLength" => json!(reader.get_remaining_length()),
            "getTotalLength" => json!(reader.get_total_length()),
            _ => panic!("unknown op {name}"),
        })
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("com.mojang.brigadier.StringReader");
        golden.check("programs", |input: String, ops: Vec<Value>| {
            let mut reader = StringReader::new(&input);
            let mut outputs = Vec::new();
            for op in &ops {
                match apply(&mut reader, op) {
                    Ok(output) => outputs.push(json!([output, reader.get_cursor()])),
                    Err(message) => {
                        outputs.push(json!(["exception", message, reader.get_cursor()]));
                        break;
                    }
                }
            }
            Value::Array(outputs)
        });
        golden.finish();
    }
}
