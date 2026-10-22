//! net/minecraft/world/level/block/WallBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::wall_side::WallSide;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct WallBlock;

pub static CLASS: BlockClass =
    BlockClass::new("net.minecraft.world.level.block.WallBlock", &block::CLASS);

impl WallBlock {
    pub const UP: &'static BooleanProperty = BlockStateProperties::UP;
    pub const EAST: &'static EnumProperty<WallSide> = BlockStateProperties::EAST_WALL;
    pub const NORTH: &'static EnumProperty<WallSide> = BlockStateProperties::NORTH_WALL;
    pub const SOUTH: &'static EnumProperty<WallSide> = BlockStateProperties::SOUTH_WALL;
    pub const WEST: &'static EnumProperty<WallSide> = BlockStateProperties::WEST_WALL;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub fn new(properties: Properties) -> BlockData {
        WallBlock::new_with(properties, &CLASS, WallBlock::create_block_state_definition)
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
                .set_value(WallBlock::UP, true)
                .set_value(WallBlock::NORTH, WallSide::NONE)
                .set_value(WallBlock::EAST, WallSide::NONE)
                .set_value(WallBlock::SOUTH, WallSide::NONE)
                .set_value(WallBlock::WEST, WallSide::NONE)
                .set_value(WallBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            WallBlock::UP,
            WallBlock::NORTH,
            WallBlock::EAST,
            WallBlock::WEST,
            WallBlock::SOUTH,
            WallBlock::WATERLOGGED,
        ]);
    }
}
