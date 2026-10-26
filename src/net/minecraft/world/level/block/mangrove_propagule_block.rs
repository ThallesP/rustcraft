//! net/minecraft/world/level/block/MangrovePropaguleBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `treeGrower`.
//!
//! Of its behaviour, `getFluidState` and its shape (longer as a hanging one grows, moved by its
//! offset) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::sapling_block::{self, SaplingBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct MangrovePropaguleBlock;

pub static CLASS: BlockClass = BlockClass {
    may_place_on: Some(MangrovePropaguleBlock::may_place_on),
    get_shape: Some(MangrovePropaguleBlock::get_shape),
    get_fluid_state: Some(MangrovePropaguleBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.MangrovePropaguleBlock",
        &sapling_block::CLASS,
    )
};

impl MangrovePropaguleBlock {
    pub const AGE: &'static IntegerProperty = BlockStateProperties::AGE_4;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;
    pub const HANGING: &'static BooleanProperty = BlockStateProperties::HANGING;

    pub fn new(properties: Properties) -> BlockData {
        MangrovePropaguleBlock::new_with(
            properties,
            &CLASS,
            MangrovePropaguleBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = SaplingBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(SaplingBlock::STAGE, 0)
                .set_value(MangrovePropaguleBlock::AGE, 0)
                .set_value(MangrovePropaguleBlock::WATERLOGGED, false)
                .set_value(MangrovePropaguleBlock::HANGING, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            SaplingBlock::STAGE,
            MangrovePropaguleBlock::AGE,
            MangrovePropaguleBlock::WATERLOGGED,
            MangrovePropaguleBlock::HANGING,
        ]);
    }
}

impl MangrovePropaguleBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}

const SHAPE_MIN_Y: [i32; 5] = [13, 10, 7, 3, 0];
static SHAPE_PER_AGE: LazyLock<Vec<VoxelShape>> = LazyLock::new(|| {
    Block::boxes(4, |age| {
        Block::column(2.0, f64::from(SHAPE_MIN_Y[age as usize]), 16.0)
    })
});

impl MangrovePropaguleBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        let age = if state.get_value(MangrovePropaguleBlock::HANGING) {
            state.get_value(MangrovePropaguleBlock::AGE)
        } else {
            4
        };
        SHAPE_PER_AGE[age as usize].r#move(&state.get_offset(pos))
    }
}

impl MangrovePropaguleBlock {
    /// `mayPlaceOn`: on what supports a mangrove propagule.
    fn may_place_on(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
    ) -> bool {
        state.is_tag(BlockTags::SUPPORTS_MANGROVE_PROPAGULE)
    }
}
