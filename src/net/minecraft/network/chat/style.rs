//! net/minecraft/network/chat/Style.java
//!
//! How a text looks: its color, shadow, the bold, italic, underlined, strikethrough and
//! obfuscated flags (each unset, true or false), its click and hover events, the text it inserts,
//! and its font. Java tells the `EMPTY` constant apart by reference (`isEmpty`, `applyTo`), and
//! makes it wherever a style would equal it (`create`, `checkEmptyAfterChange`); here the constant
//! carries a flag only it and its copies have. `applyLegacyFormat` is a `§` code's change; the
//! other `ChatFormatting` methods (`withColor(ChatFormatting)`, `applyFormat`, `applyFormats`) and
//! `hashCode` come with their first callers.

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::SharedCodec;
use crate::com::mojang::serialization::codec::StaticCodec;
use crate::com::mojang::serialization::codec::{BOOL, CodecExt, STRING};
use crate::com::mojang::serialization::codecs::record_codec_builder;
use crate::com::mojang::serialization::map_codec::SharedMapCodec;
use crate::com::mojang::serialization::map_codec::{MapCodecExt, StaticMapCodec};
use crate::net::minecraft::chat_formatting::ChatFormatting;
use crate::net::minecraft::network::chat::click_event::ClickEvent;
use crate::net::minecraft::network::chat::font_description::FontDescription;
use crate::net::minecraft::network::chat::hover_event::HoverEvent;
use crate::net::minecraft::network::chat::text_color::TextColor;
use crate::net::minecraft::util::extra_codecs::ExtraCodecs;

#[derive(Clone, Debug)]
pub struct Style {
    color: Option<TextColor>,
    shadow_color: Option<i32>,
    bold: Option<bool>,
    italic: Option<bool>,
    underlined: Option<bool>,
    strikethrough: Option<bool>,
    obfuscated: Option<bool>,
    click_event: Option<ClickEvent>,
    hover_event: Option<HoverEvent>,
    insertion: Option<String>,
    font: Option<FontDescription>,
    // Whether this is `EMPTY` itself.
    is_empty_constant: bool,
}

impl Style {
    pub const EMPTY: Style = Style {
        color: None,
        shadow_color: None,
        bold: None,
        italic: None,
        underlined: None,
        strikethrough: None,
        obfuscated: None,
        click_event: None,
        hover_event: None,
        insertion: None,
        font: None,
        is_empty_constant: true,
    };
    pub const NO_SHADOW: i32 = 0;

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        color: Option<TextColor>,
        shadow_color: Option<i32>,
        bold: Option<bool>,
        italic: Option<bool>,
        underlined: Option<bool>,
        strikethrough: Option<bool>,
        obfuscated: Option<bool>,
        click_event: Option<ClickEvent>,
        hover_event: Option<HoverEvent>,
        insertion: Option<String>,
        font: Option<FontDescription>,
    ) -> Style {
        Style {
            color,
            shadow_color,
            bold,
            italic,
            underlined,
            strikethrough,
            obfuscated,
            click_event,
            hover_event,
            insertion,
            font,
            is_empty_constant: false,
        }
    }

    /// The codec's constructor: `EMPTY` for a style equal to it.
    #[allow(clippy::too_many_arguments)]
    fn create(
        color: Option<TextColor>,
        shadow_color: Option<i32>,
        bold: Option<bool>,
        italic: Option<bool>,
        underlined: Option<bool>,
        strikethrough: Option<bool>,
        obfuscated: Option<bool>,
        click_event: Option<ClickEvent>,
        hover_event: Option<HoverEvent>,
        insertion: Option<String>,
        font: Option<FontDescription>,
    ) -> Style {
        let result = Style::new(
            color,
            shadow_color,
            bold,
            italic,
            underlined,
            strikethrough,
            obfuscated,
            click_event,
            hover_event,
            insertion,
            font,
        );
        if result == Style::EMPTY {
            Style::EMPTY
        } else {
            result
        }
    }

    /// The flags as they're set, for the goldens: Java's getters say only whether one is true.
    #[cfg(test)]
    pub(crate) fn flags(&self) -> [Option<bool>; 5] {
        [
            self.bold,
            self.italic,
            self.underlined,
            self.strikethrough,
            self.obfuscated,
        ]
    }

    /// The font as it's set, for the goldens.
    #[cfg(test)]
    pub(crate) fn font(&self) -> Option<&FontDescription> {
        self.font.as_ref()
    }

    pub fn get_color(&self) -> Option<TextColor> {
        self.color
    }

    pub fn get_shadow_color(&self) -> Option<i32> {
        self.shadow_color
    }

    pub fn is_bold(&self) -> bool {
        self.bold == Some(true)
    }

    pub fn is_italic(&self) -> bool {
        self.italic == Some(true)
    }

    pub fn is_strikethrough(&self) -> bool {
        self.strikethrough == Some(true)
    }

    pub fn is_underlined(&self) -> bool {
        self.underlined == Some(true)
    }

    pub fn is_obfuscated(&self) -> bool {
        self.obfuscated == Some(true)
    }

    /// Whether this is `EMPTY` itself, not a style that equals it.
    pub fn is_empty(&self) -> bool {
        self.is_empty_constant
    }

    pub fn get_click_event(&self) -> Option<&ClickEvent> {
        self.click_event.as_ref()
    }

    pub fn get_hover_event(&self) -> Option<&HoverEvent> {
        self.hover_event.as_ref()
    }

    pub fn get_insertion(&self) -> Option<&str> {
        self.insertion.as_deref()
    }

    /// The font, or `FontDescription.DEFAULT`.
    pub fn get_font(&self) -> FontDescription {
        self.font
            .clone()
            .unwrap_or_else(FontDescription::default_font)
    }

    // `checkEmptyAfterChange`: `EMPTY` when a field was cleared and the style became equal to it.
    fn check_empty_after_change(new_style: Style, previous_set: bool, next_set: bool) -> Style {
        if previous_set && !next_set && new_style == Style::EMPTY {
            Style::EMPTY
        } else {
            new_style
        }
    }

    pub fn with_color(&self, color: Option<TextColor>) -> Style {
        if self.color == color {
            return self.clone();
        }
        let mut new_style = self.clone();
        new_style.is_empty_constant = false;
        new_style.color = color;
        Style::check_empty_after_change(new_style, self.color.is_some(), color.is_some())
    }

    /// `withColor(int)`.
    pub fn with_color_rgb(&self, color: i32) -> Style {
        self.with_color(Some(TextColor::from_rgb(color)))
    }

    pub fn with_shadow_color(&self, shadow_color: i32) -> Style {
        if self.shadow_color == Some(shadow_color) {
            return self.clone();
        }
        let mut new_style = self.clone();
        new_style.is_empty_constant = false;
        new_style.shadow_color = Some(shadow_color);
        Style::check_empty_after_change(new_style, self.shadow_color.is_some(), true)
    }

    /// `withBold(Boolean)`.
    pub fn with_bold(&self, bold: Option<bool>) -> Style {
        if self.bold == bold {
            return self.clone();
        }
        let mut new_style = self.clone();
        new_style.is_empty_constant = false;
        new_style.bold = bold;
        Style::check_empty_after_change(new_style, self.bold.is_some(), bold.is_some())
    }

    /// `withFont(FontDescription)`.
    pub fn with_font(&self, font: Option<FontDescription>) -> Style {
        if self.font == font {
            return self.clone();
        }
        let had_font = self.font.is_some();
        let has_font = font.is_some();
        let mut new_style = self.clone();
        new_style.is_empty_constant = false;
        new_style.font = font;
        Style::check_empty_after_change(new_style, had_font, has_font)
    }

    /// `withItalic(Boolean)`.
    pub fn with_italic(&self, italic: Option<bool>) -> Style {
        if self.italic == italic {
            return self.clone();
        }
        let mut new_style = self.clone();
        new_style.is_empty_constant = false;
        new_style.italic = italic;
        Style::check_empty_after_change(new_style, self.italic.is_some(), italic.is_some())
    }

    /// `withColor(ChatFormatting)`.
    pub fn with_color_formatting(&self, color: ChatFormatting) -> Style {
        self.with_color(TextColor::from_legacy_format(color))
    }

    pub fn without_shadow(&self) -> Style {
        self.with_shadow_color(Style::NO_SHADOW)
    }

    /// `applyLegacyFormat`: a `§` code's change. A flag sets its flag; a color clears the flags
    /// and sets the color; reset is `EMPTY`.
    pub fn apply_legacy_format(&self, format: ChatFormatting) -> Style {
        let mut color = self.color;
        let mut bold = self.bold;
        let mut italic = self.italic;
        let mut strikethrough = self.strikethrough;
        let mut underlined = self.underlined;
        let mut obfuscated = self.obfuscated;
        match format {
            ChatFormatting::OBFUSCATED => obfuscated = Some(true),
            ChatFormatting::BOLD => bold = Some(true),
            ChatFormatting::STRIKETHROUGH => strikethrough = Some(true),
            ChatFormatting::UNDERLINE => underlined = Some(true),
            ChatFormatting::ITALIC => italic = Some(true),
            ChatFormatting::RESET => return Style::EMPTY,
            _ => {
                obfuscated = Some(false);
                bold = Some(false);
                strikethrough = Some(false);
                underlined = Some(false);
                italic = Some(false);
                color = TextColor::from_legacy_format(format);
            }
        }
        Style::new(
            color,
            self.shadow_color,
            bold,
            italic,
            underlined,
            strikethrough,
            obfuscated,
            self.click_event.clone(),
            self.hover_event.clone(),
            self.insertion.clone(),
            self.font.clone(),
        )
    }

    /// This style's fields where they're set, the other's where they aren't; either one when the
    /// other is `EMPTY`.
    pub fn apply_to(&self, other: &Style) -> Style {
        if self.is_empty_constant {
            return other.clone();
        }
        if other.is_empty_constant {
            return self.clone();
        }
        Style::new(
            self.color.or(other.color),
            self.shadow_color.or(other.shadow_color),
            self.bold.or(other.bold),
            self.italic.or(other.italic),
            self.underlined.or(other.underlined),
            self.strikethrough.or(other.strikethrough),
            self.obfuscated.or(other.obfuscated),
            self.click_event
                .clone()
                .or_else(|| other.click_event.clone()),
            self.hover_event
                .clone()
                .or_else(|| other.hover_event.clone()),
            self.insertion.clone().or_else(|| other.insertion.clone()),
            self.font.clone().or_else(|| other.font.clone()),
        )
    }
}

/// Every field, whether it's `EMPTY` itself or not.
impl PartialEq for Style {
    fn eq(&self, style: &Style) -> bool {
        self.bold == style.bold
            && self.color == style.color
            && self.shadow_color == style.shadow_color
            && self.italic == style.italic
            && self.obfuscated == style.obfuscated
            && self.strikethrough == style.strikethrough
            && self.underlined == style.underlined
            && self.click_event == style.click_event
            && self.hover_event == style.hover_event
            && self.insertion == style.insertion
            && self.font == style.font
    }
}

/// `{color=red,!bold,…}`: the set fields in order, a flag as its name (`!` when false).
impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fields: Vec<String> = Vec::new();
        let flag = |fields: &mut Vec<String>, name: &str, value: Option<bool>| {
            if let Some(value) = value {
                fields.push(format!("{}{name}", if value { "" } else { "!" }));
            }
        };
        if let Some(color) = &self.color {
            fields.push(format!("color={color}"));
        }
        if let Some(shadow_color) = &self.shadow_color {
            fields.push(format!("shadowColor={shadow_color}"));
        }
        flag(&mut fields, "bold", self.bold);
        flag(&mut fields, "italic", self.italic);
        flag(&mut fields, "underlined", self.underlined);
        flag(&mut fields, "strikethrough", self.strikethrough);
        flag(&mut fields, "obfuscated", self.obfuscated);
        if let Some(click_event) = &self.click_event {
            fields.push(format!("clickEvent={click_event}"));
        }
        if let Some(hover_event) = &self.hover_event {
            fields.push(format!("hoverEvent={hover_event}"));
        }
        if let Some(insertion) = &self.insertion {
            fields.push(format!("insertion={insertion}"));
        }
        if let Some(font) = &self.font {
            fields.push(format!("font={font}"));
        }
        write!(f, "{{{}}}", fields.join(","))
    }
}

// `Style.Serializer.MAP_CODEC`: each field optional.
static MAP_CODEC: StaticMapCodec<Style> = LazyLock::new(|| {
    record_codec_builder::map_codec(
        (
            TextColor::CODEC
                .shared()
                .optional_field_of("color")
                .for_getter(|o: &Style| o.color),
            ExtraCodecs::ARGB_COLOR_CODEC
                .shared()
                .optional_field_of("shadow_color")
                .for_getter(|o: &Style| o.shadow_color),
            BOOL.optional_field_of("bold")
                .for_getter(|o: &Style| o.bold),
            BOOL.optional_field_of("italic")
                .for_getter(|o: &Style| o.italic),
            BOOL.optional_field_of("underlined")
                .for_getter(|o: &Style| o.underlined),
            BOOL.optional_field_of("strikethrough")
                .for_getter(|o: &Style| o.strikethrough),
            BOOL.optional_field_of("obfuscated")
                .for_getter(|o: &Style| o.obfuscated),
            ClickEvent::CODEC
                .shared()
                .optional_field_of("click_event")
                .for_getter(|o: &Style| o.click_event.clone()),
            HoverEvent::CODEC
                .shared()
                .optional_field_of("hover_event")
                .for_getter(|o: &Style| o.hover_event.clone()),
            STRING
                .optional_field_of("insertion")
                .for_getter(|o: &Style| o.insertion.clone()),
            FontDescription::CODEC
                .shared()
                .optional_field_of("font")
                .for_getter(|o: &Style| o.font.clone()),
        ),
        Style::create,
    )
    .boxed()
});

static CODEC: StaticCodec<Style> = LazyLock::new(|| MAP_CODEC.shared().codec().boxed());

/// `Style.Serializer`.
pub struct Serializer;

impl Serializer {
    pub const MAP_CODEC: &'static StaticMapCodec<Style> = &MAP_CODEC;
    pub const CODEC: &'static StaticCodec<Style> = &CODEC;
}
