//! com/mojang/blaze3d/font/GlyphInfo.java
//!
//! A glyph's advance, and how far its bold copy and its shadow are drawn off it. Java's
//! implementations are `simple(advance)`'s lambda and `SpecialGlyphs`' constants (the missing and
//! white glyphs, whose advance is their image's width and one), which `FontSet` tells apart by
//! reference; here they're an enum's variants, compared by value. The TrueType provider's comes
//! with it.

use crate::net::minecraft::client::gui::font::glyphs::special_glyphs::SpecialGlyphs;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GlyphInfo {
    Simple(f32),
    Special(SpecialGlyphs),
}

impl GlyphInfo {
    pub fn simple(advance: f32) -> GlyphInfo {
        GlyphInfo::Simple(advance)
    }

    pub fn get_advance(&self) -> f32 {
        match self {
            GlyphInfo::Simple(advance) => *advance,
            GlyphInfo::Special(special) => special.get_advance(),
        }
    }

    /// `getAdvance(boolean)`: one more when bold.
    pub fn get_advance_bold(&self, bold: bool) -> f32 {
        self.get_advance() + if bold { self.get_bold_offset() } else { 0.0 }
    }

    pub fn get_bold_offset(&self) -> f32 {
        1.0
    }

    pub fn get_shadow_offset(&self) -> f32 {
        1.0
    }
}
