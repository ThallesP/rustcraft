//! net/minecraft/client/renderer/block/dispatch/SingleVariant.java

use std::any::Any;
use std::sync::Arc;

use crate::net::minecraft::client::renderer::block::dispatch::block_state_model::{
    self, BlockStateModel,
};
use crate::net::minecraft::client::renderer::block::dispatch::block_state_model_part::{
    self, BlockStateModelPart,
};
use crate::net::minecraft::client::renderer::block::dispatch::variant::Variant;
use crate::net::minecraft::client::resources::model::model_baker::ModelBaker;
use crate::net::minecraft::client::resources::model::resolvable_model::{
    ResolvableModel, Resolver,
};
use crate::net::minecraft::client::resources::model::sprite::material::Baked;
use crate::net::minecraft::util::random_source::RandomSource;

/// One part, whatever the seed.
pub struct SingleVariant {
    model: Arc<dyn BlockStateModelPart>,
}

impl SingleVariant {
    pub fn new(model: Arc<dyn BlockStateModelPart>) -> SingleVariant {
        SingleVariant { model }
    }

    pub fn model(&self) -> &Arc<dyn BlockStateModelPart> {
        &self.model
    }
}

impl BlockStateModel for SingleVariant {
    fn collect_parts(
        &self,
        _random: &mut dyn RandomSource,
        output: &mut Vec<Arc<dyn BlockStateModelPart>>,
    ) {
        output.push(self.model.clone());
    }

    fn particle_material(&self) -> &Baked {
        self.model.particle_material()
    }

    fn material_flags(&self) -> i32 {
        self.model.material_flags()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// `SingleVariant.Unbaked`.
pub struct Unbaked {
    pub variant: Variant,
}

impl block_state_model::Unbaked for Unbaked {
    fn bake(&self, model_bakery: &dyn ModelBaker) -> Arc<dyn BlockStateModel> {
        Arc::new(SingleVariant::new(block_state_model_part::Unbaked::bake(
            &self.variant,
            model_bakery,
        )))
    }
}

impl ResolvableModel for Unbaked {
    fn resolve_dependencies(&self, resolver: &mut dyn Resolver) {
        self.variant.resolve_dependencies(resolver);
    }
}
