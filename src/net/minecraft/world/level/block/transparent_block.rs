//! net/minecraft/world/level/block/TransparentBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.
//!
//! Of its behaviour, `propagatesSkylightDown` is ported, for the light engine.

use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::half_transparent_block::{
    self, HalfTransparentBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

pub struct TransparentBlock;

pub static CLASS: BlockClass = BlockClass {
    propagates_skylight_down: Some(TransparentBlock::propagates_skylight_down),
    ..BlockClass::new(
        "net.minecraft.world.level.block.TransparentBlock",
        &half_transparent_block::CLASS,
    )
};

impl TransparentBlock {
    pub fn new(properties: Properties) -> BlockData {
        TransparentBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        HalfTransparentBlock::new_with(properties, class, definition)
    }
}

impl TransparentBlock {
    /// `propagatesSkylightDown`.
    fn propagates_skylight_down(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}
