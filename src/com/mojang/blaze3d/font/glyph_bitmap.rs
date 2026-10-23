//! com/mojang/blaze3d/font/GlyphBitmap.java
//!
//! A glyph's pixels, which a font texture uploads where it stitched them, and where they're drawn
//! from the glyph's origin: from its bearings, at its oversample (pixels per GUI pixel).

use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::textures::gpu_texture::GpuTexture;

pub trait GlyphBitmap {
    fn get_pixel_width(&self) -> i32;

    fn get_pixel_height(&self) -> i32;

    /// `upload(int, int, GpuTexture)`, on the device Java reads from `RenderSystem`.
    fn upload(&self, device: &dyn GpuDevice, x: i32, y: i32, texture: &dyn GpuTexture);

    fn is_colored(&self) -> bool;

    fn get_oversample(&self) -> f32;

    fn get_left(&self) -> f32 {
        self.get_bearing_left()
    }

    fn get_right(&self) -> f32 {
        self.get_left() + self.get_pixel_width() as f32 / self.get_oversample()
    }

    fn get_top(&self) -> f32 {
        7.0 - self.get_bearing_top()
    }

    fn get_bottom(&self) -> f32 {
        self.get_top() + self.get_pixel_height() as f32 / self.get_oversample()
    }

    fn get_bearing_left(&self) -> f32 {
        0.0
    }

    fn get_bearing_top(&self) -> f32 {
        7.0
    }
}
