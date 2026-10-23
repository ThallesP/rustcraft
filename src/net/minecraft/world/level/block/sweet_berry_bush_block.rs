//! net/minecraft/world/level/block/SweetBerryBushBlock.java
//!
//! Only what decides its blocks' states so far: its properties, its default state and which of its
//! states tick. Not ported yet: its behaviour.
//!
//! Of its behaviour, which states tick and its shape (by its age) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block::vegetation_block::{self, VegetationBlock};
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct SweetBerryBushBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(SweetBerryBushBlock::get_shape),
    is_randomly_ticking: Some(SweetBerryBushBlock::is_randomly_ticking),
    ..BlockClass::new(
        "net.minecraft.world.level.block.SweetBerryBushBlock",
        &vegetation_block::CLASS,
    )
};

impl SweetBerryBushBlock {
    pub const AGE: &'static IntegerProperty = BlockStateProperties::AGE_3;

    pub fn new(properties: Properties) -> BlockData {
        SweetBerryBushBlock::new_with(
            properties,
            &CLASS,
            SweetBerryBushBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = VegetationBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(SweetBerryBushBlock::AGE, 0),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[SweetBerryBushBlock::AGE]);
    }

    fn is_randomly_ticking(_block: &BlockData, state: BlockState) -> bool {
        state.get_value(SweetBerryBushBlock::AGE) < 3
    }
}

static SHAPE_SAPLING: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(10.0, 0.0, 8.0));
static SHAPE_GROWING: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(14.0, 0.0, 16.0));

impl SweetBerryBushBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        match state.get_value(SweetBerryBushBlock::AGE) {
            0 => SHAPE_SAPLING.clone(),
            3 => Shapes::block(),
            _ => SHAPE_GROWING.clone(),
        }
    }
}
