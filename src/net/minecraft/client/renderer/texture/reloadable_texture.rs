//! net/minecraft/client/renderer/texture/ReloadableTexture.java
//!
//! A texture loaded from a resource, which a resource reload loads again. Its kinds
//! (`SimpleTexture`) hold one.

use std::rc::Rc;

use crate::com::mojang::blaze3d::platform::native_image::NativeImage;
use crate::com::mojang::blaze3d::systems::sampler_cache::SamplerCache;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use crate::com::mojang::renderpearl::api::textures::address_mode::AddressMode;
use crate::com::mojang::renderpearl::api::textures::filter_mode::FilterMode;
use crate::com::mojang::renderpearl::api::textures::gpu_texture;
use crate::net::minecraft::client::renderer::texture::abstract_texture::AbstractTexture;
use crate::net::minecraft::client::renderer::texture::texture_contents::TextureContents;
use crate::net::minecraft::resources::identifier::Identifier;

pub struct ReloadableTexture {
    pub(crate) base: AbstractTexture,
    resource_id: Identifier,
}

impl ReloadableTexture {
    pub fn new(resource_id: Identifier, sampler_cache: &SamplerCache) -> ReloadableTexture {
        ReloadableTexture {
            base: AbstractTexture::new(sampler_cache),
            resource_id,
        }
    }

    pub fn resource_id(&self) -> &Identifier {
        &self.resource_id
    }

    /// The contents' sampler and image, on the device.
    pub fn apply(
        &mut self,
        device: &dyn GpuDevice,
        sampler_cache: &SamplerCache,
        contents: TextureContents,
    ) {
        self.set_sampler(sampler_cache, &contents);
        self.do_load(device, &contents.image);
    }

    fn set_sampler(&mut self, sampler_cache: &SamplerCache, contents: &TextureContents) {
        let address_mode = if contents.clamp() {
            AddressMode::CLAMP_TO_EDGE
        } else {
            AddressMode::REPEAT
        };
        let min_mag = if contents.blur() {
            FilterMode::LINEAR
        } else {
            FilterMode::NEAREST
        };
        self.base.sampler = Rc::clone(sampler_cache.get_sampler(
            address_mode,
            address_mode,
            min_mag,
            min_mag,
            false,
        ));
    }

    fn do_load(&mut self, device: &dyn GpuDevice, image: &NativeImage) {
        self.base.release_textures();
        let texture = device.create_texture(
            Some(&self.resource_id.to_string()),
            gpu_texture::USAGE_COPY_DST | gpu_texture::USAGE_TEXTURE_BINDING,
            GpuFormat::RGBA8_UNORM,
            image.get_width(),
            image.get_height(),
            1,
            1,
        );
        self.base.texture_view = Some(device.create_texture_view(&texture));
        device
            .create_command_encoder()
            .write_to_texture_whole(&*texture, image);
        self.base.texture = Some(texture);
    }
}
