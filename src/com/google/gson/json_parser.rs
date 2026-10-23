//! com/google/gson/JsonParser.java, with com/google/gson/stream/JsonReader.java (Gson 2.14.0)
//!
//! `JsonParser.parseReader` over a `JsonReader` set to `Strictness.STRICT`, the way the game
//! reads models (`GsonHelper.fromJson`) and block states (`StrictJsonParser`): RFC 8259's JSON
//! and nothing else, the whole document one value. Written from Gson's behaviour: its sources
//! aren't among the game's. Gson's lenient extensions (comments, unquoted names, single quotes)
//! aren't ported; nothing the game parses strictly can have them. The `Err` is Java's
//! `JsonSyntaxException`, with the reason.

use crate::com::google::gson::internal::lazily_parsed_number::LazilyParsedNumber;
use crate::com::google::gson::json_element::{JsonElement, JsonObject, JsonPrimitive};

pub struct JsonParser;

/// Gson's default nesting limit, `JsonReader.setNestingLimit`'s 255.
const NESTING_LIMIT: usize = 255;

struct Reader<'a> {
    source: &'a str,
    text: &'a [u8],
    pos: usize,
}

impl JsonParser {
    /// `StrictJsonParser.parse`: one value, and nothing after it but whitespace.
    pub fn parse_strict(text: &str) -> Result<JsonElement, String> {
        let mut reader = Reader {
            source: text,
            text: text.as_bytes(),
            pos: 0,
        };
        // A byte order mark is skipped, like JsonReader's.
        if text.starts_with('\u{feff}') {
            reader.pos = 3;
        }
        let element = reader.value(0)?;
        reader.skip_whitespace();
        if reader.pos != reader.text.len() {
            return Err(reader.error("Did not consume the entire document."));
        }
        Ok(element)
    }
}

impl Reader<'_> {
    fn error(&self, message: &str) -> String {
        format!("{message} at offset {}", self.pos)
    }

    fn skip_whitespace(&mut self) {
        while let Some(b' ' | b'\t' | b'\n' | b'\r') = self.text.get(self.pos) {
            self.pos += 1;
        }
    }

    fn peek(&mut self) -> Option<u8> {
        self.skip_whitespace();
        self.text.get(self.pos).copied()
    }

    fn expect(&mut self, byte: u8) -> Result<(), String> {
        if self.peek() == Some(byte) {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.error(&format!("Expected '{}'", byte as char)))
        }
    }

    fn value(&mut self, depth: usize) -> Result<JsonElement, String> {
        match self.peek() {
            Some(b'{') => {
                if depth >= NESTING_LIMIT {
                    return Err(self.error("Nesting limit 255 reached"));
                }
                self.pos += 1;
                let mut object = JsonObject::new();
                if self.peek() == Some(b'}') {
                    self.pos += 1;
                    return Ok(JsonElement::Object(object));
                }
                loop {
                    if self.peek() != Some(b'"') {
                        return Err(self.error("Expected name"));
                    }
                    let name = self.string()?;
                    self.expect(b':')?;
                    let value = self.value(depth + 1)?;
                    object.add(name, value);
                    match self.peek() {
                        Some(b',') => self.pos += 1,
                        Some(b'}') => {
                            self.pos += 1;
                            return Ok(JsonElement::Object(object));
                        }
                        _ => return Err(self.error("Unterminated object")),
                    }
                }
            }
            Some(b'[') => {
                if depth >= NESTING_LIMIT {
                    return Err(self.error("Nesting limit 255 reached"));
                }
                self.pos += 1;
                let mut array = Vec::new();
                if self.peek() == Some(b']') {
                    self.pos += 1;
                    return Ok(JsonElement::Array(array));
                }
                loop {
                    array.push(self.value(depth + 1)?);
                    match self.peek() {
                        Some(b',') => self.pos += 1,
                        Some(b']') => {
                            self.pos += 1;
                            return Ok(JsonElement::Array(array));
                        }
                        _ => return Err(self.error("Unterminated array")),
                    }
                }
            }
            Some(b'"') => Ok(JsonElement::Primitive(JsonPrimitive::String(
                self.string()?,
            ))),
            Some(b't') => {
                self.literal("true", JsonElement::Primitive(JsonPrimitive::Boolean(true)))
            }
            Some(b'f') => self.literal(
                "false",
                JsonElement::Primitive(JsonPrimitive::Boolean(false)),
            ),
            Some(b'n') => self.literal("null", JsonElement::Null),
            Some(b'-' | b'0'..=b'9') => self.number(),
            Some(_) => Err(self.error("Expected value")),
            None => Err(self.error("End of input")),
        }
    }

    fn literal(&mut self, word: &str, element: JsonElement) -> Result<JsonElement, String> {
        if self.text[self.pos..].starts_with(word.as_bytes()) {
            self.pos += word.len();
            Ok(element)
        } else {
            Err(self.error("Expected value"))
        }
    }

    // `-? (0 | [1-9][0-9]*) (. [0-9]+)? ([eE] [+-]? [0-9]+)?`, kept as its text.
    fn number(&mut self) -> Result<JsonElement, String> {
        let start = self.pos;
        let digits = |reader: &mut Reader| {
            let from = reader.pos;
            while let Some(b'0'..=b'9') = reader.text.get(reader.pos) {
                reader.pos += 1;
            }
            reader.pos - from
        };
        if self.text.get(self.pos) == Some(&b'-') {
            self.pos += 1;
        }
        let integer_start = self.pos;
        let integer_digits = digits(self);
        if integer_digits == 0 || (integer_digits > 1 && self.text[integer_start] == b'0') {
            return Err(self.error("Malformed number"));
        }
        if self.text.get(self.pos) == Some(&b'.') {
            self.pos += 1;
            if digits(self) == 0 {
                return Err(self.error("Malformed number"));
            }
        }
        if let Some(b'e' | b'E') = self.text.get(self.pos) {
            self.pos += 1;
            if let Some(b'+' | b'-') = self.text.get(self.pos) {
                self.pos += 1;
            }
            if digits(self) == 0 {
                return Err(self.error("Malformed number"));
            }
        }
        let text = std::str::from_utf8(&self.text[start..self.pos]).expect("ASCII");
        Ok(JsonElement::Primitive(JsonPrimitive::Number(
            LazilyParsedNumber::new(text.to_owned()),
        )))
    }

    fn string(&mut self) -> Result<String, String> {
        self.pos += 1;
        let mut units: Vec<u16> = Vec::new();
        loop {
            let Some(&byte) = self.text.get(self.pos) else {
                return Err(self.error("Unterminated string"));
            };
            match byte {
                b'"' => {
                    self.pos += 1;
                    return Ok(String::from_utf16_lossy(&units));
                }
                b'\\' => {
                    self.pos += 1;
                    let escaped = *self
                        .text
                        .get(self.pos)
                        .ok_or_else(|| self.error("Unterminated escape sequence"))?;
                    self.pos += 1;
                    let unit = match escaped {
                        b'"' => b'"' as u16,
                        b'\\' => b'\\' as u16,
                        b'/' => b'/' as u16,
                        b'b' => 0x08,
                        b'f' => 0x0c,
                        b'n' => b'\n' as u16,
                        b'r' => b'\r' as u16,
                        b't' => b'\t' as u16,
                        b'u' => {
                            let hex = self
                                .text
                                .get(self.pos..self.pos + 4)
                                .and_then(|hex| std::str::from_utf8(hex).ok())
                                .and_then(|hex| u16::from_str_radix(hex, 16).ok())
                                .ok_or_else(|| self.error("Malformed Unicode escape"))?;
                            self.pos += 4;
                            hex
                        }
                        _ => return Err(self.error("Invalid escape sequence")),
                    };
                    units.push(unit);
                }
                0x00..=0x1f => return Err(self.error("Unescaped control characters")),
                _ => {
                    // One character, as its UTF-16 units. The text is a `str`, and every step
                    // before this one moved over whole characters.
                    let character = self.source[self.pos..].chars().next().expect("a character");
                    let mut buffer = [0u16; 2];
                    units.extend_from_slice(character.encode_utf16(&mut buffer));
                    self.pos += character.len_utf8();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::JsonParser;
    use crate::com::google::gson::json_element::JsonElement;

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
        for text in ["{a: 1}", "[1,]", "01", "'x'", "[1] 2", "NaN", "// x\n1", ""] {
            assert!(JsonParser::parse_strict(text).is_err(), "{text}");
        }
    }
}
