//! net/minecraft/world/level/BaseSpawner.java
//!
//! A spawner's timing and range, and what it spawns next, as `load` reads them, and its client
//! tick (`clientTick`) and block event (`onEventTriggered`, which starts its delay over). The
//! entity spinning in it (`displayEntity`) is made by its renderer (`getOrCreateDisplayEntity`),
//! which isn't ported, so it has none, and its client tick only keeps its spin: Java's makes
//! particles and turns the entity once the renderer has made one. Not ported yet:
//! `SpawnPotentials` (which a server doesn't send) and `serverTick`'s spawning.

use crate::net::minecraft::client::multiplayer::client_level::LevelTicking;
use crate::net::minecraft::core::block_pos::BlockPos;
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
    spin: f64,
    o_spin: f64,
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
            spin: 0.0,
            o_spin: 0.0,
        }
    }

    /// `isNearPlayer`: an alive player within its range of its middle.
    fn is_near_player(&self, level: &LevelTicking<'_>, pos: &BlockPos) -> bool {
        level.has_nearby_alive_player(
            f64::from(pos.get_x()) + 0.5,
            f64::from(pos.get_y()) + 0.5,
            f64::from(pos.get_z()) + 0.5,
            f64::from(self.required_player_range),
        )
    }

    /// `clientTick`: without a display entity (above), the spin stays where it is.
    pub fn client_tick(&mut self, level: &LevelTicking<'_>, pos: &BlockPos) {
        if !self.is_near_player(level, pos) {
            self.o_spin = self.spin;
        }
    }

    /// `onEventTriggered`: event 1 starts the delay over, from its least.
    pub fn on_event_triggered(&mut self, id: i32) -> bool {
        if id == 1 {
            // `level.isClientSide()`.
            self.spawn_delay = self.min_spawn_delay;
            true
        } else {
            false
        }
    }

    pub fn get_spin(&self) -> f64 {
        self.spin
    }

    pub fn get_o_spin(&self) -> f64 {
        self.o_spin
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
