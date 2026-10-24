//! net/minecraft/network/chat/contents/SelectorContents.java
//!
//! The entities a selector finds, which the server resolves (`resolve`, not ported) before it sends
//! a component; one sent unresolved shows the selector's text.

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{CodecExt, SharedCodec};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::map_codec::{MapCodecExt, StaticMapCodec};
use crate::net::minecraft::commands::arguments::selector::entity_selector::EntitySelector;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::component_contents::ComponentContents;
use crate::net::minecraft::network::chat::component_serialization::ComponentSerialization;
use crate::net::minecraft::network::chat::formatted_text::FormattedText;
use crate::net::minecraft::network::chat::style::Style;
use crate::net::minecraft::util::compilable_string::CompilableString;

#[derive(Clone, Debug, PartialEq)]
pub struct SelectorContents {
    selector: CompilableString<EntitySelector>,
    separator: Option<Component>,
}

static MAP_CODEC: StaticMapCodec<ComponentContents> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        (
            EntitySelector::compilable_codec()
                .field_of("selector")
                .for_getter(|contents: &ComponentContents| match contents {
                    ComponentContents::Selector(selector) => selector.selector.clone(),
                    other => panic!("ClassCastException: {other}"),
                }),
            ComponentSerialization::CODEC
                .shared()
                .optional_field_of("separator")
                .for_getter(|contents: &ComponentContents| match contents {
                    ComponentContents::Selector(selector) => selector.separator.clone(),
                    other => panic!("ClassCastException: {other}"),
                }),
        ),
        |selector, separator| {
            ComponentContents::Selector(SelectorContents::new(selector, separator))
        },
    )
    .boxed()
});

impl SelectorContents {
    pub const MAP_CODEC: &'static StaticMapCodec<ComponentContents> = &MAP_CODEC;

    pub fn new(
        selector: CompilableString<EntitySelector>,
        separator: Option<Component>,
    ) -> SelectorContents {
        SelectorContents {
            selector,
            separator,
        }
    }

    pub fn selector(&self) -> &CompilableString<EntitySelector> {
        &self.selector
    }

    pub fn separator(&self) -> Option<&Component> {
        self.separator.as_ref()
    }
}

/// The selector's text.
impl FormattedText for SelectorContents {
    fn visit<T>(&self, output: &mut dyn FnMut(&str) -> Option<T>) -> Option<T> {
        output(self.selector.source())
    }

    fn visit_styled<T>(
        &self,
        output: &mut dyn FnMut(&Style, &str) -> Option<T>,
        current_style: &Style,
    ) -> Option<T> {
        output(current_style, self.selector.source())
    }
}

/// `pattern{@e[type=pig]}`.
impl fmt::Display for SelectorContents {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "pattern{{{}}}", self.selector)
    }
}
