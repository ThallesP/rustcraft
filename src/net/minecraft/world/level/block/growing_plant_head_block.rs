//! net/minecraft/world/level/block/GrowingPlantHeadBlock.java
//!
//! Only what decides its blocks' states so far: its properties, its default state and which of its
//! states tick. Not ported yet: its behaviour, and its constructor's `growthDirection`, `shape`,
//! `scheduleFluidTicks` and `growPerTickProbability`.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::growing_plant_block::{self, GrowingPlantBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;

pub struct GrowingPlantHeadBlock;

pub static CLASS: BlockClass = BlockClass {
    is_randomly_ticking: Some(GrowingPlantHeadBlock::is_randomly_ticking),
    ..BlockClass::new(
        "net.minecraft.world.level.block.GrowingPlantHeadBlock",
        &growing_plant_block::CLASS,
    )
};

impl GrowingPlantHeadBlock {
    pub const AGE: &'static IntegerProperty = BlockStateProperties::AGE_25;

    pub(crate) fn new_with(
        properties: Properties,
        shape: VoxelShape,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = GrowingPlantBlock::new_with(properties, shape, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(GrowingPlantHeadBlock::AGE, 0),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[GrowingPlantHeadBlock::AGE]);
    }

    fn is_randomly_ticking(_block: &BlockData, state: BlockState) -> bool {
        state.get_value(GrowingPlantHeadBlock::AGE) < 25
    }
}
