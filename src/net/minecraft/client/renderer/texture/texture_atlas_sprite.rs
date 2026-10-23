//! net/minecraft/client/renderer/texture/TextureAtlasSprite.java
//!
//! A sprite placed in an atlas: where, its padding, and its UVs. `uploadSpriteUbo` writes the
//! `SpriteAnimationInfo` blocks `TextureAtlas` draws it into the atlas with.

use crate::com::mojang::blaze3d::buffers::std140_builder::Std140Builder;
use crate::com::mojang::blaze3d::platform::transparency::Transparency;
use crate::net::minecraft::client::renderer::texture::sprite_contents::SpriteContents;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::org::joml::matrix4f::Matrix4f;

pub struct TextureAtlasSprite {
    atlas_location: Identifier,
    contents: SpriteContents,
    x: i32,
    y: i32,
    u0: f32,
    u1: f32,
    v0: f32,
    v1: f32,
    padding: i32,
}

impl TextureAtlasSprite {
    pub fn new(
        atlas_location: Identifier,
        contents: SpriteContents,
        atlas_width: i32,
        atlas_height: i32,
        x: i32,
        y: i32,
        padding: i32,
    ) -> TextureAtlasSprite {
        let u0 = (x + padding) as f32 / atlas_width as f32;
        let u1 = (x + padding + contents.width()) as f32 / atlas_width as f32;
        let v0 = (y + padding) as f32 / atlas_height as f32;
        let v1 = (y + padding + contents.height()) as f32 / atlas_height as f32;
        TextureAtlasSprite {
            atlas_location,
            contents,
            x,
            y,
            u0,
            u1,
            v0,
            v1,
            padding,
        }
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn padding(&self) -> i32 {
        self.padding
    }

    pub fn get_u0(&self) -> f32 {
        self.u0
    }

    pub fn get_u1(&self) -> f32 {
        self.u1
    }

    pub fn get_v0(&self) -> f32 {
        self.v0
    }

    pub fn get_v1(&self) -> f32 {
        self.v1
    }

    pub fn contents(&self) -> &SpriteContents {
        &self.contents
    }

    pub fn contents_mut(&mut self) -> &mut SpriteContents {
        &mut self.contents
    }

    pub fn transparency(&self) -> Transparency {
        self.contents.transparency()
    }

    pub fn get_u(&self, offset: f32) -> f32 {
        let diff = self.u1 - self.u0;
        self.u0 + diff * offset
    }

    pub fn get_v(&self, offset: f32) -> f32 {
        let diff = self.v1 - self.v0;
        self.v0 + diff * offset
    }

    pub fn atlas_location(&self) -> &Identifier {
        &self.atlas_location
    }

    pub fn is_animated(&self) -> bool {
        self.contents.is_animated()
    }

    /// Each mip level's block, `sprite_ubo_size` apart from `start_offset`: the atlas level's
    /// projection, the padded sprite's place in it, the padding as a fraction of the sprite, and
    /// the level.
    pub fn upload_sprite_ubo(
        &self,
        ubo_buffer: &mut [u8],
        start_offset: i32,
        max_mip_level: i32,
        atlas_width: i32,
        atlas_height: i32,
        sprite_ubo_size: i32,
    ) {
        for level in 0..=max_mip_level {
            let mut projection = Matrix4f::new();
            projection.ortho2_d(
                0.0,
                (atlas_width >> level) as f32,
                0.0,
                (atlas_height >> level) as f32,
            );
            let mut sprite = Matrix4f::new();
            sprite
                .translate((self.x >> level) as f32, (self.y >> level) as f32, 0.0)
                .scale(
                    ((self.contents.width() + self.padding * 2) >> level) as f32,
                    ((self.contents.height() + self.padding * 2) >> level) as f32,
                    1.0,
                );
            let mut builder = Std140Builder::on_stack(sprite_ubo_size);
            builder
                .put_mat4f(&projection)
                .put_mat4f(&sprite)
                .put_float(self.padding as f32 / self.contents.width() as f32)
                .put_float(self.padding as f32 / self.contents.height() as f32)
                .put_int(level);
            let bytes = builder.get();
            let start = (start_offset + level * sprite_ubo_size) as usize;
            ubo_buffer[start..start + bytes.len()].copy_from_slice(&bytes);
        }
    }
}
