//! net/minecraft/world/level/block/DriedGhastBlock.java
//!
//! What decides its blocks' states: its properties and its default state. Not ported yet: the rest
//! of its behaviour.
//!
//! Of its behaviour, `getFluidState` and its shape, and its particles and sounds (`animateTick`)
//! are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::sounds::sound_events::SoundEvents;
use crate::net::minecraft::sounds::sound_source::SoundSource;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::{
    self, HorizontalDirectionalBlock,
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
use std::sync::LazyLock;

pub struct DriedGhastBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(DriedGhastBlock::animate_tick),
    get_shape: Some(DriedGhastBlock::get_shape),
    get_fluid_state: Some(DriedGhastBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.DriedGhastBlock",
        &horizontal_directional_block::CLASS,
    )
};

impl DriedGhastBlock {
    pub const HYDRATION_LEVEL: &'static IntegerProperty =
        BlockStateProperties::DRIED_GHAST_HYDRATION_LEVELS;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub fn new(properties: Properties) -> BlockData {
        DriedGhastBlock::new_with(
            properties,
            &CLASS,
            DriedGhastBlock::create_block_state_definition,
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
                .set_value(DriedGhastBlock::HYDRATION_LEVEL, 0)
                .set_value(DriedGhastBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            HorizontalDirectionalBlock::FACING,
            DriedGhastBlock::HYDRATION_LEVEL,
            DriedGhastBlock::WATERLOGGED,
        ]);
    }
}

impl DriedGhastBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column_xz(10.0, 10.0, 0.0, 10.0));

impl DriedGhastBlock {
    fn get_shape(
        _block: &BlockData,
        _state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        _context: &dyn CollisionContext,
    ) -> VoxelShape {
        SHAPE.clone()
    }
}

impl DriedGhastBlock {
    /// `animateTick`: dry, its sound now and then on soul sand and white smoke; in water, its other sound and happy particles.
    fn animate_tick(
        _block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        let x = f64::from(pos.get_x()) + 0.5;
        let y = f64::from(pos.get_y()) + 0.5;
        let z = f64::from(pos.get_z()) + 0.5;
        if !state.get_value(DriedGhastBlock::WATERLOGGED) {
            if random.next_int_bounded(40) == 0
                && level
                    .get_block_state(&pos.below())
                    .is_tag(BlockTags::TRIGGERS_AMBIENT_DRIED_GHAST_BLOCK_SOUNDS)
            {
                level.play_local_sound(
                    x,
                    y,
                    z,
                    SoundEvents::DRIED_GHAST_AMBIENT,
                    SoundSource::BLOCKS,
                    1.0,
                    1.0,
                    false,
                );
            }

            if random.next_int_bounded(6) == 0 {
                level.add_particle(ParticleTypes::WHITE_SMOKE, x, y, z, 0.0, 0.02, 0.0);
            }
        } else {
            if random.next_int_bounded(40) == 0 {
                level.play_local_sound(
                    x,
                    y,
                    z,
                    SoundEvents::DRIED_GHAST_AMBIENT_WATER,
                    SoundSource::BLOCKS,
                    1.0,
                    1.0,
                    false,
                );
            }

            if random.next_int_bounded(6) == 0 {
                let px = x + f64::from((random.next_float() * 2.0 - 1.0) / 3.0);
                let pz = z + f64::from((random.next_float() * 2.0 - 1.0) / 3.0);
                let yd = f64::from(random.next_float());
                level.add_particle(ParticleTypes::HAPPY_VILLAGER, px, y + 0.4, pz, 0.0, yd, 0.0);
            }
        }
    }
}
