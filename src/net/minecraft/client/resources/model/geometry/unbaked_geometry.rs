//! net/minecraft/client/resources/model/geometry/UnbakedGeometry.java

use crate::net::minecraft::client::renderer::block::dispatch::model_state::ModelState;
use crate::net::minecraft::client::resources::model::geometry::quad_collection::QuadCollection;
use crate::net::minecraft::client::resources::model::model_baker::ModelBaker;
use crate::net::minecraft::client::resources::model::model_debug_name::ModelDebugName;
use crate::net::minecraft::client::resources::model::sprite::texture_slots::TextureSlots;

/// A model's shape, before it's baked into quads for a model state.
pub trait UnbakedGeometry: Send + Sync {
    fn bake(
        &self,
        textures: &TextureSlots,
        model_baker: &dyn ModelBaker,
        model_state: &'static dyn ModelState,
        name: &dyn ModelDebugName,
    ) -> QuadCollection;
}

/// `EMPTY`: no quads.
pub struct Empty;

impl UnbakedGeometry for Empty {
    fn bake(
        &self,
        _textures: &TextureSlots,
        _model_baker: &dyn ModelBaker,
        _model_state: &'static dyn ModelState,
        _name: &dyn ModelDebugName,
    ) -> QuadCollection {
        QuadCollection::empty()
    }
}
