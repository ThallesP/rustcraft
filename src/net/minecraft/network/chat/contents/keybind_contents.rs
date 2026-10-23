//! net/minecraft/network/chat/contents/KeybindContents.java
//!
//! A key binding's name, shown as `KeybindResolver` resolves it. Java keeps the resolved component
//! once made (`nameResolver`); here it's resolved on each visit, which gives the same text.

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{CodecExt, STRING};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::map_codec::{MapCodecExt, StaticMapCodec};
use crate::net::minecraft::network::chat::component_contents::ComponentContents;
use crate::net::minecraft::network::chat::contents::keybind_resolver::KeybindResolver;
use crate::net::minecraft::network::chat::formatted_text::FormattedText;
use crate::net::minecraft::network::chat::style::Style;

#[derive(Clone, Debug, PartialEq)]
pub struct KeybindContents {
    name: String,
}

static MAP_CODEC: StaticMapCodec<ComponentContents> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        (STRING
            .field_of("keybind")
            .for_getter(|contents: &ComponentContents| match contents {
                ComponentContents::Keybind(keybind) => keybind.name.clone(),
                other => panic!("ClassCastException: {other}"),
            }),),
        |name: String| ComponentContents::Keybind(KeybindContents::new(name)),
    )
    .boxed()
});

impl KeybindContents {
    pub const MAP_CODEC: &'static StaticMapCodec<ComponentContents> = &MAP_CODEC;

    pub fn new(name: String) -> KeybindContents {
        KeybindContents { name }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

impl FormattedText for KeybindContents {
    fn visit<T>(&self, output: &mut dyn FnMut(&str) -> Option<T>) -> Option<T> {
        KeybindResolver::resolve(&self.name).visit(output)
    }

    fn visit_styled<T>(
        &self,
        output: &mut dyn FnMut(&Style, &str) -> Option<T>,
        current_style: &Style,
    ) -> Option<T> {
        KeybindResolver::resolve(&self.name).visit_styled(output, current_style)
    }
}

/// `keybind{name}`.
impl fmt::Display for KeybindContents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "keybind{{{}}}", self.name)
    }
}
