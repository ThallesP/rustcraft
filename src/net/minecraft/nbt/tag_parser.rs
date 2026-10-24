//! net/minecraft/nbt/TagParser.java
//!
//! SNBT's parser (`SnbtGrammar`) for an ops, with the readers commands call: `parseFully` (the
//! whole input, whitespace after it allowed), `parseAsArgument` (as much as parses, the reader's
//! cursor after it) and the compound ones over `NbtOps`. A parse that fails throws the grammar's
//! error, leaving the cursor wherever the parse left it; trailing data throws where it starts,
//! after the whitespace, and a value that isn't a compound where the value ends.
//!
//! Java's parser is generic in the ops' value, and so is this one, over an ops that lives as long
//! as the parser (`&'static`, like `NbtOps::INSTANCE`): the grammar's actions hold it. A parser
//! over registry ops (`ItemParser`'s) settles how long those live when it comes.
//!
//! `FLATTENED_CODEC` is a compound as its SNBT, and `LENIENT_CODEC` that or the compound itself
//! (`CompoundTag::CODEC`). `\N{name}` escapes aren't ported (see `SnbtGrammar`).

use std::sync::LazyLock;

use crate::com::mojang::brigadier::exceptions::command_syntax_exception::CommandSyntaxException;
use crate::com::mojang::brigadier::exceptions::simple_command_exception_type::SimpleCommandExceptionType;
use crate::com::mojang::brigadier::string_reader::StringReader;
use crate::com::mojang::serialization::codec::{
    BoxedCodec, CodecExt, STRING, SharedCodec, StaticCodec,
};
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::lifecycle::Lifecycle;
use crate::net::minecraft::nbt::compound_tag::CompoundTag;
use crate::net::minecraft::nbt::nbt_ops::NbtOps;
use crate::net::minecraft::nbt::snbt_grammar;
use crate::net::minecraft::nbt::tag::Tag;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::util::parsing::packrat::commands::command_argument_parser::CommandArgumentParser;
use crate::net::minecraft::util::parsing::packrat::commands::grammar::Grammar;

static ERROR_TRAILING_DATA: LazyLock<SimpleCommandExceptionType> = LazyLock::new(|| {
    SimpleCommandExceptionType::new(Component::from(Component::translatable(
        "argument.nbt.trailing",
    )))
});
static ERROR_EXPECTED_COMPOUND: LazyLock<SimpleCommandExceptionType> = LazyLock::new(|| {
    SimpleCommandExceptionType::new(Component::from(Component::translatable(
        "argument.nbt.expected.compound",
    )))
});
static NBT_OPS_PARSER: LazyLock<TagParser<Tag>> =
    LazyLock::new(|| TagParser::create(&NbtOps::INSTANCE));
static FLATTENED_CODEC: StaticCodec<CompoundTag> = LazyLock::new(|| {
    BoxedCodec::new(STRING.comap_flat_map(
        |s: String| match NBT_OPS_PARSER.parse_fully(&s) {
            Ok(Tag::Compound(compound_tag)) => {
                DataResult::success_lifecycle(compound_tag, Lifecycle::stable())
            }
            Ok(result) => DataResult::error(move || format!("Expected compound tag, got {result}")),
            Err(e) => DataResult::error(move || e.get_message()),
        },
        CompoundTag::to_string,
    ))
});
static LENIENT_CODEC: StaticCodec<CompoundTag> = LazyLock::new(|| {
    BoxedCodec::new(
        FLATTENED_CODEC
            .shared()
            .with_alternative(CompoundTag::CODEC.shared()),
    )
});

/// A parser of SNBT into the ops' values.
pub struct TagParser<T: 'static> {
    ops: &'static (dyn DynamicOps<T> + Sync),
    grammar: Grammar<T>,
}

impl<T: OpsValue + Send + Sync> TagParser<T> {
    pub const ELEMENT_SEPARATOR: u16 = b',' as u16;
    pub const NAME_VALUE_SEPARATOR: u16 = b':' as u16;

    pub fn get_ops(&self) -> &'static (dyn DynamicOps<T> + Sync) {
        self.ops
    }

    pub fn create(ops: &'static (dyn DynamicOps<T> + Sync)) -> TagParser<T> {
        TagParser {
            ops,
            grammar: snbt_grammar::create_parser(ops),
        }
    }

    /// `parseFully(String)`.
    pub fn parse_fully(&self, input: &str) -> Result<T, CommandSyntaxException> {
        self.parse_fully_reader(&mut StringReader::new(input))
    }

    /// `parseFully(StringReader)`: the value, then nothing but whitespace, which the cursor ends
    /// after.
    pub fn parse_fully_reader(
        &self,
        reader: &mut StringReader,
    ) -> Result<T, CommandSyntaxException> {
        let result = self.grammar.parse_for_commands(reader)?;
        reader.skip_whitespace();
        if reader.can_read() {
            return Err(ERROR_TRAILING_DATA.create_with_context(reader));
        }
        Ok(result)
    }

    /// The value, with the cursor after it.
    pub fn parse_as_argument(
        &self,
        reader: &mut StringReader,
    ) -> Result<T, CommandSyntaxException> {
        self.grammar.parse_for_commands(reader)
    }
}

impl TagParser<Tag> {
    pub const ERROR_TRAILING_DATA: &'static LazyLock<SimpleCommandExceptionType> =
        &ERROR_TRAILING_DATA;
    pub const ERROR_EXPECTED_COMPOUND: &'static LazyLock<SimpleCommandExceptionType> =
        &ERROR_EXPECTED_COMPOUND;
    pub const FLATTENED_CODEC: &'static StaticCodec<CompoundTag> = &FLATTENED_CODEC;
    pub const LENIENT_CODEC: &'static StaticCodec<CompoundTag> = &LENIENT_CODEC;

    /// The parser over `NbtOps` the static methods use, Java's private `NBT_OPS_PARSER`, which
    /// callers share here rather than make another (`TagParser.create(NbtOps.INSTANCE)`, which
    /// parses the same).
    pub fn nbt_ops_parser() -> &'static TagParser<Tag> {
        &NBT_OPS_PARSER
    }

    fn cast_to_compound_or_throw(
        reader: &StringReader,
        result: Tag,
    ) -> Result<CompoundTag, CommandSyntaxException> {
        match result {
            Tag::Compound(compound_tag) => Ok(compound_tag),
            _ => Err(ERROR_EXPECTED_COMPOUND.create_with_context(reader)),
        }
    }

    pub fn parse_compound_fully(input: &str) -> Result<CompoundTag, CommandSyntaxException> {
        let mut reader = StringReader::new(input);
        let result = NBT_OPS_PARSER.parse_fully_reader(&mut reader)?;
        TagParser::cast_to_compound_or_throw(&reader, result)
    }

    /// A compound, with the cursor after it (after any value, when it isn't one).
    pub fn parse_compound_as_argument(
        reader: &mut StringReader,
    ) -> Result<CompoundTag, CommandSyntaxException> {
        let result = NBT_OPS_PARSER.parse_as_argument(reader)?;
        TagParser::cast_to_compound_or_throw(reader, result)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::TagParser;
    use crate::com::mojang::brigadier::exceptions::command_syntax_exception::CommandSyntaxException;
    use crate::com::mojang::brigadier::string_reader::StringReader;
    use crate::com::mojang::serialization::codec::StaticCodec;
    use crate::com::mojang::serialization::data_result::tests::encode_result;
    use crate::com::mojang::serialization::decoder::Decoder;
    use crate::com::mojang::serialization::encoder::Encoder;
    use crate::golden::{Encode, Golden};
    use crate::net::minecraft::nbt::compound_tag::CompoundTag;
    use crate::net::minecraft::nbt::nbt_ops::NbtOps;
    use crate::net::minecraft::nbt::tag::Tag;

    // `["tag", tag]` or `["error", message]`, as the suite records a parse.
    fn outcome(result: Result<impl Into<Tag>, CommandSyntaxException>) -> Value {
        match result {
            Ok(tag) => json!(["tag", tag.into().encode()]),
            Err(e) => json!(["error", e.get_message()]),
        }
    }

    // The outcome, then the reader's cursor.
    fn with_cursor(mut outcome: Value, reader: &StringReader) -> Value {
        outcome
            .as_array_mut()
            .expect("an outcome")
            .push(json!(reader.get_cursor()));
        outcome
    }

    fn check_codec(golden: &Golden, name: &str, codec: &'static StaticCodec<CompoundTag>) {
        let ops = &NbtOps::INSTANCE;
        golden.check(&format!("{name}.decode"), |input: Tag| {
            encode_result(&codec.decode(ops, &input), |pair| {
                Box::new((pair.get_first().clone(), pair.get_second().clone()))
            })
        });
        golden.check(
            &format!("{name}.encode"),
            |value: CompoundTag, prefix: Tag| {
                encode_result(&codec.encode(&value, ops, prefix), |tag| {
                    Box::new(tag.clone())
                })
            },
        );
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.nbt.TagParser");
        let parser = TagParser::nbt_ops_parser();
        golden.check("parseFully(String)", |input: String| {
            outcome(parser.parse_fully(&input))
        });
        golden.check("parseFully(StringReader)", |input: String, start: i32| {
            let mut reader = StringReader::new(&input);
            reader.set_cursor(start);
            with_cursor(outcome(parser.parse_fully_reader(&mut reader)), &reader)
        });
        golden.check("parseAsArgument", |input: String| {
            let mut reader = StringReader::new(&input);
            with_cursor(outcome(parser.parse_as_argument(&mut reader)), &reader)
        });
        golden.check("parseCompoundFully", |input: String| {
            outcome(TagParser::parse_compound_fully(&input))
        });
        golden.check("parseCompoundAsArgument", |input: String| {
            let mut reader = StringReader::new(&input);
            with_cursor(
                outcome(TagParser::parse_compound_as_argument(&mut reader)),
                &reader,
            )
        });
        check_codec(&golden, "FLATTENED_CODEC", TagParser::FLATTENED_CODEC);
        check_codec(&golden, "LENIENT_CODEC", TagParser::LENIENT_CODEC);
        golden.finish();
    }
}
