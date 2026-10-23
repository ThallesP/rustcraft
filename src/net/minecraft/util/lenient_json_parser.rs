//! net/minecraft/util/LenientJsonParser.java
//!
//! How the game reads JSON from the network (`ByteBufCodecs.lenientJson`): Gson's
//! `JsonParser.parseString`, lenient, then nothing after the value. `parse(Reader)` is the same
//! over a reader, which the port reads into a string first.

use crate::com::google::gson::json_element::JsonElement;
use crate::com::google::gson::json_parser::JsonParser;
use crate::java::lang::throwable::Throwable;

pub struct LenientJsonParser;

impl LenientJsonParser {
    /// `parse(String)`. The `Err` is a `JsonSyntaxException`.
    pub fn parse(json: &str) -> Result<JsonElement, Throwable> {
        JsonParser::parse_string(json)
    }
}
