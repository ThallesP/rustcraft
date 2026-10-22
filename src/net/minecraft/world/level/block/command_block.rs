//! net/minecraft/world/level/block/CommandBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `automatic`.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::directional_block::DirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct CommandBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.CommandBlock",
    &base_entity_block::CLASS,
);

impl CommandBlock {
    pub const FACING: &'static EnumProperty<Direction> = DirectionalBlock::FACING;
    pub const CONDITIONAL: &'static BooleanProperty = BlockStateProperties::CONDITIONAL;

    pub fn new(properties: Properties) -> BlockData {
        CommandBlock::new_with(
            properties,
            &CLASS,
            CommandBlock::create_block_state_definition,
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
                .set_value(CommandBlock::FACING, Direction::NORTH)
                .set_value(CommandBlock::CONDITIONAL, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[CommandBlock::FACING, CommandBlock::CONDITIONAL]);
    }
}
