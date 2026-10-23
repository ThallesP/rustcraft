//! net/minecraft/network/chat/contents/TranslatableContents.java
//!
//! A translation key, its fallback, and the arguments its translation's `%s`s take. Java's
//! arguments are `Object`s, a string, a number or a boolean (which `ExtraCodecs.JAVA` reads any
//! data as, and `filterAllowedArguments` keeps) or a component; here `TranslationArgument`. Java
//! decomposes the translation once per language; here on each visit, from the one language, which
//! gives the same parts. `resolve` (a server's) isn't ported.

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::datafixers::util::either::Either;
use crate::com::mojang::datafixers::util::pair::Pair;
use crate::com::mojang::serialization::codec::SharedCodec;
use crate::com::mojang::serialization::codec::{BoxedCodec, CodecExt, STRING, StaticCodec};
use crate::com::mojang::serialization::codecs::either_codec::EitherCodec;
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::decoder::Decoder;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::encoder::{Encoder, Valued};
use crate::com::mojang::serialization::java_ops::{JavaOps, JavaValue};
use crate::com::mojang::serialization::map_codec::{MapCodecExt, StaticMapCodec};
use crate::java::lang::double::Double;
use crate::java::lang::float::Float;
use crate::java::lang::integer::Integer;
use crate::java::lang::number::Number;
use crate::net::minecraft::locale::language::Language;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::component_contents::ComponentContents;
use crate::net::minecraft::network::chat::component_serialization::ComponentSerialization;
use crate::net::minecraft::network::chat::formatted_text::{FormattedText, Of};
use crate::net::minecraft::network::chat::style::Style;
use crate::net::minecraft::util::extra_codecs;

/// A translation's argument: what `isAllowedPrimitiveArgument` allows, or a component.
#[derive(Clone, Debug)]
pub enum TranslationArgument {
    String(String),
    Number(Number),
    Boolean(bool),
    Component(Component),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TranslatableContents {
    key: String,
    fallback: Option<String>,
    args: Vec<TranslationArgument>,
}

/// A part of the decomposed translation: text, or an argument's component.
enum Part {
    Text(Of),
    Component(Component),
}

/// `ExtraCodecs.JAVA.validate(filterAllowedArguments)`: any data read as the Java object it
/// converts to, kept if it's a string, a number or a boolean; written back from the object.
#[derive(Clone, Copy)]
struct PrimitiveArgCodec;

impl Valued for PrimitiveArgCodec {
    type Value = TranslationArgument;
}

impl<T: OpsValue> Decoder<T> for PrimitiveArgCodec {
    fn decode(
        &self,
        ops: &dyn DynamicOps<T>,
        input: &T,
    ) -> DataResult<Pair<TranslationArgument, T>> {
        let argument = match extra_codecs::convert(ops, &JavaOps::INSTANCE, input) {
            JavaValue::String(value) => TranslationArgument::String(value),
            JavaValue::Number(value) => TranslationArgument::Number(value),
            JavaValue::Boolean(value) => TranslationArgument::Boolean(value),
            _ => {
                return DataResult::error(|| {
                    "This value needs to be parsed as component".to_owned()
                });
            }
        };
        DataResult::success(Pair::new(argument, ops.empty()))
    }
}

impl<T: OpsValue> Encoder<T> for PrimitiveArgCodec {
    fn encode(
        &self,
        input: &TranslationArgument,
        ops: &dyn DynamicOps<T>,
        prefix: T,
    ) -> DataResult<T> {
        let value = match input {
            TranslationArgument::String(value) => JavaValue::String(value.clone()),
            TranslationArgument::Number(value) => JavaValue::Number(value.clone()),
            TranslationArgument::Boolean(value) => JavaValue::Boolean(*value),
            TranslationArgument::Component(_) => {
                return DataResult::error(|| {
                    "This value needs to be parsed as component".to_owned()
                });
            }
        };
        let casted = extra_codecs::convert(&JavaOps::INSTANCE, ops, &value);
        extra_codecs::passthrough_encode(casted, ops, prefix)
            .add_lifecycle(crate::com::mojang::serialization::lifecycle::Lifecycle::experimental())
    }
}

// `ARG_CODEC`: a primitive, or a component, which reads as its string when it's plain text.
static ARG_CODEC: StaticCodec<TranslationArgument> = LazyLock::new(|| {
    EitherCodec::new(PrimitiveArgCodec, ComponentSerialization::CODEC.shared())
        .xmap(
            |either| {
                either.map(
                    |argument| argument,
                    |component: Component| match component.try_collapse_to_string() {
                        Some(text) => TranslationArgument::String(text.to_owned()),
                        None => TranslationArgument::Component(component),
                    },
                )
            },
            |argument: &TranslationArgument| match argument {
                TranslationArgument::Component(component) => Either::Right(component.clone()),
                other => Either::Left(other.clone()),
            },
        )
        .boxed()
});

static MAP_CODEC: StaticMapCodec<ComponentContents> = LazyLock::new(|| {
    let contents = |contents: &ComponentContents| match contents {
        ComponentContents::Translatable(translatable) => translatable.clone(),
        other => panic!("ClassCastException: {other}"),
    };
    record_codec_builder::map_codec(
        (
            STRING
                .field_of("translate")
                .for_getter(move |o: &ComponentContents| contents(o).key),
            STRING
                .lenient_optional_field_of("fallback")
                .for_getter(move |o: &ComponentContents| contents(o).fallback),
            ARG_CODEC
                .shared()
                .list_of()
                .optional_field_of("with")
                .for_getter(move |o: &ComponentContents| {
                    let args = contents(o).args;
                    (!args.is_empty()).then_some(args)
                }),
        ),
        |key: String, fallback: Option<String>, args: Option<Vec<TranslationArgument>>| {
            ComponentContents::Translatable(TranslatableContents::new(
                key,
                fallback,
                args.unwrap_or_default(),
            ))
        },
    )
    .boxed()
});

impl TranslatableContents {
    pub const MAP_CODEC: &'static StaticMapCodec<ComponentContents> = &MAP_CODEC;

    pub fn new(
        key: String,
        fallback: Option<String>,
        args: Vec<TranslationArgument>,
    ) -> TranslatableContents {
        TranslatableContents {
            key,
            fallback,
            args,
        }
    }

    pub fn get_key(&self) -> &str {
        &self.key
    }

    pub fn get_fallback(&self) -> Option<&str> {
        self.fallback.as_deref()
    }

    pub fn get_args(&self) -> &[TranslationArgument] {
        &self.args
    }

    /// `ARG_CODEC`, for the arguments' goldens.
    pub fn arg_codec() -> BoxedCodec<TranslationArgument> {
        ARG_CODEC.shared()
    }

    // `decompose`: the translation (or the fallback, or the key) split into text and arguments,
    // or the whole translation as text when it doesn't decompose.
    fn decompose(&self) -> Vec<Part> {
        let language = Language::get_instance();
        let format = match &self.fallback {
            Some(fallback) => language.get_or_default_with(&self.key, fallback),
            None => language.get_or_default(&self.key),
        };
        self.decompose_template(&format)
            .unwrap_or_else(|| vec![Part::Text(Of(format))])
    }

    // `decomposeTemplate`: `None` where Java throws `TranslatableFormatException`.
    fn decompose_template(&self, template: &str) -> Option<Vec<Part>> {
        let mut parts = Vec::new();
        let mut replacement_index = 0;
        let mut current = 0;
        while let Some(format) = find_format(template, current) {
            if format.start > current {
                let prefix = &template[current..format.start];
                if prefix.contains('%') {
                    return None;
                }
                parts.push(Part::Text(Of(prefix.to_owned())));
            }
            let format_string = &template[format.start..format.end];
            if format.format_type == "%" && format_string == "%%" {
                parts.push(Part::Text(Of("%".to_owned())));
            } else {
                if format.format_type != "s" {
                    return None;
                }
                let index = match format.position {
                    Some(position) => Integer::parse_int(position).ok()?.wrapping_sub(1),
                    None => {
                        replacement_index += 1;
                        replacement_index - 1
                    }
                };
                parts.push(self.get_argument(index)?);
            }
            current = format.end;
        }
        if current < template.len() {
            let tail = &template[current..];
            if tail.contains('%') {
                return None;
            }
            parts.push(Part::Text(Of(tail.to_owned())));
        }
        Some(parts)
    }

    // `getArgument`: the argument's component, or its string.
    fn get_argument(&self, index: i32) -> Option<Part> {
        let arg = self.args.get(usize::try_from(index).ok()?)?;
        Some(match arg {
            TranslationArgument::Component(component) => Part::Component(component.clone()),
            other => Part::Text(Of(other.to_string())),
        })
    }
}

/// A match of `FORMAT_PATTERN`, `%(?:(\d+)\$)?([A-Za-z%]|$)`.
struct Format<'a> {
    start: usize,
    end: usize,
    position: Option<&'a str>,
    format_type: &'a str,
}

// `matcher.find(from)`: the first `%` from there that starts a match. `$` matches at the end, and
// before a line terminator that ends the input, like Java's without `MULTILINE`.
fn find_format(template: &str, from: usize) -> Option<Format<'_>> {
    let bytes = template.as_bytes();
    let mut start = from;
    while start < bytes.len() {
        if bytes[start] != b'%' {
            start += 1;
            continue;
        }
        let after = start + 1;
        let digits = bytes[after..]
            .iter()
            .take_while(|b| b.is_ascii_digit())
            .count();
        let candidates: &[(Option<usize>, usize)] =
            if digits > 0 && bytes.get(after + digits) == Some(&b'$') {
                &[(Some(digits), after + digits + 1), (None, after)]
            } else {
                &[(None, after)]
            };
        for &(position, at) in candidates {
            let position = position.map(|digits| &template[after..after + digits]);
            match bytes.get(at) {
                Some(c) if c.is_ascii_alphabetic() || *c == b'%' => {
                    return Some(Format {
                        start,
                        end: at + 1,
                        position,
                        format_type: &template[at..at + 1],
                    });
                }
                _ if is_end(&template[at..]) => {
                    return Some(Format {
                        start,
                        end: at,
                        position,
                        format_type: "",
                    });
                }
                _ => {}
            }
        }
        start += 1;
    }
    None
}

// Whether `$` matches before this rest of the input: nothing, or one final line terminator.
fn is_end(rest: &str) -> bool {
    matches!(
        rest,
        "" | "\n" | "\r\n" | "\r" | "\u{85}" | "\u{2028}" | "\u{2029}"
    )
}

impl FormattedText for TranslatableContents {
    fn visit<T>(&self, output: &mut dyn FnMut(&str) -> Option<T>) -> Option<T> {
        for part in self.decompose() {
            let result = match &part {
                Part::Text(text) => text.visit(output),
                Part::Component(component) => component.visit(output),
            };
            if result.is_some() {
                return result;
            }
        }
        None
    }

    fn visit_styled<T>(
        &self,
        output: &mut dyn FnMut(&Style, &str) -> Option<T>,
        current_style: &Style,
    ) -> Option<T> {
        for part in self.decompose() {
            let result = match &part {
                Part::Text(text) => text.visit_styled(output, current_style),
                Part::Component(component) => component.visit_styled(output, current_style),
            };
            if result.is_some() {
                return result;
            }
        }
        None
    }
}

/// `equals`: a number by its box and `equals`, so a float compares its bits.
impl PartialEq for TranslationArgument {
    fn eq(&self, other: &TranslationArgument) -> bool {
        match (self, other) {
            (TranslationArgument::String(a), TranslationArgument::String(b)) => a == b,
            (TranslationArgument::Boolean(a), TranslationArgument::Boolean(b)) => a == b,
            (TranslationArgument::Component(a), TranslationArgument::Component(b)) => a == b,
            (
                TranslationArgument::Number(Number::Float(a)),
                TranslationArgument::Number(Number::Float(b)),
            ) => Float::float_to_int_bits(*a) == Float::float_to_int_bits(*b),
            (
                TranslationArgument::Number(Number::Double(a)),
                TranslationArgument::Number(Number::Double(b)),
            ) => Double::double_to_long_bits(*a) == Double::double_to_long_bits(*b),
            (TranslationArgument::Number(a), TranslationArgument::Number(b)) => a == b,
            _ => false,
        }
    }
}

/// Each kind's `toString`.
impl fmt::Display for TranslationArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TranslationArgument::String(value) => f.write_str(value),
            TranslationArgument::Number(value) => write!(f, "{value}"),
            TranslationArgument::Boolean(value) => write!(f, "{value}"),
            TranslationArgument::Component(value) => write!(f, "{value}"),
        }
    }
}

/// `translation{key='…', fallback='…', args=[…]}`, the fallback only when there's one.
impl fmt::Display for TranslatableContents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "translation{{key='{}'", self.key)?;
        if let Some(fallback) = &self.fallback {
            write!(f, ", fallback='{fallback}'")?;
        }
        let args: Vec<String> = self.args.iter().map(ToString::to_string).collect();
        write!(f, ", args=[{}]}}", args.join(", "))
    }
}
