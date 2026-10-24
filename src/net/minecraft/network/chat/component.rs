//! net/minecraft/network/chat/Component.java
//!
//! A text: what a chat message, a title or a disconnect reason is. Java's interface has one class
//! the client reads, `MutableComponent`, shared by reference; here a `Component` is a shared
//! `MutableComponent` (an `Arc`, which derefs to it), and the static factories make the mutable
//! one, which `into()` shares. `getVisualOrderText` comes with the font renderer, `toFlatList` and
//! `contains` with their callers, and the factories of contents not ported yet (`nbt`, `score`,
//! `selector`, `object`) with them.
//!
//! `visit` and `visit_styled` (`FormattedText`'s) hand each piece of text to a consumer in order,
//! the contents' then each sibling's, the styled one with each piece's style: the component's own
//! style applied over its parent's.

use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

use crate::com::mojang::brigadier::message::Message;
use crate::com::mojang::datafixers::util::unit::Unit;
use crate::net::minecraft::network::chat::component_contents::ComponentContents;
use crate::net::minecraft::network::chat::contents::keybind_contents::KeybindContents;
use crate::net::minecraft::network::chat::contents::plain_text_contents::PlainTextContents;
use crate::net::minecraft::network::chat::contents::translatable_contents::{
    TranslatableContents, TranslationArgument,
};
use crate::net::minecraft::network::chat::formatted_text::FormattedText;
use crate::net::minecraft::network::chat::mutable_component::MutableComponent;
use crate::net::minecraft::network::chat::style::Style;

#[derive(Clone, Debug, PartialEq)]
pub struct Component(Arc<MutableComponent>);

impl Deref for Component {
    type Target = MutableComponent;

    fn deref(&self) -> &MutableComponent {
        &self.0
    }
}

impl From<MutableComponent> for Component {
    fn from(component: MutableComponent) -> Component {
        Component(Arc::new(component))
    }
}

impl Component {
    pub fn literal(text: &str) -> MutableComponent {
        MutableComponent::create(ComponentContents::Text(PlainTextContents::create(text)))
    }

    pub fn translatable(key: &str) -> MutableComponent {
        Component::translatable_with_args(key, Vec::new())
    }

    /// `translatable(String, Object...)`.
    pub fn translatable_with_args(key: &str, args: Vec<TranslationArgument>) -> MutableComponent {
        MutableComponent::create(ComponentContents::Translatable(TranslatableContents::new(
            key.to_owned(),
            None,
            args,
        )))
    }

    /// `translatableWithFallback(String, String, Object...)`.
    pub fn translatable_with_fallback(
        key: &str,
        fallback: Option<&str>,
        args: Vec<TranslationArgument>,
    ) -> MutableComponent {
        MutableComponent::create(ComponentContents::Translatable(TranslatableContents::new(
            key.to_owned(),
            fallback.map(str::to_owned),
            args,
        )))
    }

    pub fn empty() -> MutableComponent {
        MutableComponent::create(ComponentContents::Text(PlainTextContents::EMPTY))
    }

    pub fn keybind(name: &str) -> MutableComponent {
        MutableComponent::create(ComponentContents::Keybind(KeybindContents::new(
            name.to_owned(),
        )))
    }
}

impl MutableComponent {
    /// `getString(int)`: at most `limit` UTF-16 units of the text.
    pub fn get_string_limit(&self, limit: usize) -> String {
        let mut builder: Vec<u16> = Vec::new();
        self.visit::<Unit>(&mut |contents| {
            if builder.len() >= limit {
                return Some(Unit::INSTANCE);
            }
            let remaining = limit - builder.len();
            builder.extend(contents.encode_utf16().take(remaining));
            None
        });
        String::from_utf16_lossy(&builder)
    }

    /// The text, when this is plain text with nothing else: no siblings and the `EMPTY` style.
    pub fn try_collapse_to_string(&self) -> Option<&str> {
        match self.get_contents() {
            ComponentContents::Text(text)
                if self.get_siblings().is_empty() && self.get_style().is_empty() =>
            {
                Some(text.text())
            }
            _ => None,
        }
    }

    /// `plainCopy`: the contents alone.
    pub fn plain_copy(&self) -> MutableComponent {
        MutableComponent::create(self.get_contents().clone())
    }

    /// `copy`: the contents, siblings and style, the siblings in a list of their own.
    pub fn copy(&self) -> MutableComponent {
        MutableComponent::new(
            self.get_contents().clone(),
            self.get_siblings().to_vec(),
            self.get_style().clone(),
        )
    }
}

impl FormattedText for MutableComponent {
    fn visit<T>(&self, output: &mut dyn FnMut(&str) -> Option<T>) -> Option<T> {
        if let Some(result) = self.get_contents().visit(output) {
            return Some(result);
        }
        for sibling in self.get_siblings() {
            if let Some(result) = sibling.visit(output) {
                return Some(result);
            }
        }
        None
    }

    fn visit_styled<T>(
        &self,
        output: &mut dyn FnMut(&Style, &str) -> Option<T>,
        parent_style: &Style,
    ) -> Option<T> {
        let self_style = self.get_style().apply_to(parent_style);
        if let Some(result) = self.get_contents().visit_styled(output, &self_style) {
            return Some(result);
        }
        for sibling in self.get_siblings() {
            if let Some(result) = sibling.visit_styled(output, &self_style) {
                return Some(result);
            }
        }
        None
    }
}

impl FormattedText for Component {
    fn visit<T>(&self, output: &mut dyn FnMut(&str) -> Option<T>) -> Option<T> {
        self.0.visit(output)
    }

    fn visit_styled<T>(
        &self,
        output: &mut dyn FnMut(&Style, &str) -> Option<T>,
        parent_style: &Style,
    ) -> Option<T> {
        self.0.visit_styled(output, parent_style)
    }
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Java's `Component` is Brigadier's `Message`: the game's command errors are components.
impl Message for Component {
    fn get_string(&self) -> String {
        FormattedText::get_string(&**self)
    }
}
