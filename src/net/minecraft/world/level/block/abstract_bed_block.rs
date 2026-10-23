//! net/minecraft/world/level/block/AbstractBedBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, `getSeed` is ported: both halves take the foot's column.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::{
    self, HorizontalDirectionalBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::bed_part::BedPart;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct AbstractBedBlock;

pub static CLASS: BlockClass = BlockClass {
    get_seed: Some(AbstractBedBlock::get_seed),
    ..BlockClass::new(
        "net.minecraft.world.level.block.AbstractBedBlock",
        &horizontal_directional_block::CLASS,
    )
};

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

impl AbstractBedBlock {
    /// `getSeed`: the foot's column at the state's height, for both halves.
    fn get_seed(_block: &BlockData, state: BlockState, pos: &BlockPos) -> i64 {
        let steps = if state.get_value(BlockStateProperties::BED_PART) == BedPart::HEAD {
            0
        } else {
            1
        };
        let source_pos = pos.relative_steps(
            state.get_value(BlockStateProperties::HORIZONTAL_FACING),
            steps,
        );
        #[allow(deprecated)]
        Mth::get_seed(source_pos.get_x(), pos.get_y(), source_pos.get_z())
    }

    /// `getConnectedDirection`: toward the other half.
    pub fn get_connected_direction(
        state: BlockState,
    ) -> crate::net::minecraft::core::direction::Direction {
        let facing = state.get_value(BlockStateProperties::HORIZONTAL_FACING);
        if state.get_value(BlockStateProperties::BED_PART) == BedPart::HEAD {
            facing.get_opposite()
        } else {
            facing
        }
    }
}
