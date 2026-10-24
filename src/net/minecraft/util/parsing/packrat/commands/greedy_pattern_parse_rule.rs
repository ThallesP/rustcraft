//! net/minecraft/util/parsing/packrat/commands/GreedyPatternParseRule.java
//!
//! What its pattern matches at the cursor (`Matcher.lookingAt`), with no whitespace skipped;
//! nothing is the error, stored at the cursor. Java takes any `Pattern`; the one grammar ported
//! (`SnbtGrammar`'s character names, `[-a-zA-Z0-9 ]+`) only needs a run of one or more
//! characters of a class, so that's what the pattern is here: its class, as a predicate.

use crate::com::mojang::brigadier::string_reader::StringReader;
use crate::net::minecraft::util::parsing::packrat::delayed_exception::DelayedException;
use crate::net::minecraft::util::parsing::packrat::parse_state::ParseState;
use crate::net::minecraft::util::parsing::packrat::rule::Rule;

pub struct GreedyPatternParseRule {
    pattern: Box<dyn Fn(u16) -> bool + Send + Sync>,
    error: DelayedException,
}

impl GreedyPatternParseRule {
    /// The pattern `[class]+`, as the class's predicate.
    pub fn new(
        pattern: impl Fn(u16) -> bool + Send + Sync + 'static,
        error: DelayedException,
    ) -> GreedyPatternParseRule {
        GreedyPatternParseRule {
            pattern: Box::new(pattern),
            error,
        }
    }
}

impl Rule<StringReader, Vec<u16>> for GreedyPatternParseRule {
    fn parse(&self, state: &mut dyn ParseState<StringReader>) -> Option<Vec<u16>> {
        let input = state.input();
        let start = input.get_cursor();
        let mut end = start;
        while end < input.get_total_length() && (self.pattern)(input.char_at(end)) {
            end += 1;
        }
        if end == start {
            let mark = state.mark();
            state.error_collector().store(mark, &self.error);
            return None;
        }
        let group = (start..end).map(|i| input.char_at(i)).collect();
        input.set_cursor(end);
        Some(group)
    }
}
