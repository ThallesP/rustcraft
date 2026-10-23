//! net/minecraft/world/level/block/BambooStalkBlock.java
//!
//! Only what decides its blocks' states so far: its properties, its default state and which of its
//! states tick. Not ported yet: its behaviour.
//!
//! Of its behaviour, which states tick, `propagatesSkylightDown` and its shapes (thicker with large
//! leaves, a thin collision pole, all moved by its offset; never a whole block) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::bamboo_leaves::BambooLeaves;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct BambooStalkBlock;

pub static CLASS: BlockClass = BlockClass {
    get_shape: Some(BambooStalkBlock::get_shape),
    get_collision_shape: Some(BambooStalkBlock::get_collision_shape),
    is_collision_shape_full_block: Some(BambooStalkBlock::is_collision_shape_full_block),
    propagates_skylight_down: Some(BambooStalkBlock::propagates_skylight_down),
    is_randomly_ticking: Some(BambooStalkBlock::is_randomly_ticking),
    ..BlockClass::new(
        "net.minecraft.world.level.block.BambooStalkBlock",
        &block::CLASS,
    )
};

impl BambooStalkBlock {
    pub const AGE: &'static IntegerProperty = BlockStateProperties::AGE_1;
    pub const LEAVES: &'static EnumProperty<BambooLeaves> = BlockStateProperties::BAMBOO_LEAVES;
    pub const STAGE: &'static IntegerProperty = BlockStateProperties::STAGE;

    pub fn new(properties: Properties) -> BlockData {
        BambooStalkBlock::new_with(
            properties,
            &CLASS,
            BambooStalkBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = Block::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(BambooStalkBlock::AGE, 0)
                .set_value(BambooStalkBlock::LEAVES, BambooLeaves::NONE)
                .set_value(BambooStalkBlock::STAGE, 0),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            BambooStalkBlock::AGE,
            BambooStalkBlock::LEAVES,
            BambooStalkBlock::STAGE,
        ]);
    }

    fn is_randomly_ticking(_block: &BlockData, state: BlockState) -> bool {
        state.get_value(BambooStalkBlock::STAGE) == 0
    }
}

impl BambooStalkBlock {
    /// `propagatesSkylightDown`.
    fn propagates_skylight_down(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}

static SHAPE_SMALL: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(6.0, 0.0, 16.0));
static SHAPE_LARGE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(10.0, 0.0, 16.0));
static SHAPE_COLLISION: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(3.0, 0.0, 16.0));

impl BambooStalkBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        let shape = if state.get_value(BambooStalkBlock::LEAVES) == BambooLeaves::LARGE {
            &SHAPE_LARGE
        } else {
            &SHAPE_SMALL
        };
        shape.r#move(&state.get_offset(pos))
    }

    fn get_collision_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        SHAPE_COLLISION.r#move(&state.get_offset(pos))
    }

    fn is_collision_shape_full_block(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
    ) -> bool {
        false
    }
}
