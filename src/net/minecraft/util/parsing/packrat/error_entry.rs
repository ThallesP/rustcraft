//! net/minecraft/util/parsing/packrat/ErrorEntry.java
//!
//! Not ported: the entry's suggestions (`SuggestionSupplier`), which only the command UI's
//! completions read (see `ErrorCollector`).

use crate::net::minecraft::util::parsing::packrat::delayed_exception::DelayedException;

/// An error a parse stored, and where.
#[derive(Clone, Debug)]
pub struct ErrorEntry {
    cursor: i32,
    reason: DelayedException,
}

impl ErrorEntry {
    pub fn new(cursor: i32, reason: DelayedException) -> ErrorEntry {
        ErrorEntry { cursor, reason }
    }

    pub fn cursor(&self) -> i32 {
        self.cursor
    }

    pub fn reason(&self) -> &DelayedException {
        &self.reason
    }
}
