//! net/minecraft/util/parsing/packrat/commands/StringReaderTerms.java
//!
//! Terms over Brigadier's reader: a character of a set, after any whitespace. A character that
//! isn't one is read all the same (the term that holds this one moves the input back), and the
//! error is Brigadier's `literalIncorrect` of the set (`Expected literal x|X`), stored where the
//! character is.
//!
//! Not ported: `word` (`TerminalWord`), which no grammar ported uses, and the suggestions each
//! term stores with its error (see `ErrorCollector`). `TerminalCharacters`' subclasses, Java's
//! anonymous classes overriding `isAccepted`, are its predicate.

use crate::com::mojang::brigadier::string_reader::StringReader;
use crate::net::minecraft::commands::brigadier_exceptions::BrigadierExceptions;
use crate::net::minecraft::util::parsing::packrat::control::Control;
use crate::net::minecraft::util::parsing::packrat::delayed_exception::{self, DelayedException};
use crate::net::minecraft::util::parsing::packrat::parse_state::ParseState;
use crate::net::minecraft::util::parsing::packrat::term::Term;

/// Java's `char` of a Rust one, which must be in the Basic Multilingual Plane.
fn unit(value: char) -> u16 {
    u16::try_from(u32::from(value)).expect("a char outside the Basic Multilingual Plane")
}

/// `character(char)`.
pub fn character(value: char) -> Box<dyn Term<StringReader>> {
    let value = unit(value);
    Box::new(TerminalCharacters::new(&[value], move |v| value == v))
}

/// `characters(char, char)`: either.
pub fn characters(v1: char, v2: char) -> Box<dyn Term<StringReader>> {
    let (v1, v2) = (unit(v1), unit(v2));
    Box::new(TerminalCharacters::new(&[v1, v2], move |v| {
        v == v1 || v == v2
    }))
}

/// A reader of the contents at the cursor, which a `DelayedException` takes its context from.
pub fn create_reader(contents: &str, cursor: i32) -> StringReader {
    let mut reader = StringReader::new(contents);
    reader.set_cursor(cursor);
    reader
}

/// A character its predicate accepts, after whitespace.
pub struct TerminalCharacters {
    error: DelayedException,
    is_accepted: Box<dyn Fn(u16) -> bool + Send + Sync>,
}

impl TerminalCharacters {
    /// The characters name the set in the error: Java's `values`, which only the error and the
    /// suggestions read.
    pub fn new(
        values: &[u16],
        is_accepted: impl Fn(u16) -> bool + Send + Sync + 'static,
    ) -> TerminalCharacters {
        let joined_values: Vec<String> = values
            .iter()
            .map(|&value| String::from_utf16_lossy(&[value]))
            .collect();
        TerminalCharacters {
            error: delayed_exception::create_with_argument(
                BrigadierExceptions::literal_incorrect(),
                joined_values.join("|"),
            ),
            is_accepted: Box::new(is_accepted),
        }
    }
}

impl Term<StringReader> for TerminalCharacters {
    fn parse(&self, state: &mut dyn ParseState<StringReader>, _control: &Control) -> bool {
        state.input().skip_whitespace();
        let cursor = state.mark();
        let input = state.input();
        if input.can_read() && (self.is_accepted)(input.read()) {
            return true;
        }
        state.error_collector().store(cursor, &self.error);
        false
    }
}
