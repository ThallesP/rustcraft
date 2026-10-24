//! net/minecraft/util/parsing/packrat/DelayedException.java
//!
//! An error a rule stores as it parses, which becomes an exception only if the parse fails with
//! it, from the input and where in it the error is. Java's is generic in the exception; every one
//! ported makes a `CommandSyntaxException`, so this is a shared function making one.

use std::fmt;
use std::sync::Arc;

use crate::com::mojang::brigadier::exceptions::command_syntax_exception::CommandSyntaxException;
use crate::com::mojang::brigadier::exceptions::dynamic_command_exception_type::DynamicCommandExceptionType;
use crate::com::mojang::brigadier::exceptions::simple_command_exception_type::SimpleCommandExceptionType;
use crate::net::minecraft::util::parsing::packrat::commands::string_reader_terms;

/// Java's `create(String, int)`.
type Create = dyn Fn(&str, i32) -> CommandSyntaxException + Send + Sync;

/// Makes the exception from the whole input and the error's position in it.
#[derive(Clone)]
pub struct DelayedException(Arc<Create>);

impl DelayedException {
    /// Java's lambda implementing the interface.
    pub fn new(
        create: impl Fn(&str, i32) -> CommandSyntaxException + Send + Sync + 'static,
    ) -> DelayedException {
        DelayedException(Arc::new(create))
    }

    pub fn create(&self, contents: &str, position: i32) -> CommandSyntaxException {
        (self.0)(contents, position)
    }
}

impl fmt::Debug for DelayedException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DelayedException")
    }
}

/// `create(SimpleCommandExceptionType)`: the type's exception, with the input as its context.
pub fn create(r#type: SimpleCommandExceptionType) -> DelayedException {
    DelayedException::new(move |contents, position| {
        r#type.create_with_context(&string_reader_terms::create_reader(contents, position))
    })
}

/// `create(DynamicCommandExceptionType, String)`: the type's exception of the argument, with the
/// input as its context.
pub fn create_with_argument(
    r#type: &'static DynamicCommandExceptionType<String>,
    argument: String,
) -> DelayedException {
    DelayedException::new(move |contents, position| {
        r#type.create_with_context(
            &string_reader_terms::create_reader(contents, position),
            argument.clone(),
        )
    })
}
