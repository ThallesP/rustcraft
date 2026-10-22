//! net/minecraft/world/level/block/PoweredRailBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::base_rail_block::{self, BaseRailBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::rail_shape::RailShape;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct PoweredRailBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.PoweredRailBlock",
    &base_rail_block::CLASS,
);

impl PoweredRailBlock {
    pub const SHAPE: &'static EnumProperty<RailShape> = BlockStateProperties::RAIL_SHAPE_STRAIGHT;
    pub const POWERED: &'static BooleanProperty = BlockStateProperties::POWERED;

    pub fn new(properties: Properties) -> BlockData {
        PoweredRailBlock::new_with(
            properties,
            &CLASS,
            PoweredRailBlock::create_block_state_definition,
        )
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
                .set_value(PoweredRailBlock::SHAPE, RailShape::NORTH_SOUTH)
                .set_value(PoweredRailBlock::POWERED, false)
                .set_value(BaseRailBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            PoweredRailBlock::SHAPE,
            PoweredRailBlock::POWERED,
            BaseRailBlock::WATERLOGGED,
        ]);
    }
}
