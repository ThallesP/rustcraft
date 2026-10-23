//! net/minecraft/network/chat/MutableComponent.java
//!
//! The component class: contents, siblings and a style, which `append` and the `with…` methods
//! change in place. `getVisualOrderText` (bidirectional text, cached per language) comes with the
//! font renderer, and the `ChatFormatting` overloads of `withStyle` with `ChatFormatting`.

use std::fmt;

use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::network::chat::component_contents::ComponentContents;
use crate::net::minecraft::network::chat::style::Style;
use crate::net::minecraft::network::chat::text_color::TextColor;

#[derive(Clone, Debug)]
pub struct MutableComponent {
    contents: ComponentContents,
    siblings: Vec<Component>,
    style: Style,
}

impl MutableComponent {
    pub(crate) fn new(
        contents: ComponentContents,
        siblings: Vec<Component>,
        style: Style,
    ) -> MutableComponent {
        MutableComponent {
            contents,
            siblings,
            style,
        }
    }

    pub fn create(contents: ComponentContents) -> MutableComponent {
        MutableComponent::new(contents, Vec::new(), Style::EMPTY)
    }

    pub fn get_contents(&self) -> &ComponentContents {
        &self.contents
    }

    pub fn get_siblings(&self) -> &[Component] {
        &self.siblings
    }

    pub fn get_style(&self) -> &Style {
        &self.style
    }

    pub fn set_style(&mut self, style: Style) -> &mut MutableComponent {
        self.style = style;
        self
    }

    /// `append(String)`: a literal, unless the text's empty.
    pub fn append_str(&mut self, text: &str) -> &mut MutableComponent {
        if text.is_empty() {
            self
        } else {
            self.append(Component::literal(text).into())
        }
    }

    pub fn append(&mut self, component: Component) -> &mut MutableComponent {
        self.siblings.push(component);
        self
    }

    /// `withStyle(Style)`: the patch applied over this style.
    pub fn with_style(&mut self, patch: &Style) -> &mut MutableComponent {
        let style = patch.apply_to(&self.style);
        self.set_style(style)
    }

    /// `withColor(int)`.
    pub fn with_color_rgb(&mut self, color: i32) -> &mut MutableComponent {
        let style = self.style.with_color_rgb(color);
        self.set_style(style)
    }

    pub fn with_color(&mut self, color: TextColor) -> &mut MutableComponent {
        let style = self.style.with_color(Some(color));
        self.set_style(style)
    }

    pub fn without_shadow(&mut self) -> &mut MutableComponent {
        let style = self.style.without_shadow();
        self.set_style(style)
    }
}

/// Contents, style and siblings, in that order.
impl PartialEq for MutableComponent {
    fn eq(&self, that: &MutableComponent) -> bool {
        self.contents == that.contents && self.style == that.style && self.siblings == that.siblings
    }
}

/// The contents, then `[style=…, siblings=[…]]` for what's there.
impl fmt::Display for MutableComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.contents)?;
        let has_style = !self.style.is_empty();
        let has_siblings = !self.siblings.is_empty();
        if has_style || has_siblings {
            f.write_str("[")?;
            if has_style {
                write!(f, "style={}", self.style)?;
            }
            if has_style && has_siblings {
                f.write_str(", ")?;
            }
            if has_siblings {
                let siblings: Vec<String> = self.siblings.iter().map(ToString::to_string).collect();
                write!(f, "siblings=[{}]", siblings.join(", "))?;
            }
            f.write_str("]")?;
        }
        Ok(())
    }
}
