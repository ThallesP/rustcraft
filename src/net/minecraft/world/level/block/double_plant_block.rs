//! net/minecraft/world/level/block/DoublePlantBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, `getSeed` (the lower half's, for both), `updateShape` and `canSurvive` are
//! ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::{Axis, Direction};
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::double_block_half::DoubleBlockHalf;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block::vegetation_block::{self, VegetationBlock};
use crate::net::minecraft::world::level::block_getter::BlockGetter;

pub struct DoublePlantBlock;

pub static CLASS: BlockClass = BlockClass {
    update_shape: Some(DoublePlantBlock::update_shape),
    can_survive: Some(DoublePlantBlock::can_survive),
    get_seed: Some(DoublePlantBlock::get_seed),
    ..BlockClass::new(
        "net.minecraft.world.level.block.DoublePlantBlock",
        &vegetation_block::CLASS,
    )
};

impl DoublePlantBlock {
    pub const HALF: &'static EnumProperty<DoubleBlockHalf> =
        BlockStateProperties::DOUBLE_BLOCK_HALF;

    pub fn new(properties: Properties) -> BlockData {
        DoublePlantBlock::new_with(
            properties,
            &CLASS,
            DoublePlantBlock::create_block_state_definition,
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
                .set_value(DoublePlantBlock::HALF, DoubleBlockHalf::LOWER),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[DoublePlantBlock::HALF]);
    }
}

impl DoublePlantBlock {
    /// `getSeed`: the lower half's, for both halves.
    fn get_seed(_block: &BlockData, state: BlockState, pos: &BlockPos) -> i64 {
        let steps =
            if state.get_value(BlockStateProperties::DOUBLE_BLOCK_HALF) == DoubleBlockHalf::LOWER {
                0
            } else {
                1
            };
        #[allow(deprecated)]
        Mth::get_seed(pos.get_x(), pos.below_steps(steps).get_y(), pos.get_z())
    }
}

impl DoublePlantBlock {
    /// `updateShape`: air (which a client leaves) when its other half is gone, or the lower half
    /// can't stay; its parent's otherwise.
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
        let half = state.get_value(DoublePlantBlock::HALF);
        if direction_to_neighbour.get_axis() != Axis::Y
            || (half == DoubleBlockHalf::LOWER) != (direction_to_neighbour == Direction::UP)
            || std::ptr::eq(&*neighbour_state.get_block(), block)
                && neighbour_state.get_value(DoublePlantBlock::HALF) != half
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
        } else {
            Blocks::AIR.default_block_state()
        }
    }

    /// `canSurvive`: the upper half on its lower half; the lower half as its parent has it.
    fn can_survive(
        block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
    ) -> bool {
        if state.get_value(DoublePlantBlock::HALF) != DoubleBlockHalf::UPPER {
            return VegetationBlock::super_can_survive(block, &CLASS, state, level, pos);
        }

        let below_state = level.get_block_state(&pos.below());
        std::ptr::eq(&*below_state.get_block(), block)
            && below_state.get_value(DoublePlantBlock::HALF) == DoubleBlockHalf::LOWER
    }
}
