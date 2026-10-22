//! net/minecraft/world/level/block/AbstractCandleBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;

pub struct AbstractCandleBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.AbstractCandleBlock",
    &block::CLASS,
);

impl AbstractCandleBlock {
    pub const LIT: &'static BooleanProperty = BlockStateProperties::LIT;

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}
