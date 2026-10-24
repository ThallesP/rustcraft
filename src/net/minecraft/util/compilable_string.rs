//! net/minecraft/util/CompilableString.java
//!
//! A string a text component holds and the command parser's reading of it: a selector, an NBT
//! path, coordinates. The codec decodes the string and fails where it doesn't parse, so a server's
//! component with a bad one fails like Java's. `CommandParserHelper` is `parse_command`, with
//! Java's two abstract methods as its arguments.

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::brigadier::exceptions::command_syntax_exception::CommandSyntaxException;
use crate::com::mojang::brigadier::exceptions::dynamic_command_exception_type::DynamicCommandExceptionType;
use crate::com::mojang::brigadier::string_reader::StringReader;
use crate::com::mojang::serialization::codec::{BoxedCodec, CodecExt, STRING};
use crate::com::mojang::serialization::data_result::DataResult;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::contents::translatable_contents::TranslationArgument;

static TRAILING_DATA: LazyLock<DynamicCommandExceptionType<String>> = LazyLock::new(|| {
    DynamicCommandExceptionType::new(|command_and_remainder| {
        std::sync::Arc::new(Component::from(Component::translatable_with_args(
            "command.trailing_data",
            vec![TranslationArgument::String(command_and_remainder)],
        )))
    })
});

/// A string and what it compiles to; equal when the strings are.
#[derive(Clone, Debug)]
pub struct CompilableString<T> {
    source: String,
    compiled: T,
}

impl<T> CompilableString<T> {
    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn compiled(&self) -> &T {
        &self.compiled
    }
}

impl<T: Clone + Send + Sync + 'static> CompilableString<T> {
    /// `codec(Function<String, DataResult<T>>)`: a string, compiled as it's decoded.
    pub fn codec(compiler: fn(&str) -> DataResult<T>) -> BoxedCodec<CompilableString<T>> {
        STRING
            .comap_flat_map(
                move |source: String| {
                    compiler(&source).map(|compiled| CompilableString {
                        source: source.clone(),
                        compiled,
                    })
                },
                |compilable: &CompilableString<T>| compilable.source.clone(),
            )
            .boxed()
    }
}

/// `CommandParserHelper.apply`: the whole string parsed, or the error `error_message` words; text
/// left after what parses is an error too.
pub fn parse_command<T>(
    contents: &str,
    parse: impl FnOnce(&mut StringReader) -> Result<T, CommandSyntaxException>,
    error_message: impl FnOnce(&str, &CommandSyntaxException) -> String,
) -> DataResult<T> {
    let mut reader = StringReader::new(contents);
    let result = parse(&mut reader).and_then(|result| {
        if reader.can_read() {
            let parsed = reader.substring(0, reader.get_cursor());
            let leftovers = reader.substring(reader.get_cursor(), reader.get_total_length());
            Err(TRAILING_DATA.create(format!("{parsed}[{leftovers}]")))
        } else {
            Ok(result)
        }
    });
    match result {
        Ok(result) => DataResult::success(result),
        Err(exception) => {
            let message = error_message(contents, &exception);
            DataResult::error(|| message)
        }
    }
}

impl<T> PartialEq for CompilableString<T> {
    fn eq(&self, other: &CompilableString<T>) -> bool {
        self.source == other.source
    }
}

/// `toString`: the string.
impl<T> fmt::Display for CompilableString<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.source)
    }
}
