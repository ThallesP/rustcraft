//! net/minecraft/client/renderer/block/FluidModel.java
//!
//! A fluid's sprites and how they're drawn: the still one on its top and bottom, the flowing one
//! on its sloped top and sides, the overlay (water's) on a side against glass or leaves, and the
//! layer they go in, which their pixels decide.

use std::sync::Arc;

use crate::com::mojang::blaze3d::platform::transparency::Transparency;
use crate::net::minecraft::client::color::block::block_tint_source::BlockTintSource;
use crate::net::minecraft::client::renderer::chunk::chunk_section_layer::ChunkSectionLayer;
use crate::net::minecraft::client::renderer::texture::texture_atlas::TextureAtlas;
use crate::net::minecraft::client::resources::model::model_debug_name::ModelDebugName;
use crate::net::minecraft::client::resources::model::sprite::material::{Baked, Material};
use crate::net::minecraft::client::resources::model::sprite::material_baker::MaterialBaker;
use crate::net::minecraft::resources::identifier::Identifier;

pub struct FluidModel {
    pub layer: ChunkSectionLayer,
    pub still_material: Baked,
    pub flowing_material: Baked,
    pub overlay_material: Option<Baked>,
    pub tint_source: Option<Arc<dyn BlockTintSource>>,
}

/// `FluidModel.Unbaked`.
pub struct Unbaked {
    pub still_material: Material,
    pub flowing_material: Material,
    pub overlay_material: Option<Material>,
    pub tint_source: Option<Arc<dyn BlockTintSource>>,
}

impl Unbaked {
    pub fn bake(
        &self,
        materials: &MaterialBaker<'_>,
        model_name: &dyn ModelDebugName,
    ) -> FluidModel {
        let still_material = Unbaked::get_and_validate_material(
            &self.still_material,
            materials,
            "still",
            model_name,
        );
        let flowing_material = Unbaked::get_and_validate_material(
            &self.flowing_material,
            materials,
            "flowing",
            model_name,
        );
        let overlay_material = self.overlay_material.as_ref().map(|overlay_material| {
            Unbaked::get_and_validate_material(overlay_material, materials, "overlay", model_name)
        });
        let mut transparency = Unbaked::get_transparency(&still_material)
            .or(Unbaked::get_transparency(&flowing_material));
        if let Some(overlay_material) = &overlay_material {
            transparency = transparency.or(Unbaked::get_transparency(overlay_material));
        }

        FluidModel {
            layer: ChunkSectionLayer::by_transparency(transparency),
            still_material,
            flowing_material,
            overlay_material,
            tint_source: self.tint_source.clone(),
        }
    }

    /// The material's sprite, or the missing one reported for a sprite outside the block atlas.
    fn get_and_validate_material(
        material: &Material,
        materials: &MaterialBaker<'_>,
        texture_name: &str,
        model_name: &dyn ModelDebugName,
    ) -> Baked {
        let baked = materials.get(material, model_name);
        if *baked.sprite.atlas_location()
            != Identifier::with_default_namespace(TextureAtlas::LOCATION_BLOCKS)
        {
            materials.report_missing_reference(texture_name, model_name)
        } else {
            baked
        }
    }

    fn get_transparency(material: &Baked) -> Transparency {
        if material.force_translucent {
            Transparency::TRANSLUCENT
        } else {
            material.sprite.transparency()
        }
    }
}
