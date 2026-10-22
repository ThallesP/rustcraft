//! net/minecraft/client/multiplayer/ClientLevel.java
//!
//! Not ported yet: nearly all of it, which the world track ports with chunk storage, blocks and
//! light. So far a level is what the login said about it.

use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::resources::resource_key::ResourceKey;

/// The level the player is in, as the client knows it.
#[derive(Clone, Debug)]
pub struct ClientLevel {
    dimension: ResourceKey,
    dimension_type: Holder,
    server_chunk_radius: i32,
    server_simulation_distance: i32,
    is_debug: bool,
    biome_zoom_seed: i64,
    sea_level: i32,
    hardcore: bool,
    is_flat: bool,
}

impl ClientLevel {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        hardcore: bool,
        is_flat: bool,
        dimension: ResourceKey,
        dimension_type: Holder,
        server_chunk_radius: i32,
        server_simulation_distance: i32,
        is_debug: bool,
        biome_zoom_seed: i64,
        sea_level: i32,
    ) -> ClientLevel {
        ClientLevel {
            dimension,
            dimension_type,
            server_chunk_radius,
            server_simulation_distance,
            is_debug,
            biome_zoom_seed,
            sea_level,
            hardcore,
            is_flat,
        }
    }

    pub fn dimension(&self) -> &ResourceKey {
        &self.dimension
    }

    pub fn dimension_type_registration(&self) -> &Holder {
        &self.dimension_type
    }

    pub fn server_chunk_radius(&self) -> i32 {
        self.server_chunk_radius
    }

    pub fn get_biome_zoom_seed(&self) -> i64 {
        self.biome_zoom_seed
    }

    /// `getLevelData().isHardcore()`.
    pub fn is_hardcore(&self) -> bool {
        self.hardcore
    }

    /// `getLevelData().isFlat()`.
    pub fn is_flat(&self) -> bool {
        self.is_flat
    }

    pub fn get_sea_level(&self) -> i32 {
        self.sea_level
    }

    pub fn is_debug(&self) -> bool {
        self.is_debug
    }

    pub fn set_server_simulation_distance(&mut self, server_simulation_distance: i32) {
        self.server_simulation_distance = server_simulation_distance;
    }
}
