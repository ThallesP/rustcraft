//! net/minecraft/network/chat/ComponentContents.java
//!
//! What a component holds before its siblings: text, a translation, a key binding's name, and the
//! kinds not ported yet (a score, an entity selector, NBT, an object), whose codecs fail to decode.
//! Java's interface is an enum of the ported classes here, each a variant, and `codec()` is the
//! variant's `ContentType`, which `ComponentSerialization` dispatches on. `resolve`, which only a
//! server runs (commands), isn't ported.

use std::fmt;

use crate::com::mojang::serialization::map_codec::SharedMapCodec;
use crate::com::mojang::serialization::map_codec::{self, BoxedMapCodec, MapCodecExt};
use crate::net::minecraft::network::chat::contents::keybind_contents::KeybindContents;
use crate::net::minecraft::network::chat::contents::plain_text_contents::PlainTextContents;
use crate::net::minecraft::network::chat::contents::translatable_contents::TranslatableContents;
use crate::net::minecraft::network::chat::formatted_text::FormattedText;
use crate::net::minecraft::network::chat::style::Style;

#[derive(Clone, Debug, PartialEq)]
pub enum ComponentContents {
    Text(PlainTextContents),
    Translatable(TranslatableContents),
    Keybind(KeybindContents),
}

/// Each class's `MAP_CODEC`, the values of `ComponentSerialization`'s `LateBoundIdMapper`, in the
/// order its `bootstrap` puts them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentType {
    Text,
    Translatable,
    Keybind,
    Score,
    Selector,
    Nbt,
    Object,
}

impl ContentType {
    /// The class's map codec, of the contents.
    pub fn codec(self) -> BoxedMapCodec<ComponentContents> {
        match self {
            ContentType::Text => PlainTextContents::MAP_CODEC.shared(),
            ContentType::Translatable => TranslatableContents::MAP_CODEC.shared(),
            ContentType::Keybind => KeybindContents::MAP_CODEC.shared(),
            ContentType::Score => map_codec::unported("ScoreContents").boxed(),
            ContentType::Selector => map_codec::unported("SelectorContents").boxed(),
            ContentType::Nbt => map_codec::unported("NbtContents").boxed(),
            ContentType::Object => map_codec::unported("ObjectContents").boxed(),
        }
    }
}

/// `MapCodec.toString`, which an unknown element's error would print; the port never writes one.
impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl ComponentContents {
    /// `codec()`.
    pub fn codec(&self) -> ContentType {
        match self {
            ComponentContents::Text(_) => ContentType::Text,
            ComponentContents::Translatable(_) => ContentType::Translatable,
            ComponentContents::Keybind(_) => ContentType::Keybind,
        }
    }
}

impl FormattedText for ComponentContents {
    fn visit<T>(&self, output: &mut dyn FnMut(&str) -> Option<T>) -> Option<T> {
        match self {
            ComponentContents::Text(contents) => contents.visit(output),
            ComponentContents::Translatable(contents) => contents.visit(output),
            ComponentContents::Keybind(contents) => contents.visit(output),
        }
    }

    fn visit_styled<T>(
        &self,
        output: &mut dyn FnMut(&Style, &str) -> Option<T>,
        current_style: &Style,
    ) -> Option<T> {
        match self {
            ComponentContents::Text(contents) => contents.visit_styled(output, current_style),
            ComponentContents::Translatable(contents) => {
                contents.visit_styled(output, current_style)
            }
            ComponentContents::Keybind(contents) => contents.visit_styled(output, current_style),
        }
    }
}

impl fmt::Display for ComponentContents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentContents::Text(contents) => contents.fmt(f),
            ComponentContents::Translatable(contents) => contents.fmt(f),
            ComponentContents::Keybind(contents) => contents.fmt(f),
        }
    }
}
