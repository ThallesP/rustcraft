//! net/minecraft/world/level/block/piston/MovingPistonBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::directional_block::DirectionalBlock;
use crate::net::minecraft::world::level::block::piston::piston_head_block::PistonHeadBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::piston_type::PistonType;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct MovingPistonBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.piston.MovingPistonBlock",
    &base_entity_block::CLASS,
);

impl MovingPistonBlock {
    pub const FACING: &'static EnumProperty<Direction> = DirectionalBlock::FACING;
    pub const TYPE: &'static EnumProperty<PistonType> = PistonHeadBlock::TYPE;

    pub fn new(properties: Properties) -> BlockData {
        MovingPistonBlock::new_with(
            properties,
            &CLASS,
            MovingPistonBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = BaseEntityBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(MovingPistonBlock::FACING, Direction::NORTH)
                .set_value(MovingPistonBlock::TYPE, PistonType::DEFAULT),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[MovingPistonBlock::FACING, MovingPistonBlock::TYPE]);
    }
}
