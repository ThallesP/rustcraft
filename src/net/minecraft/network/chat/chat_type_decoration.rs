//! net/minecraft/network/chat/ChatTypeDecoration.java
//!
//! How a chat type lays out a message: a translation, its arguments, and a style. Not ported yet:
//! `decorate` and the parameters' selectors (`ChatType.Bound`), `STREAM_CODEC`, `withSender`,
//! `incomingDirectMessage` and their kin, which come with chat.

use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{CodecExt, STRING, SharedCodec, StaticCodec};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::map_codec::MapCodecExt;
use crate::net::minecraft::network::chat::style::{Serializer, Style};
use crate::net::minecraft::util::string_representable::{self, StringRepresentable};

static CODEC: StaticCodec<ChatTypeDecoration> = LazyLock::new(|| {
    record_codec_builder::create(
        (
            STRING
                .field_of("translation_key")
                .for_getter(|o: &ChatTypeDecoration| o.translation_key.clone()),
            Parameter::CODEC
                .shared()
                .list_of()
                .field_of("parameters")
                .for_getter(|o: &ChatTypeDecoration| o.parameters.clone()),
            Serializer::CODEC
                .shared()
                .optional_field_of_default("style", Style::EMPTY)
                .for_getter(|o: &ChatTypeDecoration| o.style.clone()),
        ),
        ChatTypeDecoration::new,
    )
    .boxed()
});

#[derive(Clone, Debug, PartialEq)]
pub struct ChatTypeDecoration {
    translation_key: String,
    parameters: Vec<Parameter>,
    style: Style,
}

impl ChatTypeDecoration {
    pub const CODEC: &'static StaticCodec<ChatTypeDecoration> = &CODEC;

    pub fn new(
        translation_key: String,
        parameters: Vec<Parameter>,
        style: Style,
    ) -> ChatTypeDecoration {
        ChatTypeDecoration {
            translation_key,
            parameters,
            style,
        }
    }

    pub fn translation_key(&self) -> &str {
        &self.translation_key
    }

    pub fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }

    pub fn style(&self) -> &Style {
        &self.style
    }
}

static PARAMETER_CODEC: StaticCodec<Parameter> =
    LazyLock::new(|| string_representable::from_values(&Parameter::VALUES));

/// `ChatTypeDecoration.Parameter`: what a translation's argument is. Its selector comes with
/// `decorate`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::upper_case_acronyms)]
pub enum Parameter {
    SENDER,
    TARGET,
    CONTENT,
}

impl Parameter {
    pub const VALUES: [Parameter; 3] = [Parameter::SENDER, Parameter::TARGET, Parameter::CONTENT];
    pub const CODEC: &'static StaticCodec<Parameter> = &PARAMETER_CODEC;
}

impl StringRepresentable for Parameter {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            Parameter::SENDER => "sender",
            Parameter::TARGET => "target",
            Parameter::CONTENT => "content",
        }
    }
}
