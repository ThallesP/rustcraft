//! net/minecraft/client/resources/model/ModelDiscovery.java
//!
//! Not ported yet: `addSpecialModel` and the special model ModelManager adds with it
//! (`ItemModelGenerator` as `builtin/generated`), which only item models have as a parent.
//!
//! A model and the parent it's found to have are shared (`Arc`); a cycle of parents would never be
//! freed, which only an invalid pack has, and Java ignores its models. Java's map of models is an
//! `Object2ObjectOpenHashMap`, whose order only decides the order of the cycles' warnings.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use crate::net::minecraft::client::renderer::block::dispatch::block_model_rotation::BlockModelRotation;
use crate::net::minecraft::client::renderer::block::dispatch::model_state::ModelState;
use crate::net::minecraft::client::resources::model::cuboid::missing_cuboid_model::MissingCuboidModel;
use crate::net::minecraft::client::resources::model::geometry::quad_collection::QuadCollection;
use crate::net::minecraft::client::resources::model::geometry::unbaked_geometry::UnbakedGeometry;
use crate::net::minecraft::client::resources::model::model_baker::ModelBaker;
use crate::net::minecraft::client::resources::model::model_debug_name::ModelDebugName;
use crate::net::minecraft::client::resources::model::resolvable_model::{
    ResolvableModel, Resolver,
};
use crate::net::minecraft::client::resources::model::resolved_model::{self, ResolvedModel};
use crate::net::minecraft::client::resources::model::sprite::material::Baked;
use crate::net::minecraft::client::resources::model::sprite::texture_slots::TextureSlots;
use crate::net::minecraft::client::resources::model::unbaked_model::{GuiLight, UnbakedModel};
use crate::net::minecraft::resources::identifier::Identifier;

pub struct ModelDiscovery {
    unbaked_models: HashMap<Identifier, Arc<dyn UnbakedModel>>,
    model_wrappers: HashMap<Identifier, Arc<ModelWrapper>>,
    missing_model: Arc<ModelWrapper>,
    parent_discovery_queue: VecDeque<Arc<ModelWrapper>>,
}

/// `ModelDiscovery.ModelWrapper`: a model, its parent once found, whether its parents end in a
/// root, and what it caches of what it inherits (Java's slots, set once each).
pub struct ModelWrapper {
    id: Identifier,
    valid: AtomicBool,
    parent: OnceLock<Arc<ModelWrapper>>,
    wrapped: Arc<dyn UnbakedModel>,
    ambient_occlusion: OnceLock<bool>,
    gui_light: OnceLock<GuiLight>,
    geometry: OnceLock<Arc<dyn UnbakedGeometry>>,
    texture_slots: OnceLock<Arc<TextureSlots>>,
    particle_sprite: OnceLock<Baked>,
    default_geometry: OnceLock<Arc<QuadCollection>>,
    model_bake_cache: Mutex<HashMap<usize, Arc<QuadCollection>>>,
}

impl ModelDiscovery {
    pub fn new(
        unbaked_models: HashMap<Identifier, Arc<dyn UnbakedModel>>,
        missing_unbaked_model: Arc<dyn UnbakedModel>,
    ) -> ModelDiscovery {
        let missing_model = Arc::new(ModelWrapper::new(
            MissingCuboidModel::location(),
            missing_unbaked_model,
            true,
        ));
        let mut model_wrappers = HashMap::new();
        model_wrappers.insert(MissingCuboidModel::location(), missing_model.clone());
        ModelDiscovery {
            unbaked_models,
            model_wrappers,
            missing_model,
            parent_discovery_queue: VecDeque::new(),
        }
    }

    fn is_root(model: &dyn UnbakedModel) -> bool {
        model.parent().is_none()
    }

    fn get_or_create_model(&mut self, id: &Identifier) -> Arc<ModelWrapper> {
        if let Some(model) = self.model_wrappers.get(id) {
            return model.clone();
        }
        // `uncachedResolver`.
        let model = match self.unbaked_models.get(id) {
            None => {
                eprintln!("Missing block model: {id}");
                self.missing_model.clone()
            }
            Some(raw_model) => {
                let raw_model = raw_model.clone();
                self.create_and_queue_wrapper(id, raw_model)
            }
        };
        self.model_wrappers.insert(id.clone(), model.clone());
        model
    }

    fn create_and_queue_wrapper(
        &mut self,
        id: &Identifier,
        raw_model: Arc<dyn UnbakedModel>,
    ) -> Arc<ModelWrapper> {
        let is_root = ModelDiscovery::is_root(&*raw_model);
        let result = Arc::new(ModelWrapper::new(id.clone(), raw_model, is_root));
        if !is_root {
            self.parent_discovery_queue.push_back(result.clone());
        }
        result
    }

    pub fn add_root(&mut self, model: &dyn ResolvableModel) {
        model.resolve_dependencies(&mut DiscoveryResolver { discovery: self });
    }

    pub fn missing_model(&self) -> Arc<ModelWrapper> {
        self.missing_model.clone()
    }

    /// Every model found whose parents end in a root; Java warns about the others.
    pub fn resolve(&mut self) -> HashMap<Identifier, Arc<ModelWrapper>> {
        let mut to_validate = Vec::new();
        self.discover_dependencies(&mut to_validate);
        ModelDiscovery::propagate_validity(&mut to_validate);
        let mut result = HashMap::new();
        for (location, model) in &self.model_wrappers {
            if model.valid.load(Ordering::Relaxed) {
                result.insert(location.clone(), model.clone());
            } else {
                eprintln!("Model {location} ignored due to cyclic dependency");
            }
        }
        result
    }

    fn discover_dependencies(&mut self, to_validate: &mut Vec<Arc<ModelWrapper>>) {
        while let Some(current) = self.parent_discovery_queue.pop_front() {
            let parent_location = current
                .wrapped
                .parent()
                .expect("only models with a parent are queued")
                .clone();
            let parent = self.get_or_create_model(&parent_location);
            let parent_valid = parent.valid.load(Ordering::Relaxed);
            current
                .parent
                .set(parent)
                .unwrap_or_else(|_| panic!("a model's parent is found once"));
            if parent_valid {
                current.valid.store(true, Ordering::Relaxed);
            } else {
                to_validate.push(current);
            }
        }
    }

    fn propagate_validity(to_validate: &mut Vec<Arc<ModelWrapper>>) {
        let mut progressed = true;
        while progressed {
            progressed = false;
            to_validate.retain(|model| {
                let parent = model
                    .parent
                    .get()
                    .expect("a queued model's parent is found");
                if parent.valid.load(Ordering::Relaxed) {
                    model.valid.store(true, Ordering::Relaxed);
                    progressed = true;
                    false
                } else {
                    true
                }
            });
        }
    }
}

struct DiscoveryResolver<'a> {
    discovery: &'a mut ModelDiscovery,
}

impl Resolver for DiscoveryResolver<'_> {
    fn mark_dependency(&mut self, id: &Identifier) {
        self.discovery.get_or_create_model(id);
    }
}

impl ModelWrapper {
    fn new(id: Identifier, wrapped: Arc<dyn UnbakedModel>, valid: bool) -> ModelWrapper {
        ModelWrapper {
            id,
            valid: AtomicBool::new(valid),
            parent: OnceLock::new(),
            wrapped,
            ambient_occlusion: OnceLock::new(),
            gui_light: OnceLock::new(),
            geometry: OnceLock::new(),
            texture_slots: OnceLock::new(),
            particle_sprite: OnceLock::new(),
            default_geometry: OnceLock::new(),
            model_bake_cache: Mutex::default(),
        }
    }

    fn bake_default_state(
        &self,
        texture_slots: &TextureSlots,
        baker: &dyn ModelBaker,
        state: &'static dyn ModelState,
    ) -> Arc<QuadCollection> {
        self.default_geometry
            .get_or_init(|| {
                Arc::new(
                    self.get_top_geometry()
                        .bake(texture_slots, baker, state, self),
                )
            })
            .clone()
    }
}

impl ModelDebugName for ModelWrapper {
    fn debug_name(&self) -> String {
        self.id.to_string()
    }
}

impl ResolvedModel for ModelWrapper {
    fn wrapped(&self) -> &dyn UnbakedModel {
        &*self.wrapped
    }

    fn parent(&self) -> Option<&dyn ResolvedModel> {
        self.parent
            .get()
            .map(|parent| &**parent as &dyn ResolvedModel)
    }

    fn get_top_texture_slots(&self) -> Arc<TextureSlots> {
        self.texture_slots
            .get_or_init(|| Arc::new(resolved_model::find_top_texture_slots(self)))
            .clone()
    }

    fn get_top_ambient_occlusion(&self) -> bool {
        *self
            .ambient_occlusion
            .get_or_init(|| resolved_model::find_top_ambient_occlusion(Some(self)))
    }

    fn get_top_gui_light(&self) -> GuiLight {
        *self
            .gui_light
            .get_or_init(|| resolved_model::find_top_gui_light(Some(self)))
    }

    fn get_top_geometry(&self) -> Arc<dyn UnbakedGeometry> {
        self.geometry
            .get_or_init(|| resolved_model::find_top_geometry(Some(self)))
            .clone()
    }

    /// The identity rotation's quads are kept in a slot of their own, the others by model
    /// state; either way the slots and baker of the first bake decide.
    fn bake_top_geometry(
        &self,
        texture_slots: &TextureSlots,
        baker: &dyn ModelBaker,
        state: &'static dyn ModelState,
    ) -> Arc<QuadCollection> {
        let identity: &'static dyn ModelState = BlockModelRotation::identity();
        if state.key() == identity.key() {
            return self.bake_default_state(texture_slots, baker, state);
        }
        if let Some(quads) = self
            .model_bake_cache
            .lock()
            .expect("not poisoned")
            .get(&state.key())
        {
            return quads.clone();
        }
        let quads = Arc::new(
            self.get_top_geometry()
                .bake(texture_slots, baker, state, self),
        );
        self.model_bake_cache
            .lock()
            .expect("not poisoned")
            .entry(state.key())
            .or_insert(quads)
            .clone()
    }

    fn resolve_particle_material(
        &self,
        texture_slots: &TextureSlots,
        baker: &dyn ModelBaker,
    ) -> Baked {
        self.particle_sprite
            .get_or_init(|| resolved_model::resolve_particle_material(texture_slots, baker, self))
            .clone()
    }
}
