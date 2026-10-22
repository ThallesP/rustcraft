//! net/minecraft/world/level/block/WallHangingSignBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `type`.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::sign_block::{self, SignBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct WallHangingSignBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.WallHangingSignBlock",
    &sign_block::CLASS,
);

impl WallHangingSignBlock {
    pub const FACING: &'static EnumProperty<Direction> = HorizontalDirectionalBlock::FACING;

    pub fn new(properties: Properties) -> BlockData {
        WallHangingSignBlock::new_with(
            properties,
            &CLASS,
            WallHangingSignBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = SignBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(WallHangingSignBlock::FACING, Direction::NORTH)
                .set_value(SignBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[WallHangingSignBlock::FACING, SignBlock::WATERLOGGED]);
    }
}
