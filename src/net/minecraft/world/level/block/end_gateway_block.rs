//! net/minecraft/world/level/block/EndGatewayBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.
//!
//! Of its behaviour, its render shape (invisible: its block entity draws it) is ported.

use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_behaviour::RenderShape;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

pub struct EndGatewayBlock;

pub static CLASS: BlockClass = BlockClass {
    get_render_shape: Some(EndGatewayBlock::get_render_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.EndGatewayBlock",
        &base_entity_block::CLASS,
    )
};

impl EndGatewayBlock {
    pub fn new(properties: Properties) -> BlockData {
        EndGatewayBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        BaseEntityBlock::new_with(properties, class, definition)
    }
}

impl EndGatewayBlock {
    fn get_render_shape(_block: &BlockData, _state: BlockState) -> RenderShape {
        RenderShape::INVISIBLE
    }
}
