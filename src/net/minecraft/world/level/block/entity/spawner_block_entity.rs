//! net/minecraft/world/level/block/entity/SpawnerBlockEntity.java
//!
//! A monster spawner: its `BaseSpawner`, which `loadAdditional` loads, the client ticks
//! (`clientTick`) and the block event starts over (`triggerEvent`). Not ported yet: the server's
//! ticker and `setEntityId`.

use crate::net::minecraft::client::multiplayer::client_level::LevelTicking;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::base_spawner::BaseSpawner;
use crate::net::minecraft::world::level::block::entity::block_entity::BlockEntityData;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::storage::tag_value_input::TagValueInput;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SpawnerBlockEntity {
    spawner: BaseSpawner,
}

impl SpawnerBlockEntity {
    pub fn new() -> SpawnerBlockEntity {
        SpawnerBlockEntity::default()
    }

    pub fn load_additional(&mut self, input: &TagValueInput<'_>) {
        self.spawner.load(input);
    }

    pub fn get_spawner(&self) -> &BaseSpawner {
        &self.spawner
    }

    /// `clientTick`, the client's ticker.
    pub fn client_tick(level: &mut LevelTicking<'_>, pos: &BlockPos, _state: BlockState) {
        let mut block_entity = level.block_entity(pos);
        if let BlockEntityData::Spawner(entity) = &mut block_entity.data {
            entity.spawner.client_tick(level, pos);
        }
    }

    pub fn trigger_event(&mut self, b0: i32, _b1: i32) -> bool {
        self.spawner.on_event_triggered(b0)
    }
}
