//! net/minecraft/client/renderer/block/dispatch/WeightedVariants.java

use std::any::Any;
use std::sync::Arc;

use crate::net::minecraft::client::renderer::block::dispatch::block_state_model::{
    self, BlockStateModel,
};
use crate::net::minecraft::client::renderer::block::dispatch::block_state_model_part::BlockStateModelPart;
use crate::net::minecraft::client::resources::model::model_baker::ModelBaker;
use crate::net::minecraft::client::resources::model::resolvable_model::{
    ResolvableModel, Resolver,
};
use crate::net::minecraft::client::resources::model::sprite::material::Baked;
use crate::net::minecraft::util::random::weighted_list::WeightedList;
use crate::net::minecraft::util::random_source::RandomSource;

/// One of several models, picked by weight with the seed; the first's particle.
pub struct WeightedVariants {
    list: WeightedList<Arc<dyn BlockStateModel>>,
    particle_material: Baked,
    material_flags: i32,
}

impl WeightedVariants {
    /// Java throws `NoSuchElementException` for an empty list.
    pub fn new(list: WeightedList<Arc<dyn BlockStateModel>>) -> WeightedVariants {
        let first_model = list
            .unwrap()
            .first()
            .expect("NoSuchElementException")
            .value();
        let particle_material = first_model.particle_material().clone();
        let material_flags = list
            .unwrap()
            .iter()
            .fold(0, |flags, entry| flags | entry.value().material_flags());
        WeightedVariants {
            list,
            particle_material,
            material_flags,
        }
    }

    pub fn list(&self) -> &WeightedList<Arc<dyn BlockStateModel>> {
        &self.list
    }
}

impl BlockStateModel for WeightedVariants {
    fn collect_parts(
        &self,
        random: &mut dyn RandomSource,
        output: &mut Vec<Arc<dyn BlockStateModelPart>>,
    ) {
        self.list
            .get_random_or_throw(random)
            .collect_parts(random, output);
    }

    fn particle_material(&self) -> &Baked {
        &self.particle_material
    }

    fn material_flags(&self) -> i32 {
        self.material_flags
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// `WeightedVariants.Unbaked`.
pub struct Unbaked {
    pub entries: WeightedList<Arc<dyn block_state_model::Unbaked>>,
}

impl block_state_model::Unbaked for Unbaked {
    fn bake(&self, model_bakery: &dyn ModelBaker) -> Arc<dyn BlockStateModel> {
        Arc::new(WeightedVariants::new(
            self.entries.map(|model| model.bake(model_bakery)),
        ))
    }
}

impl ResolvableModel for Unbaked {
    fn resolve_dependencies(&self, resolver: &mut dyn Resolver) {
        for entry in self.entries.unwrap() {
            entry.value().resolve_dependencies(resolver);
        }
    }
}
