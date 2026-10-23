//! net/minecraft/client/renderer/texture/MissingTextureAtlasSprite.java

use crate::com::mojang::blaze3d::platform::native_image::NativeImage;
use crate::net::minecraft::client::renderer::texture::sprite_contents::SpriteContents;
use crate::net::minecraft::client::resources::metadata::animation::frame_size::FrameSize;
use crate::net::minecraft::resources::identifier::Identifier;

pub struct MissingTextureAtlasSprite;

impl MissingTextureAtlasSprite {
    const MISSING_IMAGE_WIDTH: i32 = 16;
    const MISSING_IMAGE_HEIGHT: i32 = 16;

    /// Magenta and black quarters.
    pub fn generate_missing_image(width: i32, height: i32) -> NativeImage {
        let mut result = NativeImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                if (y < height / 2) ^ (x < width / 2) {
                    result.set_pixel(x, y, -524040);
                } else {
                    result.set_pixel(x, y, -16777216);
                }
            }
        }
        result
    }

    pub fn create() -> SpriteContents {
        let contents = MissingTextureAtlasSprite::generate_missing_image(
            MissingTextureAtlasSprite::MISSING_IMAGE_WIDTH,
            MissingTextureAtlasSprite::MISSING_IMAGE_HEIGHT,
        );
        SpriteContents::new(
            MissingTextureAtlasSprite::get_location(),
            FrameSize {
                width: 16,
                height: 16,
            },
            contents,
            None,
            Vec::new(),
            None,
        )
    }

    pub fn get_location() -> Identifier {
        Identifier::with_default_namespace("missingno")
    }
}
