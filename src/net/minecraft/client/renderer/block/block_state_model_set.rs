//! net/minecraft/client/renderer/block/BlockStateModelSet.java
//!
//! The models are by state id, which `get` reads for every block a section compiles, where Java's
//! map hashes the state: the same model for each state. Not ported yet: `getParticleMaterial`,
//! which comes with particles.

use std::collections::HashMap;
use std::sync::Arc;

use crate::net::minecraft::client::renderer::block::dispatch::block_state_model::BlockStateModel;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

/// Each block state's baked model, and the missing model for any other.
pub struct BlockStateModelSet {
    model_by_state: Vec<Option<Arc<dyn BlockStateModel>>>,
    missing_model: Arc<dyn BlockStateModel>,
}

impl BlockStateModelSet {
    // A state hashes its definition's address and its index, which never change: the definition's
    // cell is its registration, set once.
    #[allow(clippy::mutable_key_type)]
    pub fn new(
        model_by_state: HashMap<BlockState, Arc<dyn BlockStateModel>>,
        missing_model: Arc<dyn BlockStateModel>,
    ) -> BlockStateModelSet {
        let mut by_id = Vec::new();
        for (state, model) in model_by_state {
            let id = state.id() as usize;
            if by_id.len() <= id {
                by_id.resize(id + 1, None);
            }
            by_id[id] = Some(model);
        }
        BlockStateModelSet {
            model_by_state: by_id,
            missing_model,
        }
    }

    pub fn get(&self, state: BlockState) -> &Arc<dyn BlockStateModel> {
        self.model_by_state
            .get(state.id() as usize)
            .and_then(Option::as_ref)
            .unwrap_or(&self.missing_model)
    }

    pub fn missing_model(&self) -> &Arc<dyn BlockStateModel> {
        &self.missing_model
    }
}
