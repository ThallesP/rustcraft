//! net/minecraft/client/renderer/texture/TextureContents.java

use crate::com::mojang::blaze3d::platform::native_image::NativeImage;
use crate::net::minecraft::client::renderer::texture::missing_texture_atlas_sprite::MissingTextureAtlasSprite;
use crate::net::minecraft::client::resources::metadata::texture::texture_metadata_section::TextureMetadataSection;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::resources::resource_manager::ResourceManager;

/// A texture's image and its `.mcmeta`'s texture section, if it has one.
pub struct TextureContents {
    pub image: NativeImage,
    pub metadata: Option<TextureMetadataSection>,
}

impl TextureContents {
    /// An error where Java throws `IOException`: a texture that's missing, isn't an image, or
    /// whose metadata doesn't parse.
    pub fn load(
        resource_manager: &dyn ResourceManager,
        location: &Identifier,
    ) -> Result<TextureContents, String> {
        let resource = resource_manager
            .get_resource(location)
            .ok_or_else(|| location.to_string())?;
        let bytes = resource.open().map_err(|error| error.to_string())?;
        let image = NativeImage::read(&bytes)?;
        let metadata = resource
            .metadata()?
            .get_section(&TextureMetadataSection::TYPE)?;
        Ok(TextureContents { image, metadata })
    }

    pub fn create_missing() -> TextureContents {
        TextureContents {
            image: MissingTextureAtlasSprite::generate_missing_image(16, 16),
            metadata: None,
        }
    }

    pub fn blur(&self) -> bool {
        self.metadata.as_ref().is_some_and(|metadata| metadata.blur)
    }

    pub fn clamp(&self) -> bool {
        self.metadata
            .as_ref()
            .is_some_and(|metadata| metadata.clamp)
    }
}
