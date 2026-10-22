//! net/minecraft/world/level/block/DoorBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `type`.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::door_hinge_side::DoorHingeSide;
use crate::net::minecraft::world::level::block::state::properties::double_block_half::DoubleBlockHalf;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct DoorBlock;

pub static CLASS: BlockClass =
    BlockClass::new("net.minecraft.world.level.block.DoorBlock", &block::CLASS);

impl DoorBlock {
    pub const FACING: &'static EnumProperty<Direction> = HorizontalDirectionalBlock::FACING;
    pub const HALF: &'static EnumProperty<DoubleBlockHalf> =
        BlockStateProperties::DOUBLE_BLOCK_HALF;
    pub const HINGE: &'static EnumProperty<DoorHingeSide> = BlockStateProperties::DOOR_HINGE;
    pub const OPEN: &'static BooleanProperty = BlockStateProperties::OPEN;
    pub const POWERED: &'static BooleanProperty = BlockStateProperties::POWERED;

    pub fn new(properties: Properties) -> BlockData {
        DoorBlock::new_with(properties, &CLASS, DoorBlock::create_block_state_definition)
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
                .set_value(DoorBlock::FACING, Direction::NORTH)
                .set_value(DoorBlock::OPEN, false)
                .set_value(DoorBlock::HINGE, DoorHingeSide::LEFT)
                .set_value(DoorBlock::POWERED, false)
                .set_value(DoorBlock::HALF, DoubleBlockHalf::LOWER),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            DoorBlock::HALF,
            DoorBlock::FACING,
            DoorBlock::OPEN,
            DoorBlock::HINGE,
            DoorBlock::POWERED,
        ]);
    }
}
