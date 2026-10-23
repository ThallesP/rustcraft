//! net/minecraft/client/renderer/texture/DynamicTexture.java
//!
//! A texture of pixels the game sets, uploaded when it says. Not ported yet: `setPixels` and
//! `dumpContents`.

use std::rc::Rc;

use crate::com::mojang::blaze3d::platform::native_image::NativeImage;
use crate::com::mojang::blaze3d::systems::sampler_cache::SamplerCache;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use crate::com::mojang::renderpearl::api::textures::filter_mode::FilterMode;
use crate::com::mojang::renderpearl::api::textures::gpu_texture;
use crate::net::minecraft::client::renderer::texture::abstract_texture::AbstractTexture;

pub struct DynamicTexture {
    pub(crate) base: AbstractTexture,
    pixels: NativeImage,
}

impl DynamicTexture {
    /// `new DynamicTexture(label, width, height, zero)`: the texture made, its pixels not yet
    /// uploaded. The port's images start zeroed either way (`NativeImage::new`).
    pub fn new(
        device: &dyn GpuDevice,
        sampler_cache: &SamplerCache,
        label: &str,
        width: i32,
        height: i32,
    ) -> DynamicTexture {
        let pixels = NativeImage::new(width, height);
        let mut base = AbstractTexture::new(sampler_cache);
        let texture = device.create_texture(
            Some(label),
            gpu_texture::USAGE_COPY_DST | gpu_texture::USAGE_TEXTURE_BINDING,
            GpuFormat::RGBA8_UNORM,
            pixels.get_width(),
            pixels.get_height(),
            1,
            1,
        );
        base.sampler = Rc::clone(sampler_cache.get_repeat(FilterMode::NEAREST));
        base.texture_view = Some(device.create_texture_view(&texture));
        base.texture = Some(texture);
        DynamicTexture { base, pixels }
    }

    pub fn upload(&self, device: &dyn GpuDevice) {
        device
            .create_command_encoder()
            .write_to_texture_whole(&**self.base.get_texture(), &self.pixels);
    }

    pub fn get_pixels(&mut self) -> &mut NativeImage {
        &mut self.pixels
    }

    pub fn base(&self) -> &AbstractTexture {
        &self.base
    }
}
