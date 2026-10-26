//! net/minecraft/world/level/block/SmallDripleafBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, `getFluidState`, its shape and its offset's height (`getMaxVerticalOffset`)
//! are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::double_plant_block::{self, DoublePlantBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::double_block_half::DoubleBlockHalf;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block::vegetation_block::VegetationBlock;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct SmallDripleafBlock;

pub static CLASS: BlockClass = BlockClass {
    can_survive: Some(SmallDripleafBlock::can_survive),
    may_place_on: Some(SmallDripleafBlock::may_place_on),
    update_shape: Some(SmallDripleafBlock::update_shape),
    get_shape: Some(SmallDripleafBlock::get_shape),
    get_max_vertical_offset: Some(SmallDripleafBlock::get_max_vertical_offset),
    get_fluid_state: Some(SmallDripleafBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.SmallDripleafBlock",
        &double_plant_block::CLASS,
    )
};

impl SmallDripleafBlock {
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;
    pub const FACING: &'static EnumProperty<Direction> = BlockStateProperties::HORIZONTAL_FACING;

    pub fn new(properties: Properties) -> BlockData {
        SmallDripleafBlock::new_with(
            properties,
            &CLASS,
            SmallDripleafBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = DoublePlantBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(DoublePlantBlock::HALF, DoubleBlockHalf::LOWER)
                .set_value(SmallDripleafBlock::WATERLOGGED, false)
                .set_value(SmallDripleafBlock::FACING, Direction::NORTH),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            DoublePlantBlock::HALF,
            SmallDripleafBlock::WATERLOGGED,
            SmallDripleafBlock::FACING,
        ]);
    }
}

impl SmallDripleafBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(12.0, 0.0, 13.0));

impl SmallDripleafBlock {
    fn get_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        SHAPE.clone()
    }

    fn get_max_vertical_offset(_block: &BlockData) -> f32 {
        0.1
    }
}

impl SmallDripleafBlock {
    /// `updateShape`: its parent's. The water's tick it schedules when it's waterlogged goes
    /// nowhere on the client.
    #[allow(clippy::too_many_arguments)]
    fn update_shape(
        block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        direction_to_neighbour: Direction,
        neighbour_pos: &BlockPos,
        neighbour_state: BlockState,
        random: &mut dyn RandomSource,
    ) -> BlockState {
        Block::super_update_shape(
            block,
            &CLASS,
            state,
            level,
            pos,
            direction_to_neighbour,
            neighbour_pos,
            neighbour_state,
            random,
        )
    }
}

impl SmallDripleafBlock {
    /// `mayPlaceOn`: on what supports small dripleaf, or under still water where its parent's would
    /// stand.
    fn may_place_on(
        block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
    ) -> bool {
        state.is_tag(BlockTags::SUPPORTS_SMALL_DRIPLEAF)
            || level
                .get_fluid_state(&pos.above())
                .is_source_of_type(Fluids::WATER)
                && VegetationBlock::super_may_place_on(block, &CLASS, state, level, pos)
    }
}

impl SmallDripleafBlock {
    /// `canSurvive`: the upper half as its parent has it; the lower half on what it may be
    /// placed on.
    fn can_survive(
        block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
    ) -> bool {
        if state.get_value(DoublePlantBlock::HALF) == DoubleBlockHalf::UPPER {
            return VegetationBlock::super_can_survive(block, &CLASS, state, level, pos);
        }

        let below_pos = pos.below();
        let below_state = level.get_block_state(&below_pos);
        VegetationBlock::may_place_on(block, below_state, level, &below_pos)
    }
}
