//! net/minecraft/world/level/block/AttachedStemBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `stem`, `fruit`, `seed` and `supportBlocks`.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block::vegetation_block::{self, VegetationBlock};

pub struct AttachedStemBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.AttachedStemBlock",
    &vegetation_block::CLASS,
);

impl AttachedStemBlock {
    pub const FACING: &'static EnumProperty<Direction> = HorizontalDirectionalBlock::FACING;

    pub fn new(properties: Properties) -> BlockData {
        AttachedStemBlock::new_with(
            properties,
            &CLASS,
            AttachedStemBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = VegetationBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(AttachedStemBlock::FACING, Direction::NORTH),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[AttachedStemBlock::FACING]);
    }
}
