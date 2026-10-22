//! net/minecraft/world/level/block/WallSkullBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `type`.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::abstract_skull_block::{self, AbstractSkullBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct WallSkullBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.WallSkullBlock",
    &abstract_skull_block::CLASS,
);

impl WallSkullBlock {
    pub const FACING: &'static EnumProperty<Direction> = HorizontalDirectionalBlock::FACING;

    pub fn new(properties: Properties) -> BlockData {
        WallSkullBlock::new_with(
            properties,
            &CLASS,
            WallSkullBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = AbstractSkullBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .default_block_state()
                .set_value(WallSkullBlock::FACING, Direction::NORTH),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        AbstractSkullBlock::create_block_state_definition(builder);
        builder.add(&[WallSkullBlock::FACING]);
    }
}
