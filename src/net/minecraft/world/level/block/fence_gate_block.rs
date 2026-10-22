//! net/minecraft/world/level/block/FenceGateBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `type`.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::{
    self, HorizontalDirectionalBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct FenceGateBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.FenceGateBlock",
    &horizontal_directional_block::CLASS,
);

impl FenceGateBlock {
    pub const OPEN: &'static BooleanProperty = BlockStateProperties::OPEN;
    pub const POWERED: &'static BooleanProperty = BlockStateProperties::POWERED;
    pub const IN_WALL: &'static BooleanProperty = BlockStateProperties::IN_WALL;

    pub fn new(properties: Properties) -> BlockData {
        FenceGateBlock::new_with(
            properties,
            &CLASS,
            FenceGateBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = HorizontalDirectionalBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(FenceGateBlock::OPEN, false)
                .set_value(FenceGateBlock::POWERED, false)
                .set_value(FenceGateBlock::IN_WALL, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            HorizontalDirectionalBlock::FACING,
            FenceGateBlock::OPEN,
            FenceGateBlock::POWERED,
            FenceGateBlock::IN_WALL,
        ]);
    }
}
