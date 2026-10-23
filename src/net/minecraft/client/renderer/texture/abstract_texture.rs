//! net/minecraft/client/renderer/texture/AbstractTexture.java
//!
//! A texture on the device, its view and the sampler it's drawn with. Java's kinds of texture
//! extend it; here they hold one (`SimpleTexture`, `DynamicTexture`). Closing it is dropping it.

use std::rc::Rc;

use crate::com::mojang::blaze3d::systems::sampler_cache::SamplerCache;
use crate::com::mojang::renderpearl::api::textures::address_mode::AddressMode;
use crate::com::mojang::renderpearl::api::textures::filter_mode::FilterMode;
use crate::com::mojang::renderpearl::api::textures::gpu_sampler::GpuSampler;
use crate::com::mojang::renderpearl::api::textures::gpu_texture::GpuTexture;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;

pub struct AbstractTexture {
    pub(crate) texture: Option<Rc<dyn GpuTexture>>,
    pub(crate) texture_view: Option<GpuTextureView>,
    pub(crate) sampler: Rc<dyn GpuSampler>,
}

impl AbstractTexture {
    /// No texture yet, and the default sampler: repeating, nearest when minified and linear when
    /// magnified.
    pub fn new(sampler_cache: &SamplerCache) -> AbstractTexture {
        AbstractTexture {
            texture: None,
            texture_view: None,
            sampler: Rc::clone(sampler_cache.get_sampler(
                AddressMode::REPEAT,
                AddressMode::REPEAT,
                FilterMode::NEAREST,
                FilterMode::LINEAR,
                false,
            )),
        }
    }

    pub(crate) fn release_textures(&mut self) {
        self.texture = None;
        self.texture_view = None;
    }

    /// Panics where Java throws `IllegalStateException`, before the texture is made.
    pub fn get_texture(&self) -> &Rc<dyn GpuTexture> {
        self.texture
            .as_ref()
            .expect("Texture does not exist, can't get it before something initializes it")
    }

    /// Panics where Java throws `IllegalStateException`, before the view is made.
    pub fn get_texture_view(&self) -> &GpuTextureView {
        self.texture_view
            .as_ref()
            .expect("Texture view does not exist, can't get it before something initializes it")
    }

    pub fn get_sampler(&self) -> &Rc<dyn GpuSampler> {
        &self.sampler
    }
}
