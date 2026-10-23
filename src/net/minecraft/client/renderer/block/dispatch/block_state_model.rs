//! net/minecraft/client/renderer/block/dispatch/BlockStateModel.java
//!
//! Not ported yet: `Unbaked.CODEC`, DataFixerUpper's, which `BlockStateModelDispatcher` decodes
//! by hand (`parse_unbaked`), and `UnbakedRoot.visualEqualityGroup`, which comes with
//! `ModelGroupCollector`.

use std::any::Any;
use std::sync::Arc;

use crate::net::minecraft::client::renderer::block::dispatch::block_state_model_part::BlockStateModelPart;
use crate::net::minecraft::client::resources::model::model_baker::ModelBaker;
use crate::net::minecraft::client::resources::model::resolvable_model::{
    ResolvableModel, Resolver,
};
use crate::net::minecraft::client::resources::model::sprite::material::Baked;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

/// What a block state draws: the parts it picks for a position's random seed.
pub trait BlockStateModel: Send + Sync {
    fn collect_parts(
        &self,
        random: &mut dyn RandomSource,
        output: &mut Vec<Arc<dyn BlockStateModelPart>>,
    );

    fn particle_material(&self) -> &Baked;

    fn material_flags(&self) -> i32;

    fn has_material_flag(&self, flag: i32) -> bool {
        (self.material_flags() & flag) != 0
    }

    fn as_any(&self) -> &dyn Any;
}

/// `BlockStateModel.Unbaked`: a block state's model as its definition says it.
pub trait Unbaked: ResolvableModel + Send + Sync {
    fn bake(&self, model_bakery: &dyn ModelBaker) -> Arc<dyn BlockStateModel>;
}

/// `asRoot()`: the model, baked once for every state that has it.
pub fn as_root(contents: Arc<dyn Unbaked>) -> Arc<dyn UnbakedRoot> {
    Arc::new(SimpleCachedUnbakedRoot { contents })
}

/// `BlockStateModel.UnbakedRoot`: what a block state's definition gives it, baked for the state.
pub trait UnbakedRoot: ResolvableModel + Send + Sync {
    fn bake(
        &self,
        block_state: BlockState,
        model_bakery: &dyn ModelBaker,
    ) -> Arc<dyn BlockStateModel>;
}

/// `BlockStateModel.SimpleCachedUnbakedRoot`: Java's baking key is an object of its own, which
/// the bakery's cache keys by; here the root's address is.
pub struct SimpleCachedUnbakedRoot {
    contents: Arc<dyn Unbaked>,
}

impl ResolvableModel for SimpleCachedUnbakedRoot {
    fn resolve_dependencies(&self, resolver: &mut dyn Resolver) {
        self.contents.resolve_dependencies(resolver);
    }
}

impl UnbakedRoot for SimpleCachedUnbakedRoot {
    fn bake(
        &self,
        _block_state: BlockState,
        model_bakery: &dyn ModelBaker,
    ) -> Arc<dyn BlockStateModel> {
        let key = std::ptr::from_ref(self) as usize;
        let baked = model_bakery.compute(key, &|baker| Arc::new(self.contents.bake(baker)));
        baked
            .downcast_ref::<Arc<dyn BlockStateModel>>()
            .expect("the key's operation bakes a model")
            .clone()
    }
}
