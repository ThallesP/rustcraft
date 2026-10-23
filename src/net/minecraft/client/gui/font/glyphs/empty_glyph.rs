//! net/minecraft/client/gui/font/glyphs/EmptyGlyph.java
//!
//! A glyph with an advance and nothing to draw (`SpaceProvider`'s).

use std::rc::Rc;

use crate::com::mojang::blaze3d::font::glyph_info::GlyphInfo;
use crate::com::mojang::blaze3d::font::unbaked_glyph::{Stitcher, UnbakedGlyph};
use crate::net::minecraft::client::gui::font::glyphs::baked_glyph::BakedGlyph;

pub struct EmptyGlyph {
    info: GlyphInfo,
}

impl EmptyGlyph {
    pub fn new(advance: f32) -> EmptyGlyph {
        EmptyGlyph {
            info: GlyphInfo::simple(advance),
        }
    }
}

impl UnbakedGlyph for EmptyGlyph {
    fn info(&self) -> GlyphInfo {
        self.info
    }

    fn bake(&self, _stitcher: &mut dyn Stitcher) -> Rc<BakedGlyph> {
        Rc::new(BakedGlyph::Empty(self.info))
    }
}
