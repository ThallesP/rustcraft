//! net/minecraft/world/level/block/DirectionalBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;

pub struct DirectionalBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.DirectionalBlock",
    &block::CLASS,
);

impl DirectionalBlock {
    pub const FACING: &'static EnumProperty<Direction> = BlockStateProperties::FACING;

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}
