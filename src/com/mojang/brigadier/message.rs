//! com/mojang/brigadier/Message.java (Brigadier 1.3.11)
//!
//! What a command error says: a `LiteralMessage`, or a text component in the game's own errors
//! (Java's `Component` extends `Message`).

use std::fmt;

/// A message, as its text.
pub trait Message: fmt::Debug + Send + Sync {
    fn get_string(&self) -> String;
}
