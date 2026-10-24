//! net/minecraft/util/parsing/packrat/commands/GreedyPredicateParseRule.java
//!
//! The longest run, up to the maximum, of characters its predicate accepts (Java's subclasses'
//! `isAccepted`), with no whitespace skipped; fewer than the minimum is the error, stored at the
//! cursor. The run is Java's string, as its UTF-16 units (`SnbtGrammar`'s strings join it with
//! escapes, which can be half of a surrogate pair).

use crate::com::mojang::brigadier::string_reader::StringReader;
use crate::net::minecraft::util::parsing::packrat::delayed_exception::DelayedException;
use crate::net::minecraft::util::parsing::packrat::parse_state::ParseState;
use crate::net::minecraft::util::parsing::packrat::rule::Rule;

pub struct GreedyPredicateParseRule {
    min_size: i32,
    max_size: i32,
    error: DelayedException,
    is_accepted: Box<dyn Fn(u16) -> bool + Send + Sync>,
}

impl GreedyPredicateParseRule {
    /// `GreedyPredicateParseRule(int, DelayedException)`: no maximum.
    pub fn new(
        min_size: i32,
        error: DelayedException,
        is_accepted: impl Fn(u16) -> bool + Send + Sync + 'static,
    ) -> GreedyPredicateParseRule {
        GreedyPredicateParseRule::with_max_size(min_size, i32::MAX, error, is_accepted)
    }

    /// `GreedyPredicateParseRule(int, int, DelayedException)`.
    pub fn with_max_size(
        min_size: i32,
        max_size: i32,
        error: DelayedException,
        is_accepted: impl Fn(u16) -> bool + Send + Sync + 'static,
    ) -> GreedyPredicateParseRule {
        GreedyPredicateParseRule {
            min_size,
            max_size,
            error,
            is_accepted: Box::new(is_accepted),
        }
    }
}

impl Rule<StringReader, Vec<u16>> for GreedyPredicateParseRule {
    fn parse(&self, state: &mut dyn ParseState<StringReader>) -> Option<Vec<u16>> {
        let input = state.input();
        let length = input.get_total_length();
        let start = input.get_cursor();
        let mut pos = start;
        while pos < length && (self.is_accepted)(input.char_at(pos)) && pos - start < self.max_size
        {
            pos += 1;
        }
        if pos - start < self.min_size {
            let mark = state.mark();
            state.error_collector().store(mark, &self.error);
            return None;
        }
        let run = (start..pos).map(|i| input.char_at(i)).collect();
        input.set_cursor(pos);
        Some(run)
    }
}
