//! net/minecraft/client/gui/font/glyphs/BakedGlyph.java
//!
//! A glyph ready to draw: its metrics, and the instance it makes at a place, or none for one that
//! draws nothing. Java's implementations are `BakedSheetGlyph`, `EmptyGlyph`'s anonymous class
//! and `FontSet.INVISIBLE_MISSING_GLYPH`, which are this enum's variants.

use std::rc::Rc;

use crate::com::mojang::blaze3d::font::glyph_info::GlyphInfo;
use crate::net::minecraft::client::gui::font::glyphs::baked_sheet_glyph::{
    BakedSheetGlyph, GlyphInstance,
};
use crate::net::minecraft::client::gui::font::glyphs::special_glyphs::SpecialGlyphs;
use crate::net::minecraft::client::gui::font::text_renderable::TextRenderable;
use crate::net::minecraft::network::chat::style::Style;

pub enum BakedGlyph {
    Sheet(Rc<BakedSheetGlyph>),
    Empty(GlyphInfo),
    /// `FontSet.INVISIBLE_MISSING_GLYPH`: the missing glyph's metrics, drawing nothing, until a
    /// font set has baked its missing glyph.
    InvisibleMissing,
}

impl BakedGlyph {
    pub fn info(&self) -> GlyphInfo {
        match self {
            BakedGlyph::Sheet(glyph) => glyph.info(),
            BakedGlyph::Empty(info) => *info,
            BakedGlyph::InvisibleMissing => GlyphInfo::Special(SpecialGlyphs::MISSING),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_glyph(
        &self,
        x: f32,
        y: f32,
        color: i32,
        shadow_color: i32,
        style: &Style,
        bold_offset: f32,
        shadow_offset: f32,
    ) -> Option<TextRenderable> {
        match self {
            BakedGlyph::Sheet(glyph) => Some(TextRenderable::Glyph(GlyphInstance {
                x,
                y,
                color,
                shadow_color,
                glyph: Rc::clone(glyph),
                style: style.clone(),
                bold_offset,
                shadow_offset,
            })),
            BakedGlyph::Empty(_) | BakedGlyph::InvisibleMissing => None,
        }
    }
}
