//! com/mojang/blaze3d/font/GlyphProvider.java
//!
//! What a font's providers have in common: a glyph for a codepoint, or none, and the codepoints
//! they have glyphs for. A provider is shared by every font that references it, so it's an `Rc`.
//! `getSupportedGlyphs` is a fastutil `IntSet` in Java, whose order `FontSet` reads only to list
//! the glyphs of each width for obfuscated text; here it's the codepoints in order.

use std::rc::Rc;

use crate::com::mojang::blaze3d::font::unbaked_glyph::UnbakedGlyph;
use crate::net::minecraft::client::gui::font::font_option::Filter;

pub trait GlyphProvider {
    fn get_glyph(&self, codepoint: i32) -> Option<Rc<dyn UnbakedGlyph>>;

    fn get_supported_glyphs(&self) -> Vec<i32>;
}

/// `GlyphProvider.BASELINE`.
pub const BASELINE: f32 = 7.0;

/// `GlyphProvider.Conditional`: a provider, and the font options it's used with.
#[derive(Clone)]
pub struct Conditional {
    pub provider: Rc<dyn GlyphProvider>,
    pub filter: Filter,
}
