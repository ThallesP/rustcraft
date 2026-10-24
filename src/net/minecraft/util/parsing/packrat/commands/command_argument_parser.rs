//! net/minecraft/util/parsing/packrat/commands/CommandArgumentParser.java
//!
//! Not ported: `parseForSuggestions`, which only the command UI's completions call, and
//! `mapResult` and `withCodec`, which come with the command arguments using them.

use crate::com::mojang::brigadier::exceptions::command_syntax_exception::CommandSyntaxException;
use crate::com::mojang::brigadier::string_reader::StringReader;

/// Parses a command's argument from the reader.
pub trait CommandArgumentParser<T> {
    fn parse_for_commands(&self, reader: &mut StringReader) -> Result<T, CommandSyntaxException>;
}
