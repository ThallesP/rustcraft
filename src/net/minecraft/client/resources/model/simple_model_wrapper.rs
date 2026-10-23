//! net/minecraft/client/resources/model/SimpleModelWrapper.java

use std::any::Any;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::net::minecraft::client::renderer::block::dispatch::block_state_model_part::BlockStateModelPart;
use crate::net::minecraft::client::renderer::block::dispatch::model_state::ModelState;
use crate::net::minecraft::client::renderer::texture::texture_atlas::TextureAtlas;
use crate::net::minecraft::client::resources::model::geometry::baked_quad::BakedQuad;
use crate::net::minecraft::client::resources::model::geometry::quad_collection::QuadCollection;
use crate::net::minecraft::client::resources::model::model_baker::ModelBaker;
use crate::net::minecraft::client::resources::model::sprite::material::Baked;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::resources::identifier::Identifier;

/// A model baked for a block state: its quads, whether it's shaded with ambient occlusion, and
/// its particle.
pub struct SimpleModelWrapper {
    pub quads: Arc<QuadCollection>,
    pub use_ambient_occlusion: bool,
    pub particle_material: Baked,
}

impl SimpleModelWrapper {
    /// The model at `location` baked in `state`, or the missing model's part when a quad's sprite
    /// isn't the block atlas's, which Java warns about.
    pub fn bake(
        model_bakery: &dyn ModelBaker,
        location: &Identifier,
        state: &'static dyn ModelState,
    ) -> Arc<dyn BlockStateModelPart> {
        let model = model_bakery.get_model(location);
        let texture_slots = model.get_top_texture_slots();
        let has_ambient_occlusion = model.get_top_ambient_occlusion();
        let particle_material = model.resolve_particle_material(&texture_slots, model_bakery);
        let geometry = model.bake_top_geometry(&texture_slots, model_bakery, state);
        if let Some(forbidden_sprites) = SimpleModelWrapper::find_non_block_sprites(&geometry) {
            eprintln!(
                "Rejecting block model {location}, since it contains sprites from outside of supported atlas: {forbidden_sprites:?}"
            );
            return model_bakery.missing_block_model_part();
        }
        Arc::new(SimpleModelWrapper {
            quads: geometry,
            use_ambient_occlusion: has_ambient_occlusion,
            particle_material,
        })
    }

    /// The sprites by atlas of the quads that aren't the block atlas's, or `None`. Java's is a
    /// `HashMultimap`, only ever logged; the port's is sorted.
    pub fn find_non_block_sprites(
        geometry: &QuadCollection,
    ) -> Option<BTreeMap<String, BTreeSet<String>>> {
        let blocks = Identifier::with_default_namespace(TextureAtlas::LOCATION_BLOCKS);
        let mut forbidden_sprites: Option<BTreeMap<String, BTreeSet<String>>> = None;
        for baked_quad in geometry.get_all() {
            let sprite = &baked_quad.material_info.sprite;
            if *sprite.atlas_location() != blocks {
                forbidden_sprites
                    .get_or_insert_with(BTreeMap::new)
                    .entry(sprite.atlas_location().to_string())
                    .or_default()
                    .insert(sprite.contents().name().to_string());
            }
        }
        forbidden_sprites
    }
}

impl BlockStateModelPart for SimpleModelWrapper {
    fn get_quads(&self, direction: Option<Direction>) -> &[BakedQuad] {
        self.quads.get_quads(direction)
    }

    fn use_ambient_occlusion(&self) -> bool {
        self.use_ambient_occlusion
    }

    fn particle_material(&self) -> &Baked {
        &self.particle_material
    }

    fn material_flags(&self) -> i32 {
        self.quads.material_flags()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
