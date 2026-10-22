//! net/minecraft/world/level/chunk/MissingPaletteEntryException.java
//!
//! A `RuntimeException`, which is a panic here.

/// What a palette throws for an index it has no value for.
pub struct MissingPaletteEntryException;

impl MissingPaletteEntryException {
    /// Panics with the exception's message.
    pub fn throw(index: i32) -> ! {
        panic!("Missing Palette entry for index {index}.")
    }
}
