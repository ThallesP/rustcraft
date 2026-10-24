//! net/minecraft/util/parsing/packrat/commands/UnquotedStringParseRule.java
//!
//! After any whitespace, Brigadier's unquoted string (letters, digits, `_-.+`); shorter than the
//! minimum is the error, stored after the whitespace. The string is Java's, as its UTF-16 units.

use crate::com::mojang::brigadier::string_reader::StringReader;
use crate::net::minecraft::util::parsing::packrat::delayed_exception::DelayedException;
use crate::net::minecraft::util::parsing::packrat::parse_state::ParseState;
use crate::net::minecraft::util::parsing::packrat::rule::Rule;

pub struct UnquotedStringParseRule {
    min_size: usize,
    error: DelayedException,
}

impl UnquotedStringParseRule {
    pub fn new(min_size: usize, error: DelayedException) -> UnquotedStringParseRule {
        UnquotedStringParseRule { min_size, error }
    }
}

impl Rule<StringReader, Vec<u16>> for UnquotedStringParseRule {
    fn parse(&self, state: &mut dyn ParseState<StringReader>) -> Option<Vec<u16>> {
        state.input().skip_whitespace();
        let cursor = state.mark();
        let value: Vec<u16> = state
            .input()
            .read_unquoted_string()
            .encode_utf16()
            .collect();
        if value.len() < self.min_size {
            state.error_collector().store(cursor, &self.error);
            return None;
        }
        Some(value)
    }
}
