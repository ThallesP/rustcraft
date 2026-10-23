//! net/minecraft/network/chat/contents/PlainTextContents.java
//!
//! Literal text. Java's interface, its `EMPTY` constant and its `LiteralContents` record are an
//! enum of the two; `create` makes `EMPTY` for an empty string, so a literal is never empty.

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{CodecExt, STRING};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::map_codec::{MapCodecExt, StaticMapCodec};
use crate::net::minecraft::network::chat::component_contents::ComponentContents;
use crate::net::minecraft::network::chat::formatted_text::FormattedText;
use crate::net::minecraft::network::chat::style::Style;

#[derive(Clone, Debug, PartialEq)]
pub enum PlainTextContents {
    Empty,
    Literal(String),
}

// `RecordCodecBuilder.mapCodec(i -> i.group(Codec.STRING.fieldOf("text")…).apply(i, create))`,
// of the contents.
static MAP_CODEC: StaticMapCodec<ComponentContents> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        (STRING
            .field_of("text")
            .for_getter(|contents: &ComponentContents| match contents {
                ComponentContents::Text(text) => text.text().to_owned(),
                other => panic!("ClassCastException: {other}"),
            }),),
        |text: String| ComponentContents::Text(PlainTextContents::create(&text)),
    )
    .boxed()
});

impl PlainTextContents {
    pub const MAP_CODEC: &'static StaticMapCodec<ComponentContents> = &MAP_CODEC;
    pub const EMPTY: PlainTextContents = PlainTextContents::Empty;

    pub fn create(text: &str) -> PlainTextContents {
        if text.is_empty() {
            PlainTextContents::Empty
        } else {
            PlainTextContents::Literal(text.to_owned())
        }
    }

    pub fn text(&self) -> &str {
        match self {
            PlainTextContents::Empty => "",
            PlainTextContents::Literal(text) => text,
        }
    }
}

/// `EMPTY` visits nothing; a literal its text.
impl FormattedText for PlainTextContents {
    fn visit<T>(&self, output: &mut dyn FnMut(&str) -> Option<T>) -> Option<T> {
        match self {
            PlainTextContents::Empty => None,
            PlainTextContents::Literal(text) => output(text),
        }
    }

    fn visit_styled<T>(
        &self,
        output: &mut dyn FnMut(&Style, &str) -> Option<T>,
        current_style: &Style,
    ) -> Option<T> {
        match self {
            PlainTextContents::Empty => None,
            PlainTextContents::Literal(text) => output(current_style, text),
        }
    }
}

/// `empty` and `literal{text}`.
impl fmt::Display for PlainTextContents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlainTextContents::Empty => f.write_str("empty"),
            PlainTextContents::Literal(text) => write!(f, "literal{{{text}}}"),
        }
    }
}
