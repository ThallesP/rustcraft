//! com/mojang/brigadier/exceptions/SimpleCommandExceptionType.java (Brigadier 1.3.11)

use std::fmt;
use std::sync::Arc;

use crate::com::mojang::brigadier::exceptions::command_syntax_exception::CommandSyntaxException;
use crate::com::mojang::brigadier::message::Message;
use crate::com::mojang::brigadier::string_reader::StringReader;

/// An error that always says the same.
#[derive(Clone, Debug)]
pub struct SimpleCommandExceptionType {
    message: Arc<dyn Message>,
}

impl SimpleCommandExceptionType {
    pub fn new(message: impl Message + 'static) -> SimpleCommandExceptionType {
        SimpleCommandExceptionType {
            message: Arc::new(message),
        }
    }

    pub fn create(&self) -> CommandSyntaxException {
        CommandSyntaxException::new(Arc::clone(&self.message))
    }

    pub fn create_with_context(&self, reader: &StringReader) -> CommandSyntaxException {
        CommandSyntaxException::with_context(
            Arc::clone(&self.message),
            reader.get_string(),
            reader.get_cursor(),
        )
    }
}

impl fmt::Display for SimpleCommandExceptionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message.get_string())
    }
}
