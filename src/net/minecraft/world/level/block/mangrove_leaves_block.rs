//! net/minecraft/world/level/block/MangroveLeavesBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `leafParticleChance`.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::leaves_block::LeavesBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::tinted_particle_leaves_block::{
    self, TintedParticleLeavesBlock,
};

pub struct MangroveLeavesBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.MangroveLeavesBlock",
    &tinted_particle_leaves_block::CLASS,
);

impl MangroveLeavesBlock {
    pub fn new(properties: Properties) -> BlockData {
        MangroveLeavesBlock::new_with(
            properties,
            &CLASS,
            LeavesBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        TintedParticleLeavesBlock::new_with(properties, class, definition)
    }
}
