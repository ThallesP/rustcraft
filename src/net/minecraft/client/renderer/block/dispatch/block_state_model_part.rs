//! net/minecraft/client/renderer/block/dispatch/BlockStateModelPart.java

use std::any::Any;
use std::sync::Arc;

use crate::net::minecraft::client::resources::model::geometry::baked_quad::BakedQuad;
use crate::net::minecraft::client::resources::model::model_baker::ModelBaker;
use crate::net::minecraft::client::resources::model::resolvable_model::ResolvableModel;
use crate::net::minecraft::client::resources::model::sprite::material::Baked;
use crate::net::minecraft::core::direction::Direction;

/// One baked model of the ones a block state draws: its quads unculled (`None`) or culled toward
/// each side.
pub trait BlockStateModelPart: Send + Sync {
    fn get_quads(&self, direction: Option<Direction>) -> &[BakedQuad];

    fn use_ambient_occlusion(&self) -> bool;

    fn particle_material(&self) -> &Baked;

    fn material_flags(&self) -> i32;

    fn as_any(&self) -> &dyn Any;
}

/// `BlockStateModelPart.Unbaked`.
pub trait Unbaked: ResolvableModel + Send + Sync {
    fn bake(&self, model_bakery: &dyn ModelBaker) -> Arc<dyn BlockStateModelPart>;
}
