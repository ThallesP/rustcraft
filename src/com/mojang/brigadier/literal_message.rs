//! com/mojang/brigadier/LiteralMessage.java (Brigadier 1.3.11)

use std::fmt;

use crate::com::mojang::brigadier::message::Message;

/// A message that's its string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiteralMessage {
    string: String,
}

impl LiteralMessage {
    pub fn new(string: impl Into<String>) -> LiteralMessage {
        LiteralMessage {
            string: string.into(),
        }
    }
}

impl Message for LiteralMessage {
    fn get_string(&self) -> String {
        self.string.clone()
    }
}

impl fmt::Display for LiteralMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.string)
    }
}
