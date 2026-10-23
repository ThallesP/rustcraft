//! net/minecraft/world/level/block/HalfTransparentBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.
//!
//! Of its behaviour, `skipRendering` is ported: a face against the same block is left out.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

pub struct HalfTransparentBlock;

pub static CLASS: BlockClass = BlockClass {
    skip_rendering: Some(HalfTransparentBlock::skip_rendering),
    ..BlockClass::new(
        "net.minecraft.world.level.block.HalfTransparentBlock",
        &block::CLASS,
    )
};

impl HalfTransparentBlock {
    pub fn new(properties: Properties) -> BlockData {
        HalfTransparentBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}

impl HalfTransparentBlock {
    fn skip_rendering(
        _block: &BlockData,
        state: BlockState,
        neighbor_state: BlockState,
        _direction: Direction,
    ) -> bool {
        neighbor_state.is(state.get_block())
    }
}
