//! net/minecraft/world/level/block/StainedGlassPaneBlock.java
//!
//! Only what decides its blocks' states so far: its default state. Not ported yet: its behaviour,
//! and its constructor's `color`.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::cross_collision_block::CrossCollisionBlock;
use crate::net::minecraft::world::level::block::iron_bars_block::{self, IronBarsBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct StainedGlassPaneBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.StainedGlassPaneBlock",
    &iron_bars_block::CLASS,
);

impl StainedGlassPaneBlock {
    pub fn new(properties: Properties) -> BlockData {
        StainedGlassPaneBlock::new_with(
            properties,
            &CLASS,
            IronBarsBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = IronBarsBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(CrossCollisionBlock::NORTH, false)
                .set_value(CrossCollisionBlock::EAST, false)
                .set_value(CrossCollisionBlock::SOUTH, false)
                .set_value(CrossCollisionBlock::WEST, false)
                .set_value(CrossCollisionBlock::WATERLOGGED, false),
        );
        block
    }
}
