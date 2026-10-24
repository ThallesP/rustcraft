//! com/mojang/brigadier/exceptions/Dynamic2CommandExceptionType.java (Brigadier 1.3.11)
//!
//! Java's function takes two `Object`s; here their types are the exception type's parameters.

use std::sync::Arc;

use crate::com::mojang::brigadier::exceptions::command_syntax_exception::CommandSyntaxException;
use crate::com::mojang::brigadier::message::Message;
use crate::com::mojang::brigadier::string_reader::StringReader;

/// An error whose message is made from two arguments.
pub struct Dynamic2CommandExceptionType<A, B> {
    function: fn(A, B) -> Arc<dyn Message>,
}

impl<A, B> Dynamic2CommandExceptionType<A, B> {
    pub const fn new(function: fn(A, B) -> Arc<dyn Message>) -> Dynamic2CommandExceptionType<A, B> {
        Dynamic2CommandExceptionType { function }
    }

    pub fn create(&self, a: A, b: B) -> CommandSyntaxException {
        CommandSyntaxException::new((self.function)(a, b))
    }

    pub fn create_with_context(&self, reader: &StringReader, a: A, b: B) -> CommandSyntaxException {
        CommandSyntaxException::with_context(
            (self.function)(a, b),
            reader.get_string(),
            reader.get_cursor(),
        )
    }
}
