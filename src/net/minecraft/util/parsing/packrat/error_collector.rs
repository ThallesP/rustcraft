//! net/minecraft/util/parsing/packrat/ErrorCollector.java
//!
//! Where rules store the errors they hit. `LongestOnly` keeps the ones stored at the furthest
//! cursor any error was, in the order they came, the first of which is what a failed parse
//! throws (`Grammar.parseForCommands`); `Nop` drops them, for what a lookahead tries.
//!
//! Not ported: suggestions (`SuggestionSupplier`), which every store passes along for the command
//! UI's completions, so `store(int, SuggestionSupplier, Object)` is `store` without them, and the
//! collector isn't generic in the input they'd read. A reason is Java's `Object`; every one ported
//! is a `DelayedException`.

use crate::net::minecraft::util::parsing::packrat::delayed_exception::DelayedException;
use crate::net::minecraft::util::parsing::packrat::error_entry::ErrorEntry;

pub trait ErrorCollector {
    fn store(&mut self, cursor: i32, reason: &DelayedException);

    fn finish(&mut self, final_cursor: i32);
}

/// Keeps the errors at the furthest cursor.
#[derive(Debug)]
pub struct LongestOnly {
    // Java reuses its entries' objects; the ones past `next_error_entry` are stale.
    entries: Vec<DelayedException>,
    next_error_entry: usize,
    last_cursor: i32,
}

impl LongestOnly {
    pub fn new() -> LongestOnly {
        LongestOnly {
            entries: Vec::new(),
            next_error_entry: 0,
            last_cursor: -1,
        }
    }

    fn discard_errors_from_shorter_parse(&mut self, cursor: i32) {
        if cursor > self.last_cursor {
            self.last_cursor = cursor;
            self.next_error_entry = 0;
        }
    }

    fn add_error_entry(&mut self, reason: &DelayedException) {
        let entry_index = self.next_error_entry;
        self.next_error_entry += 1;
        if entry_index < self.entries.len() {
            self.entries[entry_index] = reason.clone();
        } else {
            self.entries.push(reason.clone());
        }
    }

    pub fn entries(&self) -> Vec<ErrorEntry> {
        self.entries[..self.next_error_entry]
            .iter()
            .map(|reason| ErrorEntry::new(self.last_cursor, reason.clone()))
            .collect()
    }

    pub fn cursor(&self) -> i32 {
        self.last_cursor
    }
}

impl Default for LongestOnly {
    fn default() -> Self {
        LongestOnly::new()
    }
}

impl ErrorCollector for LongestOnly {
    fn store(&mut self, cursor: i32, reason: &DelayedException) {
        self.discard_errors_from_shorter_parse(cursor);
        if cursor == self.last_cursor {
            self.add_error_entry(reason);
        }
    }

    fn finish(&mut self, final_cursor: i32) {
        self.discard_errors_from_shorter_parse(final_cursor);
    }
}

/// Drops every error.
#[derive(Debug, Default)]
pub struct Nop;

impl ErrorCollector for Nop {
    fn store(&mut self, _cursor: i32, _reason: &DelayedException) {}

    fn finish(&mut self, _final_cursor: i32) {}
}
