//! net/minecraft/network/chat/ComponentContents.java
//!
//! What a component holds before its siblings: text, a translation, a key binding's name, a score,
//! an entity selector, NBT from a data source, or an object.
//! Java's interface is an enum of the ported classes here, each a variant, and `codec()` is the
//! variant's `ContentType`, which `ComponentSerialization` dispatches on. `resolve`, which only a
//! server runs (commands), isn't ported.

use std::fmt;

use crate::com::mojang::serialization::map_codec::BoxedMapCodec;
use crate::com::mojang::serialization::map_codec::SharedMapCodec;
use crate::net::minecraft::network::chat::contents::keybind_contents::KeybindContents;
use crate::net::minecraft::network::chat::contents::nbt_contents::NbtContents;
use crate::net::minecraft::network::chat::contents::object_contents::ObjectContents;
use crate::net::minecraft::network::chat::contents::plain_text_contents::PlainTextContents;
use crate::net::minecraft::network::chat::contents::score_contents::ScoreContents;
use crate::net::minecraft::network::chat::contents::selector_contents::SelectorContents;
use crate::net::minecraft::network::chat::contents::translatable_contents::TranslatableContents;
use crate::net::minecraft::network::chat::formatted_text::FormattedText;
use crate::net::minecraft::network::chat::style::Style;

#[derive(Clone, Debug, PartialEq)]
pub enum ComponentContents {
    Text(PlainTextContents),
    Translatable(TranslatableContents),
    Keybind(KeybindContents),
    Score(ScoreContents),
    Selector(SelectorContents),
    Nbt(NbtContents),
    Object(ObjectContents),
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
            ContentType::Score => ScoreContents::MAP_CODEC.shared(),
            ContentType::Selector => SelectorContents::MAP_CODEC.shared(),
            ContentType::Nbt => NbtContents::MAP_CODEC.shared(),
            ContentType::Object => ObjectContents::MAP_CODEC.shared(),
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
            ComponentContents::Score(_) => ContentType::Score,
            ComponentContents::Selector(_) => ContentType::Selector,
            ComponentContents::Nbt(_) => ContentType::Nbt,
            ComponentContents::Object(_) => ContentType::Object,
        }
    }
}

impl FormattedText for ComponentContents {
    fn visit<T>(&self, output: &mut dyn FnMut(&str) -> Option<T>) -> Option<T> {
        match self {
            ComponentContents::Text(contents) => contents.visit(output),
            ComponentContents::Translatable(contents) => contents.visit(output),
            ComponentContents::Keybind(contents) => contents.visit(output),
            // Java's default: a score shows nothing until the server resolves it.
            ComponentContents::Score(_) | ComponentContents::Nbt(_) => None,
            ComponentContents::Selector(contents) => contents.visit(output),
            ComponentContents::Object(contents) => contents.visit(output),
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
            ComponentContents::Score(_) | ComponentContents::Nbt(_) => None,
            ComponentContents::Selector(contents) => contents.visit_styled(output, current_style),
            ComponentContents::Object(contents) => contents.visit_styled(output, current_style),
        }
    }
}

impl fmt::Display for ComponentContents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentContents::Text(contents) => contents.fmt(f),
            ComponentContents::Translatable(contents) => contents.fmt(f),
            ComponentContents::Keybind(contents) => contents.fmt(f),
            ComponentContents::Score(contents) => contents.fmt(f),
            ComponentContents::Selector(contents) => contents.fmt(f),
            ComponentContents::Nbt(contents) => contents.fmt(f),
            ComponentContents::Object(contents) => contents.fmt(f),
        }
    }
}
