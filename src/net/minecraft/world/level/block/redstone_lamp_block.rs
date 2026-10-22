//! net/minecraft/world/level/block/RedstoneLampBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::redstone_torch_block::RedstoneTorchBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct RedstoneLampBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.RedstoneLampBlock",
    &block::CLASS,
);

impl RedstoneLampBlock {
    pub const LIT: &'static BooleanProperty = RedstoneTorchBlock::LIT;

    pub fn new(properties: Properties) -> BlockData {
        RedstoneLampBlock::new_with(
            properties,
            &CLASS,
            RedstoneLampBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = Block::new_with(properties, class, definition);
        block.register_default_state(
            block
                .default_block_state()
                .set_value(RedstoneLampBlock::LIT, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[RedstoneLampBlock::LIT]);
    }
}
