//! net/minecraft/world/level/block/TrapDoorBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `type`.
//!
//! Of its behaviour, `getFluidState` and its shape (three sixteenths at the top or bottom, or
//! against the side it opens to) are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::{
    self, HorizontalDirectionalBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::half::Half;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct TrapDoorBlock;

pub static CLASS: BlockClass = BlockClass {
    update_shape: Some(TrapDoorBlock::update_shape),
    get_shape: Some(TrapDoorBlock::get_shape),
    get_fluid_state: Some(TrapDoorBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.TrapDoorBlock",
        &horizontal_directional_block::CLASS,
    )
};

impl TrapDoorBlock {
    pub const OPEN: &'static BooleanProperty = BlockStateProperties::OPEN;
    pub const HALF: &'static EnumProperty<Half> = BlockStateProperties::HALF;
    pub const POWERED: &'static BooleanProperty = BlockStateProperties::POWERED;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub fn new(properties: Properties) -> BlockData {
        TrapDoorBlock::new_with(
            properties,
            &CLASS,
            TrapDoorBlock::create_block_state_definition,
        )
    }

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
                .set_value(HorizontalDirectionalBlock::FACING, Direction::NORTH)
                .set_value(TrapDoorBlock::OPEN, false)
                .set_value(TrapDoorBlock::HALF, Half::BOTTOM)
                .set_value(TrapDoorBlock::POWERED, false)
                .set_value(TrapDoorBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            HorizontalDirectionalBlock::FACING,
            TrapDoorBlock::OPEN,
            TrapDoorBlock::HALF,
            TrapDoorBlock::POWERED,
            TrapDoorBlock::WATERLOGGED,
        ]);
    }
}

impl TrapDoorBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}

static SHAPES: LazyLock<Vec<(Direction, VoxelShape)>> =
    LazyLock::new(|| Shapes::rotate_all(&Block::box_z_square(16.0, 13.0, 16.0)));

impl TrapDoorBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        let direction = if state.get_value(TrapDoorBlock::OPEN) {
            state.get_value(HorizontalDirectionalBlock::FACING)
        } else if state.get_value(TrapDoorBlock::HALF) == Half::TOP {
            Direction::DOWN
        } else {
            Direction::UP
        };
        Shapes::get(&SHAPES, direction).clone()
    }
}

impl TrapDoorBlock {
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
