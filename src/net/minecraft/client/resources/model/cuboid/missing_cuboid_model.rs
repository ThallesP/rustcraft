//! net/minecraft/client/resources/model/cuboid/MissingCuboidModel.java
//!
//! Its `ItemTransforms.NO_TRANSFORMS` comes with item models.

use std::sync::Arc;

use crate::com::mojang::math::quadrant::Quadrant;
use crate::net::minecraft::client::renderer::texture::missing_texture_atlas_sprite::MissingTextureAtlasSprite;
use crate::net::minecraft::client::resources::model::cuboid::cuboid_face::{CuboidFace, UVs};
use crate::net::minecraft::client::resources::model::cuboid::cuboid_model::CuboidModel;
use crate::net::minecraft::client::resources::model::cuboid::cuboid_model_element::CuboidModelElement;
use crate::net::minecraft::client::resources::model::cuboid::unbaked_cuboid_geometry::UnbakedCuboidGeometry;
use crate::net::minecraft::client::resources::model::sprite::material::Material;
use crate::net::minecraft::client::resources::model::sprite::texture_slots::DataBuilder;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::org::joml::vector3f::Vector3f;

const TEXTURE_SLOT: &str = "missingno";

pub struct MissingCuboidModel;

impl MissingCuboidModel {
    /// `LOCATION`.
    pub fn location() -> Identifier {
        Identifier::with_default_namespace("builtin/missing")
    }

    /// A whole block, every face the missing texture, culled toward its side.
    pub fn missing_model() -> CuboidModel {
        let full_face_uv = UVs {
            min_u: 0.0,
            min_v: 0.0,
            max_u: 16.0,
            max_v: 16.0,
        };
        let faces = Direction::VALUES
            .map(|direction| {
                (
                    direction,
                    CuboidFace {
                        cull_for_direction: Some(direction),
                        tint_index: CuboidFace::NO_TINT,
                        texture: TEXTURE_SLOT.to_owned(),
                        uvs: Some(full_face_uv),
                        rotation: Quadrant::R0,
                    },
                )
            })
            .to_vec();
        let cube = CuboidModelElement::new(
            Vector3f::from_xyz(0.0, 0.0, 0.0),
            Vector3f::from_xyz(16.0, 16.0, 16.0),
            faces,
        );
        let mut texture_slots = DataBuilder::default();
        texture_slots
            .add_reference("particle", TEXTURE_SLOT)
            .add_texture(
                TEXTURE_SLOT,
                Material::new(MissingTextureAtlasSprite::get_location()),
            );
        CuboidModel {
            geometry: Some(Arc::new(UnbakedCuboidGeometry::new(vec![cube]))),
            gui_light: None,
            ambient_occlusion: None,
            texture_slots: texture_slots.build(),
            parent: None,
        }
    }
}
