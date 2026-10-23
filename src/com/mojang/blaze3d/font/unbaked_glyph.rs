//! com/mojang/blaze3d/font/UnbakedGlyph.java
//!
//! A provider's glyph before it's in a font texture: its metrics, and how it bakes. `Stitcher` is
//! the font set's, which puts the bitmap in a texture.

use std::rc::Rc;

use crate::com::mojang::blaze3d::font::glyph_bitmap::GlyphBitmap;
use crate::com::mojang::blaze3d::font::glyph_info::GlyphInfo;
use crate::net::minecraft::client::gui::font::glyphs::baked_glyph::BakedGlyph;

pub trait UnbakedGlyph {
    fn info(&self) -> GlyphInfo;

    fn bake(&self, stitcher: &mut dyn Stitcher) -> Rc<BakedGlyph>;
}

/// `UnbakedGlyph.Stitcher`.
pub trait Stitcher {
    fn stitch(&mut self, info: GlyphInfo, glyph_bitmap: &dyn GlyphBitmap) -> Rc<BakedGlyph>;

    fn get_missing(&self) -> Rc<BakedGlyph>;
}
