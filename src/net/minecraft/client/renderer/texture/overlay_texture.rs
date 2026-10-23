//! net/minecraft/client/renderer/texture/OverlayTexture.java
//!
//! The 16 by 16 texture an entity's overlay coordinates read (`Sampler1`): its top half red (a
//! hurt entity), its bottom half white fading out to the right (a flashing one), at `(u, v)`.

use crate::com::mojang::blaze3d::systems::sampler_cache::SamplerCache;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::net::minecraft::client::renderer::texture::dynamic_texture::DynamicTexture;
use crate::net::minecraft::util::argb::ARGB;

pub struct OverlayTexture {
    texture: DynamicTexture,
}

impl OverlayTexture {
    const SIZE: i32 = 16;
    pub const NO_WHITE_U: i32 = 0;
    pub const RED_OVERLAY_V: i32 = 3;
    pub const WHITE_OVERLAY_V: i32 = 10;
    pub const NO_OVERLAY: i32 = OverlayTexture::pack(0, 10);

    pub fn new(device: &dyn GpuDevice, sampler_cache: &SamplerCache) -> OverlayTexture {
        let mut texture = DynamicTexture::new(
            device,
            sampler_cache,
            "Entity Color Overlay",
            OverlayTexture::SIZE,
            OverlayTexture::SIZE,
        );
        let pixels = texture.get_pixels();
        for y in 0..16 {
            for x in 0..16 {
                if y < 8 {
                    pixels.set_pixel(x, y, -1291911168);
                } else {
                    let a = ((1.0 - x as f32 / 15.0 * 0.75) * 255.0) as i32;
                    pixels.set_pixel(x, y, ARGB::white(a));
                }
            }
        }

        texture.upload(device);
        OverlayTexture { texture }
    }

    pub fn u(white_overlay_progress: f32) -> i32 {
        (white_overlay_progress * 15.0) as i32
    }

    pub fn v(hurt_overlay: bool) -> i32 {
        if hurt_overlay { 3 } else { 10 }
    }

    pub const fn pack(u: i32, v: i32) -> i32 {
        u | v << 16
    }

    /// `pack(float, boolean)`.
    pub fn pack_progress(white_overlay_progress: f32, red_overlay: bool) -> i32 {
        OverlayTexture::pack(
            OverlayTexture::u(white_overlay_progress),
            OverlayTexture::v(red_overlay),
        )
    }

    pub fn get_texture_view(&self) -> &GpuTextureView {
        self.texture.base().get_texture_view()
    }
}
