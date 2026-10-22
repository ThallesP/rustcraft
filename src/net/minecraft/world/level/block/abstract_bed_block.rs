//! net/minecraft/world/level/block/AbstractBedBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::{
    self, HorizontalDirectionalBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::bed_part::BedPart;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct AbstractBedBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.AbstractBedBlock",
    &horizontal_directional_block::CLASS,
);

impl AbstractBedBlock {
    pub const PART: &'static EnumProperty<BedPart> = BlockStateProperties::BED_PART;
    pub const OCCUPIED: &'static BooleanProperty = BlockStateProperties::OCCUPIED;

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
                .set_value(AbstractBedBlock::PART, BedPart::FOOT)
                .set_value(AbstractBedBlock::OCCUPIED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            HorizontalDirectionalBlock::FACING,
            AbstractBedBlock::PART,
            AbstractBedBlock::OCCUPIED,
        ]);
    }
}
