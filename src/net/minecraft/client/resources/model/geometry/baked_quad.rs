//! net/minecraft/client/resources/model/geometry/BakedQuad.java
//!
//! `MaterialInfo`'s item render types (`Sheets`' render types for an item drawn with the quad)
//! aren't ported yet: they come with item rendering.

use std::sync::Arc;

use crate::com::mojang::blaze3d::platform::transparency::Transparency;
use crate::net::minecraft::client::renderer::chunk::chunk_section_layer::ChunkSectionLayer;
use crate::net::minecraft::client::renderer::texture::texture_atlas_sprite::TextureAtlasSprite;
use crate::net::minecraft::client::resources::model::sprite::material::Baked;
use crate::net::minecraft::core::direction::Direction;
use crate::org::joml::vector3f::Vector3f;

/// A baked face: its corners in block units, their packed atlas UVs (`UVPair.pack`), the way it
/// faces, and what it's drawn with.
#[derive(Clone)]
pub struct BakedQuad {
    pub positions: [Vector3f; 4],
    pub packed_uvs: [i64; 4],
    pub direction: Direction,
    pub material_info: Arc<MaterialInfo>,
}

/// `BakedQuad.MaterialInfo`: the quad's sprite, the section layer it goes in, its tint index,
/// the direction it's shaded as and the light it gives off.
pub struct MaterialInfo {
    pub sprite: Arc<TextureAtlasSprite>,
    pub layer: ChunkSectionLayer,
    pub tint_index: i32,
    pub shade_direction_override: Option<Direction>,
    pub light_emission: i32,
}

impl BakedQuad {
    pub const VERTEX_COUNT: usize = 4;
    pub const FLAG_TRANSLUCENT: i32 = 1;
    pub const FLAG_ANIMATED: i32 = 2;

    /// Java throws `IndexOutOfBoundsException` past 3.
    pub fn position(&self, vertex: usize) -> &Vector3f {
        &self.positions[vertex]
    }

    pub fn packed_uv(&self, vertex: usize) -> i64 {
        self.packed_uvs[vertex]
    }
}

/// The record's `equals`, which the bake's interner shares equal infos by: the sprite by
/// reference, the rest by value.
impl PartialEq for MaterialInfo {
    fn eq(&self, other: &MaterialInfo) -> bool {
        Arc::ptr_eq(&self.sprite, &other.sprite)
            && self.layer == other.layer
            && self.tint_index == other.tint_index
            && self.shade_direction_override == other.shade_direction_override
            && self.light_emission == other.light_emission
    }
}

impl MaterialInfo {
    pub fn of(
        material: &Baked,
        transparency: Transparency,
        tint_index: i32,
        shade_direction_override: Option<Direction>,
        light_emission: i32,
    ) -> MaterialInfo {
        MaterialInfo {
            sprite: material.sprite.clone(),
            layer: ChunkSectionLayer::by_transparency(transparency),
            tint_index,
            shade_direction_override,
            light_emission,
        }
    }

    pub fn is_tinted(&self) -> bool {
        self.tint_index != -1
    }

    pub fn flags(&self) -> i32 {
        let mut flags = 0;
        flags |= if self.layer.translucent() {
            BakedQuad::FLAG_TRANSLUCENT
        } else {
            0
        };
        flags
            | if self.sprite.contents().is_animated() {
                BakedQuad::FLAG_ANIMATED
            } else {
                0
            }
    }
}
