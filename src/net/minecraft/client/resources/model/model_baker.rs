//! net/minecraft/client/resources/model/ModelBaker.java
//!
//! `compute(SharedOperationKey)` runs an operation once for the bake: Java keys the cache by the
//! key object, here by its address (`key`), and the operation's result is any shared value, which
//! the caller downcasts back.

use std::any::Any;
use std::sync::Arc;

use crate::net::minecraft::client::renderer::block::dispatch::block_state_model_part::BlockStateModelPart;
use crate::net::minecraft::client::resources::model::geometry::baked_quad::MaterialInfo;
use crate::net::minecraft::client::resources::model::resolved_model::ResolvedModel;
use crate::net::minecraft::client::resources::model::sprite::material_baker::MaterialBaker;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::org::joml::vector3f::Vector3f;

pub type Shared = Arc<dyn Any + Send + Sync>;

pub trait ModelBaker {
    fn get_model(&self, location: &Identifier) -> Arc<dyn ResolvedModel>;

    fn missing_block_model_part(&self) -> Arc<dyn BlockStateModelPart>;

    fn materials(&self) -> &MaterialBaker<'_>;

    fn interner(&self) -> &dyn Interner;

    /// `compute(key)`: `operation`'s result for the key, worked out the first time it's asked
    /// for. Java's missing-model baker caches nothing.
    fn compute(&self, key: usize, operation: &dyn Fn(&dyn ModelBaker) -> Shared) -> Shared;
}

/// `ModelBaker.Interner`: equal values shared. A vector is a copy here, which is the same value.
pub trait Interner {
    fn vector(&self, vector: Vector3f) -> Vector3f {
        vector
    }

    fn material_info(&self, material: MaterialInfo) -> Arc<MaterialInfo>;
}
