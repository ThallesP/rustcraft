//! com/mojang/brigadier/exceptions/DynamicCommandExceptionType.java (Brigadier 1.3.11)
//!
//! Java's function takes an `Object`; here the argument's type is the exception type's parameter.

use std::sync::Arc;

use crate::com::mojang::brigadier::exceptions::command_syntax_exception::CommandSyntaxException;
use crate::com::mojang::brigadier::message::Message;
use crate::com::mojang::brigadier::string_reader::StringReader;

/// An error whose message is made from an argument.
pub struct DynamicCommandExceptionType<A> {
    function: fn(A) -> Arc<dyn Message>,
}

impl<A> DynamicCommandExceptionType<A> {
    pub const fn new(function: fn(A) -> Arc<dyn Message>) -> DynamicCommandExceptionType<A> {
        DynamicCommandExceptionType { function }
    }

    pub fn create(&self, arg: A) -> CommandSyntaxException {
        CommandSyntaxException::new((self.function)(arg))
    }

    pub fn create_with_context(&self, reader: &StringReader, arg: A) -> CommandSyntaxException {
        CommandSyntaxException::with_context(
            (self.function)(arg),
            reader.get_string(),
            reader.get_cursor(),
        )
    }
}
