//! net/minecraft/world/level/block/RailBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::base_rail_block::{self, BaseRailBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::rail_shape::RailShape;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct RailBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.RailBlock",
    &base_rail_block::CLASS,
);

impl RailBlock {
    pub const SHAPE: &'static EnumProperty<RailShape> = BlockStateProperties::RAIL_SHAPE;

    pub fn new(properties: Properties) -> BlockData {
        RailBlock::new_with(properties, &CLASS, RailBlock::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = BaseRailBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(RailBlock::SHAPE, RailShape::NORTH_SOUTH)
                .set_value(BaseRailBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[RailBlock::SHAPE, BaseRailBlock::WATERLOGGED]);
    }
}
