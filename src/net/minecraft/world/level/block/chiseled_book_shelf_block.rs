//! net/minecraft/world/level/block/ChiseledBookShelfBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct ChiseledBookShelfBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.ChiseledBookShelfBlock",
    &base_entity_block::CLASS,
);

impl ChiseledBookShelfBlock {
    pub const SLOT_OCCUPIED_PROPERTIES: [&'static BooleanProperty; 6] = [
        ChiseledBookShelfBlock::SLOT_0_OCCUPIED,
        ChiseledBookShelfBlock::SLOT_1_OCCUPIED,
        ChiseledBookShelfBlock::SLOT_2_OCCUPIED,
        ChiseledBookShelfBlock::SLOT_3_OCCUPIED,
        ChiseledBookShelfBlock::SLOT_4_OCCUPIED,
        ChiseledBookShelfBlock::SLOT_5_OCCUPIED,
    ];
    pub const FACING: &'static EnumProperty<Direction> = HorizontalDirectionalBlock::FACING;
    pub const SLOT_0_OCCUPIED: &'static BooleanProperty = BlockStateProperties::SLOT_0_OCCUPIED;
    pub const SLOT_1_OCCUPIED: &'static BooleanProperty = BlockStateProperties::SLOT_1_OCCUPIED;
    pub const SLOT_2_OCCUPIED: &'static BooleanProperty = BlockStateProperties::SLOT_2_OCCUPIED;
    pub const SLOT_3_OCCUPIED: &'static BooleanProperty = BlockStateProperties::SLOT_3_OCCUPIED;
    pub const SLOT_4_OCCUPIED: &'static BooleanProperty = BlockStateProperties::SLOT_4_OCCUPIED;
    pub const SLOT_5_OCCUPIED: &'static BooleanProperty = BlockStateProperties::SLOT_5_OCCUPIED;

    pub fn new(properties: Properties) -> BlockData {
        ChiseledBookShelfBlock::new_with(
            properties,
            &CLASS,
            ChiseledBookShelfBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = BaseEntityBlock::new_with(properties, class, definition);
        let mut default_state = block
            .get_state_definition()
            .any()
            .set_value(ChiseledBookShelfBlock::FACING, Direction::NORTH);
        for property in ChiseledBookShelfBlock::SLOT_OCCUPIED_PROPERTIES {
            default_state = default_state.set_value(property, false);
        }
        block.register_default_state(default_state);
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[ChiseledBookShelfBlock::FACING]);
        for property in ChiseledBookShelfBlock::SLOT_OCCUPIED_PROPERTIES {
            builder.add(&[property]);
        }
    }
}
