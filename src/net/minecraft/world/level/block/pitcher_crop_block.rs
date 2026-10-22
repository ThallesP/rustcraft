//! net/minecraft/world/level/block/PitcherCropBlock.java
//!
//! Only what decides its blocks' states so far: its properties, its default state and which of its
//! states tick. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::double_plant_block::{self, DoublePlantBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::double_block_half::DoubleBlockHalf;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct PitcherCropBlock;

pub static CLASS: BlockClass = BlockClass {
    is_randomly_ticking: Some(PitcherCropBlock::is_randomly_ticking),
    ..BlockClass::new(
        "net.minecraft.world.level.block.PitcherCropBlock",
        &double_plant_block::CLASS,
    )
};

impl PitcherCropBlock {
    pub const AGE: &'static IntegerProperty = BlockStateProperties::AGE_4;
    pub const HALF: &'static EnumProperty<DoubleBlockHalf> = DoublePlantBlock::HALF;

    pub fn new(properties: Properties) -> BlockData {
        PitcherCropBlock::new_with(
            properties,
            &CLASS,
            PitcherCropBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        DoublePlantBlock::new_with(properties, class, definition)
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[PitcherCropBlock::AGE]);
        DoublePlantBlock::create_block_state_definition(builder);
    }

    /// The lower half ticks until it's grown.
    fn is_randomly_ticking(_block: &BlockData, state: BlockState) -> bool {
        state.get_value(PitcherCropBlock::HALF) == DoubleBlockHalf::LOWER
            && !PitcherCropBlock::is_max_age(state)
    }

    fn is_max_age(state: BlockState) -> bool {
        state.get_value(PitcherCropBlock::AGE) >= 4
    }
}
