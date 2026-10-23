//! net/minecraft/client/resources/model/ModelManager.java
//!
//! The resource reload's block models: every model file, the block states' definitions, the
//! models they need found and resolved, and the bake over the block and item atlases. Not ported
//! yet: item models and their client items, entity models, the fluids' models, the special
//! renderers' models, and the reload's threads, which only decide when each piece is ready.

use std::collections::HashMap;
use std::sync::Arc;

use crate::net::minecraft::client::resources::model::block_state_model_loader::BlockStateModelLoader;
use crate::net::minecraft::client::resources::model::cuboid::cuboid_model::CuboidModel;
use crate::net::minecraft::client::resources::model::cuboid::missing_cuboid_model::MissingCuboidModel;
use crate::net::minecraft::client::resources::model::model_bakery::{BakingResult, ModelBakery};
use crate::net::minecraft::client::resources::model::model_discovery::ModelDiscovery;
use crate::net::minecraft::client::resources::model::sprite::material_baker::MaterialBaker;
use crate::net::minecraft::client::resources::model::unbaked_model::UnbakedModel;
use crate::net::minecraft::resources::file_to_id_converter::FileToIdConverter;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::resources::resource_manager::ResourceManager;

pub struct ModelManager;

impl ModelManager {
    /// Every block state's baked model: the model files (`loadBlockModels`, a file that doesn't
    /// parse logged and left out), the block states' definitions, discovery from their roots, and
    /// the bake.
    pub fn bake_block_state_models(
        manager: &dyn ResourceManager,
        materials: &MaterialBaker<'_>,
    ) -> BakingResult {
        let lister = FileToIdConverter::json("models");
        let mut models: HashMap<Identifier, Arc<dyn UnbakedModel>> = HashMap::new();
        for (file, resource) in lister.list_matching_resources(manager) {
            let parsed = resource
                .read_all_as_string()
                .map_err(|error| error.to_string())
                .and_then(|text| CuboidModel::from_stream(&text));
            match parsed {
                Ok(model) => {
                    models.insert(lister.file_to_id(&file), Arc::new(model));
                }
                Err(error) => eprintln!("Failed to load model {file}: {error}"),
            }
        }
        let block_states = BlockStateModelLoader::load_block_states(manager);
        let mut discovery =
            ModelDiscovery::new(models, Arc::new(MissingCuboidModel::missing_model()));
        for root in block_states.models.values() {
            discovery.add_root(&**root);
        }
        let resolved = discovery.resolve();
        ModelBakery::new(block_states.models, resolved, discovery.missing_model())
            .bake_models(materials)
    }
}
