//! net/minecraft/client/renderer/texture/SpriteLoader.java
//!
//! An atlas's sprites loaded from its definition and stitched: the mip level the smallest sprite
//! allows, the stitcher's layout, then every sprite's mip levels (Java's `readyForUpload`, run on
//! an executor; here at once). Java reads the anisotropy from `Minecraft.getInstance().options`,
//! which is the caller's argument here until the options come. Java's `regions` is a `HashMap`,
//! whose order only decides the order `TextureAtlas` draws sprites in, into places that don't
//! overlap; here it's the stitcher's order.

use std::collections::HashMap;
use std::sync::Arc;

use crate::net::minecraft::client::renderer::texture::atlas::sprite_source_list::SpriteSourceList;
use crate::net::minecraft::client::renderer::texture::missing_texture_atlas_sprite::MissingTextureAtlasSprite;
use crate::net::minecraft::client::renderer::texture::sprite_contents::{
    AdditionalMetadataType, SpriteContents,
};
use crate::net::minecraft::client::renderer::texture::stitcher::{Stitcher, StitcherException};
use crate::net::minecraft::client::renderer::texture::texture_atlas_sprite::TextureAtlasSprite;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::resources::resource_manager::ResourceManager;
use crate::net::minecraft::util::mth::Mth;

pub struct SpriteLoader {
    location: Identifier,
    max_supported_texture_size: i32,
}

/// `SpriteLoader.Preparations`: the atlas's size and mip level, and its sprites by name. A
/// sprite is shared with the models whose quads it textures (`BakedQuad.MaterialInfo`), so it's
/// an `Arc`.
pub struct Preparations {
    pub width: i32,
    pub height: i32,
    pub mip_level: i32,
    pub regions: Vec<Arc<TextureAtlasSprite>>,
    by_name: HashMap<Identifier, usize>,
}

impl Preparations {
    pub fn new(
        width: i32,
        height: i32,
        mip_level: i32,
        regions: Vec<Arc<TextureAtlasSprite>>,
    ) -> Preparations {
        let by_name = regions
            .iter()
            .enumerate()
            .map(|(index, sprite)| (sprite.contents().name().clone(), index))
            .collect();
        Preparations {
            width,
            height,
            mip_level,
            regions,
            by_name,
        }
    }

    pub fn get_sprite(&self, id: &Identifier) -> Option<&Arc<TextureAtlasSprite>> {
        self.by_name.get(id).map(|&index| &self.regions[index])
    }

    pub fn missing(&self) -> Option<&Arc<TextureAtlasSprite>> {
        self.get_sprite(&MissingTextureAtlasSprite::get_location())
    }
}

impl SpriteLoader {
    pub fn new(location: Identifier, max_supported_texture_size: i32) -> SpriteLoader {
        SpriteLoader {
            location,
            max_supported_texture_size,
        }
    }

    /// The layout, before any sprite's mip levels: the stitched sprites in the stitcher's order,
    /// and the mip level. Java logs a sprite that limits the mip level. The `Err` is Java's
    /// crash report around a `StitcherException`.
    pub fn stitch_layout(
        &self,
        sprites: Vec<SpriteContents>,
        max_mipmap_levels: i32,
        anisotropy_bit: i32,
    ) -> Result<Preparations, StitcherException> {
        let max_texture_size = self.max_supported_texture_size;
        let mut min_texel_size = i32::MAX;
        let mut lowest_one_bit = 1 << max_mipmap_levels;
        for sprite_info in &sprites {
            min_texel_size = min_texel_size.min(sprite_info.width().min(sprite_info.height()));
            let lowest_texture_bit =
                lowest_one_bit_of(sprite_info.width()).min(lowest_one_bit_of(sprite_info.height()));
            if lowest_texture_bit < lowest_one_bit {
                lowest_one_bit = lowest_texture_bit;
            }
        }
        let min_size = min_texel_size.min(lowest_one_bit);
        let min_power_of_two = Mth::log2(min_size);
        let mip_level = if min_power_of_two < max_mipmap_levels {
            min_power_of_two
        } else {
            max_mipmap_levels
        };

        let mut stitcher = Stitcher::new(
            max_texture_size,
            max_texture_size,
            mip_level,
            anisotropy_bit,
        );
        for sprite_info in sprites {
            stitcher.register_sprite(sprite_info);
        }
        stitcher.stitch()?;

        let width = stitcher.get_width();
        let height = stitcher.get_height();
        let mut regions = Vec::new();
        stitcher.gather_sprites(|contents, x, y, padding| {
            regions.push(Arc::new(TextureAtlasSprite::new(
                self.location.clone(),
                contents,
                width,
                height,
                x,
                y,
                padding,
            )));
        });
        Ok(Preparations::new(width, height, mip_level, regions))
    }

    /// `stitch`, with `readyForUpload` run: every sprite's mip levels made.
    pub fn stitch(
        &self,
        sprites: Vec<SpriteContents>,
        max_mipmap_levels: i32,
        anisotropy_bit: i32,
    ) -> Result<Preparations, StitcherException> {
        let mut preparations = self.stitch_layout(sprites, max_mipmap_levels, anisotropy_bit)?;
        let mip_level = preparations.mip_level;
        for sprite in &mut preparations.regions {
            Arc::get_mut(sprite)
                .expect("no model holds a sprite before the atlas is ready")
                .contents_mut()
                .increase_mip_level(mip_level);
        }
        Ok(preparations)
    }

    /// `loadAndStitch`: the atlas definition `atlas_info_location`'s sprites, loaded with the
    /// additional metadata sections the atlas asks for, and stitched.
    pub fn load_and_stitch(
        &self,
        manager: &dyn ResourceManager,
        atlas_info_location: &Identifier,
        max_mipmap_levels: i32,
        anisotropy_bit: i32,
        additional_metadata: &[AdditionalMetadataType],
    ) -> Result<Preparations, StitcherException> {
        let sprites = SpriteLoader::load_sprites(manager, atlas_info_location, additional_metadata);
        self.stitch(sprites, max_mipmap_levels, anisotropy_bit)
    }

    /// The definition's sprites, loaded (`runSpriteSuppliers`), the ones that failed left out.
    pub fn load_sprites(
        manager: &dyn ResourceManager,
        atlas_info_location: &Identifier,
        additional_metadata: &[AdditionalMetadataType],
    ) -> Vec<SpriteContents> {
        SpriteSourceList::load(manager, atlas_info_location)
            .list(manager)
            .iter()
            .filter_map(|loader| loader.get(additional_metadata))
            .collect()
    }
}

/// `Integer.lowestOneBit`.
fn lowest_one_bit_of(i: i32) -> i32 {
    i.isolate_lowest_one()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::com::mojang::blaze3d::platform::native_image::tests::encode_image;
    use crate::golden::Golden;
    use crate::net::minecraft::server::packs::resources::multi_pack_resource_manager::tests::vanilla_resources;
    use serde_json::{Value, json};

    /// `TextureSuites`' constants: llvmpipe's texture size, the oracle client's mipmap levels and
    /// no anisotropy.
    const MAX_TEXTURE_SIZE: i32 = 16384;
    const ANISOTROPY_BIT: i32 = 0;

    /// The block atlas's layout and every sprite's mip levels, against vanilla's stitcher and
    /// mipmapper run over the same images.
    #[test]
    fn stitches_and_mipmaps_the_block_atlas_like_vanilla() {
        let golden = Golden::load("net.minecraft.client.renderer.texture.SpriteLoader");
        let manager = vanilla_resources();
        let loader = SpriteLoader::new(
            Identifier::with_default_namespace("blocks"),
            MAX_TEXTURE_SIZE,
        );
        golden.check("stitch", |atlas: String, max_mipmap_levels: i32| {
            let sprites = SpriteLoader::load_sprites(&manager, &Identifier::parse(&atlas), &[]);
            let preparations = loader
                .stitch_layout(sprites, max_mipmap_levels, ANISOTROPY_BIT)
                .expect("the atlas fits");
            let placed: Vec<Value> = preparations
                .regions
                .iter()
                .map(|sprite| {
                    json!([
                        sprite.contents().name().to_string(),
                        sprite.get_x(),
                        sprite.get_y(),
                        sprite.contents().width(),
                        sprite.contents().height(),
                        sprite.padding()
                    ])
                })
                .collect();
            json!([
                preparations.width,
                preparations.height,
                preparations.mip_level,
                placed
            ])
        });

        let sprites = SpriteLoader::load_sprites(
            &manager,
            &Identifier::with_default_namespace("blocks"),
            &[],
        );
        golden.check("increaseMipLevel", |name: String, mip_level: i32| {
            let name = Identifier::parse(&name);
            let sprite = sprites
                .iter()
                .find(|sprite| *sprite.name() == name)
                .expect("the port loads the sprite");
            // Each case starts from the loaded sprite, as vanilla's did.
            let mut sprite = sprite.clone();
            sprite.increase_mip_level(mip_level);
            let levels: Vec<Value> = sprite.by_mip_level().iter().map(encode_image).collect();
            json!([sprite.is_animated(), levels])
        });
        golden.finish();
    }
}
