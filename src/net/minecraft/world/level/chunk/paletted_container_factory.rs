//! net/minecraft/world/level/chunk/PalettedContainerFactory.java
//!
//! Not ported yet: the containers' codecs (`blockStatesContainerCodec`, `biomeContainerCodec`),
//! the disk format, which the client doesn't read. The block states' strategy is one for the
//! game's lifetime, since `Block.BLOCK_STATE_REGISTRY` is; the biomes' takes the synchronized
//! registry the server sent, which the factory holds a copy of.

use std::sync::{Arc, LazyLock};

use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::id_map::IdMap;
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::core::registry_access::RegistryAccess;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::chunk::paletted_container::PalettedContainer;
use crate::net::minecraft::world::level::chunk::strategy::Strategy;

static BLOCK_STATES_STRATEGY: LazyLock<Arc<Strategy<BlockState>>> = LazyLock::new(|| {
    let registry: Arc<dyn IdMap<BlockState> + Send + Sync> =
        Arc::new(Block::block_state_registry());
    Arc::new(Strategy::create_for_block_states(registry))
});

/// How a level's sections number their block states and biomes, and what a new one holds.
#[derive(Clone)]
pub struct PalettedContainerFactory {
    block_states_strategy: Arc<Strategy<BlockState>>,
    default_block_state: BlockState,
    biome_strategy: Arc<Strategy<Holder>>,
    default_biome: Holder,
}

impl PalettedContainerFactory {
    /// Panics when the registries have no biomes or no plains, where Java throws.
    pub fn create(registries: &RegistryAccess) -> PalettedContainerFactory {
        let biomes = registries.lookup_or_throw(Registries::BIOME).clone();
        let plains = ResourceKey::create(
            Registries::BIOME,
            Identifier::with_default_namespace("plains"),
        );
        let default_biome = Holder::Reference(plains.clone());
        if biomes.get_id(&default_biome) == -1 {
            panic!(
                "Missing key in {}: {plains}",
                Registries::BIOME.identifier()
            );
        }
        let biomes: Arc<dyn IdMap<Holder> + Send + Sync> = Arc::new(biomes);
        PalettedContainerFactory {
            block_states_strategy: Arc::clone(&BLOCK_STATES_STRATEGY),
            default_block_state: Blocks::AIR.default_block_state(),
            biome_strategy: Arc::new(Strategy::create_for_biomes(biomes)),
            default_biome,
        }
    }

    pub fn block_states_strategy(&self) -> &Arc<Strategy<BlockState>> {
        &self.block_states_strategy
    }

    pub fn biome_strategy(&self) -> &Arc<Strategy<Holder>> {
        &self.biome_strategy
    }

    pub fn create_for_block_states(&self) -> PalettedContainer<BlockState> {
        PalettedContainer::new(
            self.default_block_state,
            Arc::clone(&self.block_states_strategy),
        )
    }

    pub fn create_for_biomes(&self) -> PalettedContainer<Holder> {
        PalettedContainer::new(self.default_biome.clone(), Arc::clone(&self.biome_strategy))
    }
}
