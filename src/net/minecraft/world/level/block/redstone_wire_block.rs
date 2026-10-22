//! net/minecraft/world/level/block/RedstoneWireBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::properties::redstone_side::RedstoneSide;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct RedstoneWireBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.RedstoneWireBlock",
    &block::CLASS,
);

impl RedstoneWireBlock {
    pub const NORTH: &'static EnumProperty<RedstoneSide> = BlockStateProperties::NORTH_REDSTONE;
    pub const EAST: &'static EnumProperty<RedstoneSide> = BlockStateProperties::EAST_REDSTONE;
    pub const SOUTH: &'static EnumProperty<RedstoneSide> = BlockStateProperties::SOUTH_REDSTONE;
    pub const WEST: &'static EnumProperty<RedstoneSide> = BlockStateProperties::WEST_REDSTONE;
    pub const POWER: &'static IntegerProperty = BlockStateProperties::POWER;

    pub fn new(properties: Properties) -> BlockData {
        RedstoneWireBlock::new_with(
            properties,
            &CLASS,
            RedstoneWireBlock::create_block_state_definition,
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
                .get_state_definition()
                .any()
                .set_value(RedstoneWireBlock::NORTH, RedstoneSide::NONE)
                .set_value(RedstoneWireBlock::EAST, RedstoneSide::NONE)
                .set_value(RedstoneWireBlock::SOUTH, RedstoneSide::NONE)
                .set_value(RedstoneWireBlock::WEST, RedstoneSide::NONE)
                .set_value(RedstoneWireBlock::POWER, 0),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            RedstoneWireBlock::NORTH,
            RedstoneWireBlock::EAST,
            RedstoneWireBlock::SOUTH,
            RedstoneWireBlock::WEST,
            RedstoneWireBlock::POWER,
        ]);
    }
}
