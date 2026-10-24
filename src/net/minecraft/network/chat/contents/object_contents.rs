//! net/minecraft/network/chat/contents/ObjectContents.java
//!
//! An object drawn as a glyph (a sprite, a player's head), with the text to show where it can't
//! be (`fallback`). Its styled text is the placeholder `U+FFFC` in the object's font; its plain
//! text is the fallback. `resolve` (the server's) isn't ported.

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{CodecExt, SharedCodec};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::map_codec::{MapCodecExt, SharedMapCodec, StaticMapCodec};
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::component_contents::ComponentContents;
use crate::net::minecraft::network::chat::component_serialization::ComponentSerialization;
use crate::net::minecraft::network::chat::contents::objects::object_info::ObjectInfo;
use crate::net::minecraft::network::chat::contents::objects::object_infos::ObjectInfos;
use crate::net::minecraft::network::chat::formatted_text::FormattedText;
use crate::net::minecraft::network::chat::style::Style;

const PLACEHOLDER: &str = "\u{FFFC}";

static MAP_CODEC: StaticMapCodec<ComponentContents> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        (
            ObjectInfos::CODEC
                .shared()
                .for_getter(|contents: &ComponentContents| match contents {
                    ComponentContents::Object(object) => object.contents.clone(),
                    other => panic!("ClassCastException: {other}"),
                }),
            ComponentSerialization::CODEC
                .shared()
                .optional_field_of("fallback")
                .for_getter(|contents: &ComponentContents| match contents {
                    ComponentContents::Object(object) => object.fallback.clone(),
                    other => panic!("ClassCastException: {other}"),
                }),
        ),
        |contents, fallback| ComponentContents::Object(ObjectContents::new(contents, fallback)),
    )
    .boxed()
});

#[derive(Clone, Debug, PartialEq)]
pub struct ObjectContents {
    contents: ObjectInfo,
    fallback: Option<Component>,
}

impl ObjectContents {
    pub const MAP_CODEC: &'static StaticMapCodec<ComponentContents> = &MAP_CODEC;

    pub fn new(contents: ObjectInfo, fallback: Option<Component>) -> ObjectContents {
        ObjectContents { contents, fallback }
    }

    pub fn contents(&self) -> &ObjectInfo {
        &self.contents
    }

    pub fn fallback(&self) -> Option<&Component> {
        self.fallback.as_ref()
    }
}

impl FormattedText for ObjectContents {
    /// The fallback's text, or the object's own.
    fn visit<T>(&self, output: &mut dyn FnMut(&str) -> Option<T>) -> Option<T> {
        match &self.fallback {
            Some(fallback) => fallback.visit(output),
            None => output(&self.contents.default_fallback()),
        }
    }

    /// The placeholder, in the object's font.
    fn visit_styled<T>(
        &self,
        output: &mut dyn FnMut(&Style, &str) -> Option<T>,
        current_style: &Style,
    ) -> Option<T> {
        output(
            &current_style.with_font(Some(self.contents.font_description())),
            PLACEHOLDER,
        )
    }
}

/// The record's `toString`.
impl fmt::Display for ObjectContents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ObjectContents[contents={}, fallback=", self.contents)?;
        match &self.fallback {
            Some(fallback) => write!(f, "Optional[{fallback}]]"),
            None => f.write_str("Optional.empty]"),
        }
    }
}
