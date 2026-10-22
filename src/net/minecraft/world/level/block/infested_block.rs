//! net/minecraft/world/level/block/InfestedBlock.java
//!
//! Only what decides its blocks' states so far: what its constructor makes of its host block's
//! hardness. Not ported yet: the rest, including its `hostBlock` field and `BLOCK_BY_HOST_BLOCK`,
//! which its constructor fills.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;

pub struct InfestedBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.InfestedBlock",
    &block::CLASS,
);

impl InfestedBlock {
    pub fn new(host_block: &BlockData, properties: Properties) -> BlockData {
        InfestedBlock::new_with(
            host_block,
            properties,
            &CLASS,
            Block::create_block_state_definition,
        )
    }

    /// Half as hard to break as its host, and easy to blow up.
    pub(crate) fn new_with(
        host_block: &BlockData,
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(
            properties
                .destroy_time(host_block.default_destroy_time() / 2.0)
                .explosion_resistance(0.75),
            class,
            definition,
        )
    }
}
