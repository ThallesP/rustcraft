//! net/minecraft/world/level/block/CandleBlock.java
//!
//! What decides its blocks' states: its properties and its default state. Not ported yet: the rest
//! of its behaviour.
//!
//! Of its behaviour, `getFluidState` and its shape (by how many candles) are ported.
//!
//! For its parent's `animateTick`, its wicks (`getParticleOffsets`, by how many candles) are
//! ported.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::abstract_candle_block::{
    self, AbstractCandleBlock,
};
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
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
use crate::net::minecraft::world::phys::vec3::Vec3;
use std::sync::LazyLock;

pub struct CandleBlock;

pub static CLASS: BlockClass = BlockClass {
    update_shape: Some(CandleBlock::update_shape),
    get_shape: Some(CandleBlock::get_shape),
    get_fluid_state: Some(CandleBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.CandleBlock",
        &abstract_candle_block::CLASS,
    )
};

impl CandleBlock {
    pub const CANDLES: &'static IntegerProperty = BlockStateProperties::CANDLES;
    pub const LIT: &'static BooleanProperty = AbstractCandleBlock::LIT;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub fn new(properties: Properties) -> BlockData {
        CandleBlock::new_with(
            properties,
            &CLASS,
            CandleBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = AbstractCandleBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(CandleBlock::CANDLES, 1)
                .set_value(CandleBlock::LIT, false)
                .set_value(CandleBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            CandleBlock::CANDLES,
            CandleBlock::LIT,
            CandleBlock::WATERLOGGED,
        ]);
    }

    /// `LIGHT_EMISSION`: 3 for each lit candle.
    pub fn light_emission(state: BlockState) -> i32 {
        if state.get_value(CandleBlock::LIT) {
            3 * state.get_value(CandleBlock::CANDLES)
        } else {
            0
        }
    }
}

impl CandleBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}

static SHAPES: LazyLock<[VoxelShape; 4]> = LazyLock::new(|| {
    [
        Block::column(2.0, 0.0, 6.0),
        Block::r#box(5.0, 0.0, 6.0, 11.0, 6.0, 9.0),
        Block::r#box(5.0, 0.0, 6.0, 10.0, 6.0, 11.0),
        Block::r#box(5.0, 0.0, 5.0, 11.0, 6.0, 10.0),
    ]
});

impl CandleBlock {
    fn get_shape(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        SHAPES[(state.get_value(CandleBlock::CANDLES) - 1) as usize].clone()
    }
}

/// `PARTICLE_OFFSETS`: where each wick is, by how many candles there are.
static PARTICLE_OFFSETS: LazyLock<[Vec<Vec3>; 4]> = LazyLock::new(|| {
    let s = 0.0625;
    let at = |x: f64, y: f64, z: f64| Vec3::new(x, y, z).scale(s);
    [
        vec![at(8.0, 8.0, 8.0)],
        vec![at(6.0, 7.0, 8.0), at(10.0, 8.0, 7.0)],
        vec![at(8.0, 5.0, 10.0), at(6.0, 7.0, 8.0), at(9.0, 8.0, 7.0)],
        vec![
            at(7.0, 5.0, 9.0),
            at(10.0, 7.0, 9.0),
            at(6.0, 7.0, 6.0),
            at(9.0, 8.0, 6.0),
        ],
    ]
});

/// `getParticleOffsets`: the wicks of its candles.
pub(crate) fn particle_offsets(state: BlockState) -> &'static [Vec3] {
    &PARTICLE_OFFSETS[(state.get_value(CandleBlock::CANDLES) - 1) as usize]
}

impl CandleBlock {
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
