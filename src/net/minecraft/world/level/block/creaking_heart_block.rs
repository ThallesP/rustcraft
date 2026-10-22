//! net/minecraft/world/level/block/CreakingHeartBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::core::direction::Axis;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::creaking_heart_state::CreakingHeartState;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct CreakingHeartBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.CreakingHeartBlock",
    &base_entity_block::CLASS,
);

impl CreakingHeartBlock {
    pub const AXIS: &'static EnumProperty<Axis> = BlockStateProperties::AXIS;
    pub const STATE: &'static EnumProperty<CreakingHeartState> =
        BlockStateProperties::CREAKING_HEART_STATE;
    pub const NATURAL: &'static BooleanProperty = BlockStateProperties::NATURAL;

    pub fn new(properties: Properties) -> BlockData {
        CreakingHeartBlock::new_with(
            properties,
            &CLASS,
            CreakingHeartBlock::create_block_state_definition,
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
                .default_block_state()
                .set_value(CreakingHeartBlock::AXIS, Axis::Y)
                .set_value(CreakingHeartBlock::STATE, CreakingHeartState::UPROOTED)
                .set_value(CreakingHeartBlock::NATURAL, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            CreakingHeartBlock::AXIS,
            CreakingHeartBlock::STATE,
            CreakingHeartBlock::NATURAL,
        ]);
    }
}
