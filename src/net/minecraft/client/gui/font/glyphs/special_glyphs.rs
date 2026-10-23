//! net/minecraft/client/gui/font/glyphs/SpecialGlyphs.java
//!
//! The glyphs every font set has: `MISSING`, a 5 by 8 outline, and `WHITE`, a 5 by 8 white block
//! that underlines, strikethroughs and backgrounds are drawn with. Their images are made when
//! asked for, where Java's enum makes them once; they're the same pixels.

use std::rc::Rc;

use crate::com::mojang::blaze3d::font::glyph_bitmap::GlyphBitmap;
use crate::com::mojang::blaze3d::font::glyph_info::GlyphInfo;
use crate::com::mojang::blaze3d::platform::native_image::NativeImage;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::textures::gpu_texture::GpuTexture;
use crate::net::minecraft::client::gui::font::glyph_stitcher::GlyphStitcher;
use crate::net::minecraft::client::gui::font::glyphs::baked_sheet_glyph::BakedSheetGlyph;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpecialGlyphs {
    WHITE,
    MISSING,
}

impl SpecialGlyphs {
    const WIDTH: i32 = 5;
    const HEIGHT: i32 = 8;

    pub fn image(self) -> NativeImage {
        let mut result = NativeImage::new(SpecialGlyphs::WIDTH, SpecialGlyphs::HEIGHT);
        for y in 0..SpecialGlyphs::HEIGHT {
            for x in 0..SpecialGlyphs::WIDTH {
                let color = match self {
                    SpecialGlyphs::WHITE => -1,
                    SpecialGlyphs::MISSING => {
                        let edge = x == 0
                            || x + 1 == SpecialGlyphs::WIDTH
                            || y == 0
                            || y + 1 == SpecialGlyphs::HEIGHT;
                        if edge { -1 } else { 0 }
                    }
                };
                result.set_pixel(x, y, color);
            }
        }
        result
    }

    /// `getAdvance()`: the image's width and one.
    pub fn get_advance(self) -> f32 {
        (SpecialGlyphs::WIDTH + 1) as f32
    }

    pub fn bake(self, stitcher: &mut GlyphStitcher) -> Option<Rc<BakedSheetGlyph>> {
        stitcher.stitch(
            GlyphInfo::Special(self),
            &SpecialBitmap {
                image: self.image(),
            },
        )
    }
}

/// The glyph's bitmap: its image, written to the texture whole.
struct SpecialBitmap {
    image: NativeImage,
}

impl GlyphBitmap for SpecialBitmap {
    fn get_pixel_width(&self) -> i32 {
        self.image.get_width()
    }

    fn get_pixel_height(&self) -> i32 {
        self.image.get_height()
    }

    fn get_oversample(&self) -> f32 {
        1.0
    }

    fn upload(&self, device: &dyn GpuDevice, x: i32, y: i32, texture: &dyn GpuTexture) {
        device
            .create_command_encoder()
            .write_to_texture_image(texture, &self.image, 0, 0, x, y);
    }

    fn is_colored(&self) -> bool {
        true
    }
}
