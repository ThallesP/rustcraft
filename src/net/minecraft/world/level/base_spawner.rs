//! net/minecraft/world/level/BaseSpawner.java
//!
//! A spawner's timing and range, and what it spawns next, as `load` reads them. Not ported yet:
//! `SpawnPotentials` (which a server doesn't send), the ticks (`clientTick`'s particles and the
//! spinning entity, `serverTick`'s spawning), the display entity and `onEventTriggered`.

use crate::net::minecraft::world::level::spawn_data::SpawnData;
use crate::net::minecraft::world::level::storage::tag_value_input::TagValueInput;

#[derive(Clone, Debug, PartialEq)]
pub struct BaseSpawner {
    spawn_delay: i32,
    next_spawn_data: Option<SpawnData>,
    min_spawn_delay: i32,
    max_spawn_delay: i32,
    spawn_count: i32,
    max_nearby_entities: i32,
    required_player_range: i32,
    spawn_range: i32,
}

impl BaseSpawner {
    pub fn new() -> BaseSpawner {
        BaseSpawner {
            spawn_delay: 20,
            next_spawn_data: None,
            min_spawn_delay: 200,
            max_spawn_delay: 800,
            spawn_count: 4,
            max_nearby_entities: 6,
            required_player_range: 16,
            spawn_range: 4,
        }
    }

    /// `load(level, pos, input)`. `setNextSpawnData`'s block update (the renderer's) isn't
    /// ported.
    pub fn load(&mut self, input: &TagValueInput<'_>) {
        self.spawn_delay = input.get_short_or("Delay", 20);
        if let Some(next_spawn_data) = input.read("SpawnData", SpawnData::CODEC) {
            self.next_spawn_data = Some(next_spawn_data);
        }
        self.min_spawn_delay = input.get_int_or("MinSpawnDelay", 200);
        self.max_spawn_delay = input.get_int_or("MaxSpawnDelay", 800);
        self.spawn_count = input.get_int_or("SpawnCount", 4);
        self.max_nearby_entities = input.get_int_or("MaxNearbyEntities", 6);
        self.required_player_range = input.get_int_or("RequiredPlayerRange", 16);
        self.spawn_range = input.get_int_or("SpawnRange", 4);
    }

    pub fn spawn_delay(&self) -> i32 {
        self.spawn_delay
    }

    pub fn next_spawn_data(&self) -> Option<&SpawnData> {
        self.next_spawn_data.as_ref()
    }

    pub fn min_spawn_delay(&self) -> i32 {
        self.min_spawn_delay
    }

    pub fn max_spawn_delay(&self) -> i32 {
        self.max_spawn_delay
    }

    pub fn spawn_count(&self) -> i32 {
        self.spawn_count
    }

    pub fn max_nearby_entities(&self) -> i32 {
        self.max_nearby_entities
    }

    pub fn required_player_range(&self) -> i32 {
        self.required_player_range
    }

    pub fn spawn_range(&self) -> i32 {
        self.spawn_range
    }
}

impl Default for BaseSpawner {
    fn default() -> BaseSpawner {
        BaseSpawner::new()
    }
}
