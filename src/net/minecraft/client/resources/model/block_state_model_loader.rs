//! net/minecraft/client/resources/model/BlockStateModelLoader.java
//!
//! Java loads each definition on an executor and gathers them into an `IdentityHashMap` by state;
//! here they're loaded in turn into a map by state.

use std::collections::HashMap;
use std::sync::Arc;

use crate::com::google::gson::json_parser::JsonParser;
use crate::net::minecraft::client::renderer::block::dispatch::block_state_model::UnbakedRoot;
use crate::net::minecraft::client::renderer::block::dispatch::block_state_model_dispatcher::BlockStateModelDispatcher;
use crate::net::minecraft::client::resources::model::block_state_definitions::BlockStateDefinitions;
use crate::net::minecraft::resources::file_to_id_converter::FileToIdConverter;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::resources::resource_manager::ResourceManager;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::state_definition::StateDefinition;

pub struct BlockStateModelLoader;

/// `BlockStateModelLoader.LoadedModels`.
pub struct LoadedModels {
    pub models: HashMap<BlockState, Arc<dyn UnbakedRoot>>,
}

struct LoadedBlockStateModelDispatcher {
    source: String,
    contents: BlockStateModelDispatcher,
}

impl BlockStateModelLoader {
    /// Every block's definition in every pack, bottom first; a definition that doesn't parse is
    /// logged and left out, and one for no block is ignored.
    // A state hashes its definition's address and its index, which never change: the definition's
    // cell is its registration, set once.
    #[allow(clippy::mutable_key_type)]
    pub fn load_block_states(manager: &dyn ResourceManager) -> LoadedModels {
        let blockstate_lister = FileToIdConverter::json("blockstates");
        let definition_to_block_state =
            BlockStateDefinitions::definition_location_to_block_state_mapper();
        let mut full_map = HashMap::new();
        for (file, stack) in blockstate_lister.list_matching_resource_stacks(manager) {
            let state_definition_id = blockstate_lister.file_to_id(&file);
            let Some(state_definition) = definition_to_block_state(&state_definition_id) else {
                continue;
            };

            let mut loaded_stack = Vec::with_capacity(stack.len());
            for resource in &stack {
                let parsed = resource
                    .read_all_as_string()
                    .map_err(|error| error.to_string())
                    .and_then(|text| JsonParser::parse_strict(&text))
                    .and_then(|element| BlockStateModelDispatcher::parse(&element));
                match parsed {
                    Ok(definition) => loaded_stack.push(LoadedBlockStateModelDispatcher {
                        source: resource.source_pack_id().to_owned(),
                        contents: definition,
                    }),
                    Err(error) => eprintln!(
                        "Failed to load blockstate definition {state_definition_id} from pack {}: {error}",
                        resource.source_pack_id()
                    ),
                }
            }

            match BlockStateModelLoader::load_block_state_definition_stack(
                &state_definition_id,
                state_definition,
                &loaded_stack,
            ) {
                Ok(models) => full_map.extend(models.models),
                Err(error) => {
                    eprintln!("Failed to load blockstate definition {state_definition_id}: {error}")
                }
            }
        }
        LoadedModels { models: full_map }
    }

    // A state hashes its definition's address and its index, which never change: the definition's
    // cell is its registration, set once.
    #[allow(clippy::mutable_key_type)]
    fn load_block_state_definition_stack(
        state_definition_id: &Identifier,
        state_definition: &'static StateDefinition,
        definition_stack: &[LoadedBlockStateModelDispatcher],
    ) -> Result<LoadedModels, String> {
        let mut result = HashMap::new();
        for definition in definition_stack {
            result.extend(definition.contents.instantiate(state_definition, &|| {
                format!("{state_definition_id}/{}", definition.source)
            })?);
        }
        Ok(LoadedModels { models: result })
    }
}
