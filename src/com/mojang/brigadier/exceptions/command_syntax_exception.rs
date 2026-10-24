//! com/mojang/brigadier/exceptions/CommandSyntaxException.java (Brigadier 1.3.11)
//!
//! Not ported: the exception's type (`CommandExceptionType`, which only says which kind made it and
//! which no ported caller reads), its stack trace, and `BUILT_IN_EXCEPTIONS` as a field: the game
//! sets it to its own `BrigadierExceptions` at startup (`SharedConstants`), which the port's reader
//! calls directly.

use std::fmt;
use std::sync::Arc;

use crate::com::mojang::brigadier::message::Message;

/// A command's input that doesn't parse: the message, and the input with where in it the error
/// is, when it's known.
#[derive(Clone, Debug)]
pub struct CommandSyntaxException {
    message: Arc<dyn Message>,
    input: Option<String>,
    cursor: i32,
}

impl CommandSyntaxException {
    /// How many of the input's characters before the cursor the context shows.
    pub const CONTEXT_AMOUNT: i32 = 10;

    pub fn new(message: Arc<dyn Message>) -> CommandSyntaxException {
        CommandSyntaxException {
            message,
            input: None,
            cursor: -1,
        }
    }

    pub fn with_context(message: Arc<dyn Message>, input: &str, cursor: i32) -> CommandSyntaxException {
        CommandSyntaxException {
            message,
            input: Some(input.to_owned()),
            cursor,
        }
    }

    /// The message's text, then where the input went wrong:
    /// `… at position 4: @e[x<--[HERE]`.
    pub fn get_message(&self) -> String {
        let message = self.message.get_string();
        match self.get_context() {
            Some(context) => format!("{message} at position {}: {context}", self.cursor),
            None => message,
        }
    }

    pub fn get_raw_message(&self) -> &Arc<dyn Message> {
        &self.message
    }

    /// The (at most) ten UTF-16 units of the input before the cursor, `...` before them when
    /// there are more, and `<--[HERE]`.
    pub fn get_context(&self) -> Option<String> {
        let input = self.input.as_ref()?;
        if self.cursor < 0 {
            return None;
        }
        let units: Vec<u16> = input.encode_utf16().collect();
        let cursor = (self.cursor as usize).min(units.len());
        let mut builder = String::new();
        if cursor > CommandSyntaxException::CONTEXT_AMOUNT as usize {
            builder.push_str("...");
        }
        let start = cursor.saturating_sub(CommandSyntaxException::CONTEXT_AMOUNT as usize);
        builder.push_str(&String::from_utf16_lossy(&units[start..cursor]));
        builder.push_str("<--[HERE]");
        Some(builder)
    }

    pub fn get_input(&self) -> Option<&str> {
        self.input.as_deref()
    }

    pub fn get_cursor(&self) -> i32 {
        self.cursor
    }
}

/// `toString`: the class's name and the message.
impl fmt::Display for CommandSyntaxException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "com.mojang.brigadier.exceptions.CommandSyntaxException: {}",
            self.get_message()
        )
    }
}
