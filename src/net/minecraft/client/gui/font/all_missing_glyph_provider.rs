//! net/minecraft/client/gui/font/AllMissingGlyphProvider.java
//!
//! The fallback every font ends with: the missing glyph for any codepoint, supporting none.

use std::rc::Rc;

use crate::com::mojang::blaze3d::font::glyph_info::GlyphInfo;
use crate::com::mojang::blaze3d::font::glyph_provider::GlyphProvider;
use crate::com::mojang::blaze3d::font::unbaked_glyph::{Stitcher, UnbakedGlyph};
use crate::net::minecraft::client::gui::font::glyphs::baked_glyph::BakedGlyph;
use crate::net::minecraft::client::gui::font::glyphs::special_glyphs::SpecialGlyphs;

pub struct AllMissingGlyphProvider {
    missing_instance: Rc<dyn UnbakedGlyph>,
}

/// `MISSING_INSTANCE`.
struct MissingGlyph;

impl UnbakedGlyph for MissingGlyph {
    fn info(&self) -> GlyphInfo {
        GlyphInfo::Special(SpecialGlyphs::MISSING)
    }

    fn bake(&self, stitcher: &mut dyn Stitcher) -> Rc<BakedGlyph> {
        stitcher.get_missing()
    }
}

impl Default for AllMissingGlyphProvider {
    fn default() -> AllMissingGlyphProvider {
        AllMissingGlyphProvider::new()
    }
}

impl AllMissingGlyphProvider {
    pub fn new() -> AllMissingGlyphProvider {
        // One instance, shared like Java's static.
        thread_local! {
            static MISSING_INSTANCE: Rc<dyn UnbakedGlyph> = Rc::new(MissingGlyph);
        }
        AllMissingGlyphProvider {
            missing_instance: MISSING_INSTANCE.with(Rc::clone),
        }
    }
}

impl GlyphProvider for AllMissingGlyphProvider {
    fn get_glyph(&self, _codepoint: i32) -> Option<Rc<dyn UnbakedGlyph>> {
        Some(Rc::clone(&self.missing_instance))
    }

    fn get_supported_glyphs(&self) -> Vec<i32> {
        Vec::new()
    }
}
