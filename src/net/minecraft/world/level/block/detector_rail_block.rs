//! net/minecraft/world/level/block/DetectorRailBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::base_rail_block::{self, BaseRailBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::rail_shape::RailShape;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct DetectorRailBlock;

pub static CLASS: BlockClass = BlockClass {
    is_signal_source: Some(DetectorRailBlock::is_signal_source),
    ..BlockClass::new(
        "net.minecraft.world.level.block.DetectorRailBlock",
        &base_rail_block::CLASS,
    )
};

impl DetectorRailBlock {
    pub const SHAPE: &'static EnumProperty<RailShape> = BlockStateProperties::RAIL_SHAPE_STRAIGHT;
    pub const POWERED: &'static BooleanProperty = BlockStateProperties::POWERED;

    pub fn new(properties: Properties) -> BlockData {
        DetectorRailBlock::new_with(
            properties,
            &CLASS,
            DetectorRailBlock::create_block_state_definition,
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
                .set_value(DetectorRailBlock::POWERED, false)
                .set_value(DetectorRailBlock::SHAPE, RailShape::NORTH_SOUTH)
                .set_value(BaseRailBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            DetectorRailBlock::SHAPE,
            DetectorRailBlock::POWERED,
            BaseRailBlock::WATERLOGGED,
        ]);
    }
}

impl DetectorRailBlock {
    /// `isSignalSource`: it is one.
    fn is_signal_source(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}
