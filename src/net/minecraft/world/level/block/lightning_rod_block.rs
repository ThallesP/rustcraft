//! net/minecraft/world/level/block/LightningRodBlock.java
//!
//! What decides its blocks' states: its properties and its default state. Not ported yet: the rest
//! of its behaviour.
//!
//! Of its behaviour, its sparks (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::util::particle_utils::ParticleUtils;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::util::valueproviders::uniform_int::UniformInt;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::directional_block::DirectionalBlock;
use crate::net::minecraft::world::level::block::rod_block::{self, RodBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::levelgen::heightmap::Types;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;

pub struct LightningRodBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(LightningRodBlock::animate_tick),
    get_fluid_state: Some(LightningRodBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.LightningRodBlock",
        &rod_block::CLASS,
    )
};

impl LightningRodBlock {
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;
    pub const POWERED: &'static BooleanProperty = BlockStateProperties::POWERED;

    pub fn new(properties: Properties) -> BlockData {
        LightningRodBlock::new_with(
            properties,
            &CLASS,
            LightningRodBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = RodBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(DirectionalBlock::FACING, Direction::UP)
                .set_value(LightningRodBlock::WATERLOGGED, false)
                .set_value(LightningRodBlock::POWERED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            DirectionalBlock::FACING,
            LightningRodBlock::POWERED,
            LightningRodBlock::WATERLOGGED,
        ]);
    }
}

impl LightningRodBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}

impl LightningRodBlock {
    /// `animateTick`: in a thunderstorm, sparks along it now and then, at the top of its column.
    fn animate_tick(
        _block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        _random: &mut dyn RandomSource,
    ) {
        if level.is_thundering()
            && i64::from(level.get_random().next_int_bounded(200)) <= level.get_game_time() % 200
            && pos.get_y()
                == level.get_height_at(Types::WORLD_SURFACE, pos.get_x(), pos.get_z()) - 1
        {
            ParticleUtils::spawn_particles_along_axis(
                state.get_value(DirectionalBlock::FACING).get_axis(),
                level,
                pos,
                0.125,
                ParticleTypes::ELECTRIC_SPARK.into(),
                UniformInt::of(1, 2),
            );
        }
    }
}
