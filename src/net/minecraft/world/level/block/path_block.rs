//! net/minecraft/world/level/block/PathBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `baseBlock`.
//!
//! Of its behaviour, `useShapeForLightOcclusion` is ported, for the light engine.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

pub struct PathBlock;

pub static CLASS: BlockClass = BlockClass {
    use_shape_for_light_occlusion: Some(PathBlock::use_shape_for_light_occlusion),
    ..BlockClass::new("net.minecraft.world.level.block.PathBlock", &block::CLASS)
};

impl PathBlock {
    pub fn new(properties: Properties) -> BlockData {
        PathBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}

impl PathBlock {
    /// `useShapeForLightOcclusion`.
    fn use_shape_for_light_occlusion(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}
