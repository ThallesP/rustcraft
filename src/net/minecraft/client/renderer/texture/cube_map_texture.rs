//! net/minecraft/client/renderer/texture/CubeMapTexture.java
//!
//! A cube map from six images beside its location (`_0.png` to `_5.png`), stacked into one image
//! a face under the other, each flipped upside down, in the faces' order (`SUFFIXES`), and copied
//! face by face into a cube texture. Java stages the image in the encoder's transient memory; the
//! port copies it from a buffer of its own, the same bytes.

use crate::com::mojang::blaze3d::platform::native_image::NativeImage;
use crate::com::mojang::blaze3d::systems::sampler_cache::SamplerCache;
use crate::com::mojang::renderpearl::api::buffers::gpu_buffer;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use crate::com::mojang::renderpearl::api::textures::gpu_texture;
use crate::net::minecraft::client::renderer::texture::mipmap_strategy::MipmapStrategy;
use crate::net::minecraft::client::renderer::texture::reloadable_texture::ReloadableTexture;
use crate::net::minecraft::client::renderer::texture::texture_contents::TextureContents;
use crate::net::minecraft::client::resources::metadata::texture::texture_metadata_section::TextureMetadataSection;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::resources::resource_manager::ResourceManager;

pub struct CubeMapTexture {
    pub(crate) reloadable: ReloadableTexture,
}

impl CubeMapTexture {
    const SUFFIXES: [&'static str; 6] =
        ["_1.png", "_3.png", "_5.png", "_4.png", "_0.png", "_2.png"];

    pub fn new(resource_id: Identifier, sampler_cache: &SamplerCache) -> CubeMapTexture {
        CubeMapTexture {
            reloadable: ReloadableTexture::new(resource_id, sampler_cache),
        }
    }

    /// `loadContents`: the six faces stacked, blurred and repeating. An error where Java throws
    /// `IOException`, for a face that's missing or of another size than the first.
    pub fn load_contents(
        &self,
        resource_manager: &dyn ResourceManager,
    ) -> Result<TextureContents, String> {
        let location = self.reloadable.resource_id();
        let first = TextureContents::load(
            resource_manager,
            &location.with_suffix(CubeMapTexture::SUFFIXES[0]),
        )?;
        let width = first.image.get_width();
        let height = first.image.get_height();
        let mut stacked_image = NativeImage::new(width, height * 6);
        first
            .image
            .copy_rect_to(&mut stacked_image, 0, 0, 0, 0, width, height, false, true);
        for (i, suffix) in CubeMapTexture::SUFFIXES.iter().enumerate().skip(1) {
            let part = TextureContents::load(resource_manager, &location.with_suffix(suffix))?;
            if part.image.get_width() != width || part.image.get_height() != height {
                return Err(format!(
                    "Image dimensions of cubemap '{location}' sides do not match: part 0 is \
                     {width}x{height}, but part {i} is {}x{}",
                    part.image.get_width(),
                    part.image.get_height()
                ));
            }
            part.image.copy_rect_to(
                &mut stacked_image,
                0,
                0,
                0,
                i as i32 * height,
                width,
                height,
                false,
                true,
            );
        }
        Ok(TextureContents {
            image: stacked_image,
            metadata: Some(TextureMetadataSection {
                blur: true,
                clamp: false,
                mipmap_strategy: MipmapStrategy::MEAN,
                alpha_cutoff_bias: 0.0,
            }),
        })
    }

    /// `ReloadableTexture.apply`, with this class's `doLoad`: a cube texture of six layers, each
    /// face copied from its part of the stacked image.
    pub fn apply(
        &mut self,
        device: &dyn GpuDevice,
        sampler_cache: &SamplerCache,
        contents: TextureContents,
    ) {
        self.reloadable.set_sampler(sampler_cache, &contents);
        let image = &contents.image;
        let width = image.get_width();
        let height = image.get_height() / 6;
        self.reloadable.base.release_textures();
        let texture = device.create_texture(
            Some(&self.reloadable.resource_id().to_string()),
            gpu_texture::USAGE_COPY_DST
                | gpu_texture::USAGE_TEXTURE_BINDING
                | gpu_texture::USAGE_CUBEMAP_COMPATIBLE,
            GpuFormat::RGBA8_UNORM,
            width,
            height,
            6,
            1,
        );
        self.reloadable.base.texture_view = Some(device.create_texture_view(&texture));
        let staging_buffer = device.create_buffer_with_data(
            Some("Cube map staging buffer"),
            gpu_buffer::USAGE_COPY_SRC,
            &image.get_pixel_bytes(),
        );
        for i in 0..6 {
            device.create_command_encoder().copy_buffer_to_texture(
                &gpu_buffer::slice(&staging_buffer),
                0,
                height * i,
                image.get_width(),
                image.get_height(),
                &*texture,
                0,
                0,
                width,
                height,
                0,
                i,
            );
        }
        self.reloadable.base.texture = Some(texture);
    }
}
