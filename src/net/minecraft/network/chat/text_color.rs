//! net/minecraft/network/chat/TextColor.java
//!
//! A text's color: one of the sixteen named ones, or any RGB, and a legacy color code's
//! (`fromLegacyFormat`). Java's `equals` compares the value only, so a named color equals the same
//! RGB made with `fromRgb`, and `hashCode` hashes the name too.

use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::serialization::codec::{CodecExt, STRING, StaticCodec};
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::lifecycle::Lifecycle;
use crate::java::lang::integer::Integer;
use crate::java::lang::string::JavaString;
use crate::net::minecraft::chat_formatting::ChatFormatting;

#[derive(Clone, Copy, Debug)]
pub struct TextColor {
    value: i32,
    name: Option<&'static str>,
}

// `NAMED_COLORS`, in the order Java puts them.
const NAMED_COLORS: [TextColor; 16] = [
    TextColor::BLACK,
    TextColor::DARK_BLUE,
    TextColor::DARK_GREEN,
    TextColor::DARK_AQUA,
    TextColor::DARK_RED,
    TextColor::DARK_PURPLE,
    TextColor::GOLD,
    TextColor::GRAY,
    TextColor::DARK_GRAY,
    TextColor::BLUE,
    TextColor::GREEN,
    TextColor::AQUA,
    TextColor::RED,
    TextColor::LIGHT_PURPLE,
    TextColor::YELLOW,
    TextColor::WHITE,
];

// `Codec.STRING.comapFlatMap(TextColor::parseColor, TextColor::serialize)`.
static CODEC: StaticCodec<TextColor> = LazyLock::new(|| {
    STRING
        .comap_flat_map(
            |color: String| TextColor::parse_color(&color),
            TextColor::serialize,
        )
        .boxed()
});

impl TextColor {
    pub const CODEC: &'static StaticCodec<TextColor> = &CODEC;
    pub const BLACK: TextColor = TextColor::named("black", 0);
    pub const DARK_BLUE: TextColor = TextColor::named("dark_blue", 170);
    pub const DARK_GREEN: TextColor = TextColor::named("dark_green", 43520);
    pub const DARK_AQUA: TextColor = TextColor::named("dark_aqua", 43690);
    pub const DARK_RED: TextColor = TextColor::named("dark_red", 11141120);
    pub const DARK_PURPLE: TextColor = TextColor::named("dark_purple", 11141290);
    pub const GOLD: TextColor = TextColor::named("gold", 16755200);
    pub const GRAY: TextColor = TextColor::named("gray", 11184810);
    pub const DARK_GRAY: TextColor = TextColor::named("dark_gray", 5592405);
    pub const BLUE: TextColor = TextColor::named("blue", 5592575);
    pub const GREEN: TextColor = TextColor::named("green", 5635925);
    pub const AQUA: TextColor = TextColor::named("aqua", 5636095);
    pub const RED: TextColor = TextColor::named("red", 16733525);
    pub const LIGHT_PURPLE: TextColor = TextColor::named("light_purple", 16733695);
    pub const YELLOW: TextColor = TextColor::named("yellow", 16777045);
    pub const WHITE: TextColor = TextColor::named("white", 16777215);

    const fn named(name: &'static str, rgb: i32) -> TextColor {
        TextColor {
            value: rgb & 0xFFFFFF,
            name: Some(name),
        }
    }

    /// `fromLegacyFormat`: a color code's named color, or none for a flag or reset.
    pub fn from_legacy_format(format: ChatFormatting) -> Option<TextColor> {
        match format {
            ChatFormatting::BLACK => Some(TextColor::BLACK),
            ChatFormatting::DARK_BLUE => Some(TextColor::DARK_BLUE),
            ChatFormatting::DARK_GREEN => Some(TextColor::DARK_GREEN),
            ChatFormatting::DARK_AQUA => Some(TextColor::DARK_AQUA),
            ChatFormatting::DARK_RED => Some(TextColor::DARK_RED),
            ChatFormatting::DARK_PURPLE => Some(TextColor::DARK_PURPLE),
            ChatFormatting::GOLD => Some(TextColor::GOLD),
            ChatFormatting::GRAY => Some(TextColor::GRAY),
            ChatFormatting::DARK_GRAY => Some(TextColor::DARK_GRAY),
            ChatFormatting::BLUE => Some(TextColor::BLUE),
            ChatFormatting::GREEN => Some(TextColor::GREEN),
            ChatFormatting::AQUA => Some(TextColor::AQUA),
            ChatFormatting::RED => Some(TextColor::RED),
            ChatFormatting::LIGHT_PURPLE => Some(TextColor::LIGHT_PURPLE),
            ChatFormatting::YELLOW => Some(TextColor::YELLOW),
            ChatFormatting::WHITE => Some(TextColor::WHITE),
            _ => None,
        }
    }

    pub fn get_value(&self) -> i32 {
        self.value
    }

    /// The name, or `#RRGGBB` in upper-case hex.
    pub fn serialize(&self) -> String {
        match self.name {
            Some(name) => name.to_owned(),
            None => self.format_value(),
        }
    }

    fn format_value(&self) -> String {
        format!("#{:06X}", self.value)
    }

    pub fn from_rgb(rgb: i32) -> TextColor {
        TextColor {
            value: rgb & 0xFFFFFF,
            name: None,
        }
    }

    /// A `#` and hex digits (`Integer.parseInt(…, 16)`, a sign and all) from 0 to `FFFFFF`, or a
    /// named color's name; stable.
    pub fn parse_color(color: &str) -> DataResult<TextColor> {
        if let Some(hex) = color.strip_prefix('#') {
            match Integer::parse_int_radix(hex, 16) {
                Ok(value) if (0..=0xFFFFFF).contains(&value) => {
                    DataResult::success_lifecycle(TextColor::from_rgb(value), Lifecycle::stable())
                }
                Ok(_) => DataResult::error(|| format!("Color value out of range: {color}")),
                Err(_) => DataResult::error(|| format!("Invalid color value: {color}")),
            }
        } else {
            match NAMED_COLORS.iter().find(|named| named.name == Some(color)) {
                Some(named) => DataResult::success_lifecycle(*named, Lifecycle::stable()),
                None => DataResult::error(|| format!("Invalid color name: {color}")),
            }
        }
    }

    /// `Objects.hash(value, name)`.
    pub fn hash_code(&self) -> i32 {
        let name = self.name.map_or(0, JavaString::hash_code);
        31i32
            .wrapping_add(self.value)
            .wrapping_mul(31)
            .wrapping_add(name)
    }
}

impl PartialEq for TextColor {
    fn eq(&self, other: &TextColor) -> bool {
        self.value == other.value
    }
}

/// `serialize()`.
impl fmt::Display for TextColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.serialize())
    }
}
