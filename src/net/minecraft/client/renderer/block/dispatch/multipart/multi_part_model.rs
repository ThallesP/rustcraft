//! net/minecraft/client/renderer/block/dispatch/multipart/MultiPartModel.java
//!
//! Not ported yet: `Unbaked.visualEqualityGroup`, which comes with `ModelGroupCollector`.
//!
//! A state's selected models are worked out on first use (Java's nullable `models`, here a
//! `OnceLock`), and the lists are shared by every state that selects the same ones, by which
//! selectors match (Java's `BitSet` keys, here a list of the matching indexes).

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use crate::net::minecraft::client::renderer::block::dispatch::block_state_model::{
    self, BlockStateModel, UnbakedRoot,
};
use crate::net::minecraft::client::renderer::block::dispatch::block_state_model_part::BlockStateModelPart;
use crate::net::minecraft::client::renderer::block::dispatch::variant_selector::Predicate;
use crate::net::minecraft::client::resources::model::model_baker::ModelBaker;
use crate::net::minecraft::client::resources::model::resolvable_model::{
    ResolvableModel, Resolver,
};
use crate::net::minecraft::client::resources::model::sprite::material::Baked;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

/// The models a state selects, shared by the states that select the same.
type Models = Arc<Vec<Arc<dyn BlockStateModel>>>;

/// Every model whose condition the state meets, each with the same seed.
pub struct MultiPartModel {
    shared: Arc<SharedBakedState>,
    block_state: BlockState,
    models: OnceLock<Models>,
}

/// `MultiPartModel.Selector`: a condition and what it selects.
pub struct Selector<T> {
    pub condition: Arc<Predicate>,
    pub model: T,
}

/// `MultiPartModel.SharedBakedState`.
pub struct SharedBakedState {
    selectors: Vec<Selector<Arc<dyn BlockStateModel>>>,
    particle_material: Baked,
    material_flags: i32,
    subsets: Mutex<HashMap<Vec<usize>, Models>>,
}

impl MultiPartModel {
    /// `new MultiPartModel.Unbaked(selectors)`.
    pub fn unbaked(selectors: Vec<Selector<Arc<dyn block_state_model::Unbaked>>>) -> Unbaked {
        Unbaked { selectors }
    }

    pub fn select_models(&self) -> &Models {
        self.models
            .get_or_init(|| self.shared.select_models(self.block_state))
    }
}

impl BlockStateModel for MultiPartModel {
    fn collect_parts(
        &self,
        random: &mut dyn RandomSource,
        output: &mut Vec<Arc<dyn BlockStateModelPart>>,
    ) {
        let models = self.select_models().clone();
        let seed = random.next_long();
        for model in models.iter() {
            random.set_seed(seed);
            model.collect_parts(random, output);
        }
    }

    fn particle_material(&self) -> &Baked {
        &self.shared.particle_material
    }

    fn material_flags(&self) -> i32 {
        self.shared.material_flags
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl SharedBakedState {
    /// Java throws `IllegalArgumentException` for a model without selectors.
    pub fn new(selectors: Vec<Selector<Arc<dyn BlockStateModel>>>) -> SharedBakedState {
        let first_model = &selectors
            .first()
            .expect("IllegalArgumentException: Model must have at least one selector")
            .model;
        let particle_material = first_model.particle_material().clone();
        let material_flags = selectors
            .iter()
            .fold(0, |flags, selector| flags | selector.model.material_flags());
        SharedBakedState {
            selectors,
            particle_material,
            material_flags,
            subsets: Mutex::default(),
        }
    }

    pub fn select_models(&self, state: BlockState) -> Models {
        let selected_models: Vec<usize> = (0..self.selectors.len())
            .filter(|&i| (self.selectors[i].condition.as_ref())(state))
            .collect();
        self.subsets
            .lock()
            .expect("not poisoned")
            .entry(selected_models)
            .or_insert_with_key(|selected| {
                Arc::new(
                    selected
                        .iter()
                        .map(|&i| self.selectors[i].model.clone())
                        .collect(),
                )
            })
            .clone()
    }
}

/// `MultiPartModel.Unbaked`: the selectors with their conditions instantiated for the block;
/// their models are baked once, the first time a state of the block is (Java's shared state key,
/// here the root's address).
pub struct Unbaked {
    selectors: Vec<Selector<Arc<dyn block_state_model::Unbaked>>>,
}

impl ResolvableModel for Unbaked {
    fn resolve_dependencies(&self, resolver: &mut dyn Resolver) {
        for selector in &self.selectors {
            selector.model.resolve_dependencies(resolver);
        }
    }
}

impl UnbakedRoot for Unbaked {
    fn bake(
        &self,
        block_state: BlockState,
        model_bakery: &dyn ModelBaker,
    ) -> Arc<dyn BlockStateModel> {
        let key = std::ptr::from_ref(self) as usize;
        let shared = model_bakery.compute(key, &|baker| {
            let selectors = self
                .selectors
                .iter()
                .map(|selector| Selector {
                    condition: selector.condition.clone(),
                    model: selector.model.bake(baker),
                })
                .collect();
            Arc::new(Arc::new(SharedBakedState::new(selectors)))
        });
        let shared = shared
            .downcast_ref::<Arc<SharedBakedState>>()
            .expect("the key's operation makes the shared state")
            .clone();
        Arc::new(MultiPartModel {
            shared,
            block_state,
            models: OnceLock::new(),
        })
    }
}
