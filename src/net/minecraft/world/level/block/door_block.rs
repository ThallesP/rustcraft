//! net/minecraft/world/level/block/DoorBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `type`.
//!
//! Of its behaviour, its shape (three sixteenths thick, swung toward its hinge when it's open),
//! `getSeed` (the lower half's, for both), `updateShape` and `canSurvive` are ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::{Axis, Direction};
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::door_hinge_side::DoorHingeSide;
use crate::net::minecraft::world::level::block::state::properties::double_block_half::DoubleBlockHalf;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct DoorBlock;

pub static CLASS: BlockClass = BlockClass {
    update_shape: Some(DoorBlock::update_shape),
    can_survive: Some(DoorBlock::can_survive),
    get_shape: Some(DoorBlock::get_shape),
    get_seed: Some(DoorBlock::get_seed),
    ..BlockClass::new("net.minecraft.world.level.block.DoorBlock", &block::CLASS)
};

impl DoorBlock {
    pub const FACING: &'static EnumProperty<Direction> = HorizontalDirectionalBlock::FACING;
    pub const HALF: &'static EnumProperty<DoubleBlockHalf> =
        BlockStateProperties::DOUBLE_BLOCK_HALF;
    pub const HINGE: &'static EnumProperty<DoorHingeSide> = BlockStateProperties::DOOR_HINGE;
    pub const OPEN: &'static BooleanProperty = BlockStateProperties::OPEN;
    pub const POWERED: &'static BooleanProperty = BlockStateProperties::POWERED;

    pub fn new(properties: Properties) -> BlockData {
        DoorBlock::new_with(properties, &CLASS, DoorBlock::create_block_state_definition)
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
                .set_value(DoorBlock::FACING, Direction::NORTH)
                .set_value(DoorBlock::OPEN, false)
                .set_value(DoorBlock::HINGE, DoorHingeSide::LEFT)
                .set_value(DoorBlock::POWERED, false)
                .set_value(DoorBlock::HALF, DoubleBlockHalf::LOWER),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            DoorBlock::HALF,
            DoorBlock::FACING,
            DoorBlock::OPEN,
            DoorBlock::HINGE,
            DoorBlock::POWERED,
        ]);
    }
}

static SHAPES: LazyLock<Vec<(Direction, VoxelShape)>> =
    LazyLock::new(|| Shapes::rotate_horizontal(&Block::box_z_square(16.0, 13.0, 16.0)));

impl DoorBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        let direction = state.get_value(DoorBlock::FACING);
        let door_direction = if state.get_value(DoorBlock::OPEN) {
            if state.get_value(DoorBlock::HINGE) == DoorHingeSide::RIGHT {
                direction.get_counter_clock_wise()
            } else {
                direction.get_clock_wise()
            }
        } else {
            direction
        };
        Shapes::get(&SHAPES, door_direction).clone()
    }

    fn get_seed(_block: &BlockData, state: BlockState, pos: &BlockPos) -> i64 {
        let steps = if state.get_value(DoorBlock::HALF) == DoubleBlockHalf::LOWER {
            0
        } else {
            1
        };
        #[allow(deprecated)]
        Mth::get_seed(pos.get_x(), pos.below_steps(steps).get_y(), pos.get_z())
    }
}

impl DoorBlock {
    /// `updateShape`: from its other half, which it goes with (or air, which a client leaves,
    /// when that's gone); air when the lower half has lost what holds it up; its parent's
    /// otherwise.
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
        let half = state.get_value(DoorBlock::HALF);
        if direction_to_neighbour.get_axis() != Axis::Y
            || (half == DoubleBlockHalf::LOWER) != (direction_to_neighbour == Direction::UP)
        {
            if half == DoubleBlockHalf::LOWER
                && direction_to_neighbour == Direction::DOWN
                && !state.can_survive(level, pos)
            {
                Blocks::AIR.default_block_state()
            } else {
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
        } else if neighbour_state.get_block().is_instance_of(&CLASS)
            && neighbour_state.get_value(DoorBlock::HALF) != half
        {
            neighbour_state.set_value(DoorBlock::HALF, half)
        } else {
            Blocks::AIR.default_block_state()
        }
    }

    /// `canSurvive`: the lower half on a sturdy top, the upper half on the door's lower half.
    fn can_survive(
        block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
    ) -> bool {
        let below = pos.below();
        let below_state = level.get_block_state(&below);
        if state.get_value(DoorBlock::HALF) == DoubleBlockHalf::LOWER {
            below_state.is_face_sturdy(level, &below, Direction::UP)
        } else {
            std::ptr::eq(&*below_state.get_block(), block)
        }
    }
}
