//! net/minecraft/world/level/block/entity/SpawnerBlockEntity.java
//!
//! A monster spawner: its `BaseSpawner`, which `loadAdditional` loads. Not ported yet: the
//! tickers (`clientTick`, `serverTick`) and `setEntityId`.

use crate::net::minecraft::world::level::base_spawner::BaseSpawner;
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
}
