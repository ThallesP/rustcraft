//! net/minecraft/util/StrictJsonParser.java
//!
//! How the game reads block states and item definitions: Gson's `JsonParser.parseReader` over a
//! reader set to `Strictness.STRICT`, so only RFC 8259's JSON, then nothing after the value. An
//! empty document is JSON null, like `Streams.parse` makes it. The `Err` is what Java throws, a
//! `JsonSyntaxException` wrapping the reader's exception.

use crate::com::google::gson::json_element::JsonElement;
use crate::com::google::gson::json_parser::JsonParser;
use crate::com::google::gson::json_syntax_exception::JsonSyntaxException;
use crate::com::google::gson::stream::json_reader::JsonReader;
use crate::com::google::gson::stream::json_token::JsonToken;
use crate::com::google::gson::stream::malformed_json_exception::MalformedJsonException;
use crate::com::google::gson::strictness::Strictness;
use crate::java::lang::number_format_exception::NumberFormatException;
use crate::java::lang::throwable::Throwable;

pub struct StrictJsonParser;

impl StrictJsonParser {
    /// `parse(String)`, which reads a `StringReader` of it.
    pub fn parse(json: &str) -> Result<JsonElement, Throwable> {
        let mut json_reader = JsonReader::new(json);
        json_reader.set_strictness(Strictness::STRICT);
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
}
