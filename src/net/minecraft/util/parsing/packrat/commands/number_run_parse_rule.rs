//! net/minecraft/util/parsing/packrat/commands/NumberRunParseRule.java
//!
//! After any whitespace, the longest run of characters its predicate accepts (Java's subclasses'
//! `isAccepted`: digits and underscores), which mustn't start or end with an underscore. No run,
//! or an underscore at either end, is an error stored after the whitespace, which stays skipped
//! (the term holding the rule moves the input back). The run is Java's string, as its UTF-16
//! units.

use crate::com::mojang::brigadier::string_reader::StringReader;
use crate::net::minecraft::util::parsing::packrat::delayed_exception::DelayedException;
use crate::net::minecraft::util::parsing::packrat::parse_state::ParseState;
use crate::net::minecraft::util::parsing::packrat::rule::Rule;

const UNDERSCORE: u16 = b'_' as u16;

pub struct NumberRunParseRule {
    no_value_error: DelayedException,
    underscore_not_allowed_error: DelayedException,
    is_accepted: Box<dyn Fn(u16) -> bool + Send + Sync>,
}

impl NumberRunParseRule {
    pub fn new(
        no_value_error: DelayedException,
        underscore_not_allowed_error: DelayedException,
        is_accepted: impl Fn(u16) -> bool + Send + Sync + 'static,
    ) -> NumberRunParseRule {
        NumberRunParseRule {
            no_value_error,
            underscore_not_allowed_error,
            is_accepted: Box::new(is_accepted),
        }
    }
}

impl Rule<StringReader, Vec<u16>> for NumberRunParseRule {
    fn parse(&self, state: &mut dyn ParseState<StringReader>) -> Option<Vec<u16>> {
        let input = state.input();
        input.skip_whitespace();
        let length = input.get_total_length();
        let start = input.get_cursor();
        let mut pos = start;
        while pos < length && (self.is_accepted)(input.char_at(pos)) {
            pos += 1;
        }
        if pos == start {
            let mark = state.mark();
            state.error_collector().store(mark, &self.no_value_error);
            None
        } else if input.char_at(start) != UNDERSCORE && input.char_at(pos - 1) != UNDERSCORE {
            let run = (start..pos).map(|i| input.char_at(i)).collect();
            input.set_cursor(pos);
            Some(run)
        } else {
            let mark = state.mark();
            state
                .error_collector()
                .store(mark, &self.underscore_not_allowed_error);
            None
        }
    }
}
