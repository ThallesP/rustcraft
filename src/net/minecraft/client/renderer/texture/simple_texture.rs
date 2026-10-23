//! net/minecraft/client/renderer/texture/SimpleTexture.java

use crate::com::mojang::blaze3d::systems::sampler_cache::SamplerCache;
use crate::net::minecraft::client::renderer::texture::reloadable_texture::ReloadableTexture;
use crate::net::minecraft::client::renderer::texture::texture_contents::TextureContents;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::resources::resource_manager::ResourceManager;

/// A texture that's a resource's image as it is.
pub struct SimpleTexture {
    pub(crate) reloadable: ReloadableTexture,
}

impl SimpleTexture {
    pub fn new(location: Identifier, sampler_cache: &SamplerCache) -> SimpleTexture {
        SimpleTexture {
            reloadable: ReloadableTexture::new(location, sampler_cache),
        }
    }

    /// An error where Java throws `IOException`.
    pub fn load_contents(
        &self,
        resource_manager: &dyn ResourceManager,
    ) -> Result<TextureContents, String> {
        TextureContents::load(resource_manager, self.reloadable.resource_id())
    }
}
