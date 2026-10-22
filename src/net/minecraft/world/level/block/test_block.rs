//! net/minecraft/world/level/block/TestBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::test_block_mode::TestBlockMode;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;

pub struct TestBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.TestBlock",
    &base_entity_block::CLASS,
);

impl TestBlock {
    pub const MODE: &'static EnumProperty<TestBlockMode> = BlockStateProperties::TEST_BLOCK_MODE;

    pub fn new(properties: Properties) -> BlockData {
        TestBlock::new_with(properties, &CLASS, TestBlock::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        BaseEntityBlock::new_with(properties, class, definition)
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[TestBlock::MODE]);
    }
}
