//! net/minecraft/world/level/block/CampfireBlock.java
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
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::entity::block_entity_ticker::BlockEntityTicker;
use crate::net::minecraft::world::level::block::entity::block_entity_type::BlockEntityType;
use crate::net::minecraft::world::level::block::entity::block_entity_types::BlockEntityTypes;
use crate::net::minecraft::world::level::block::entity::campfire_block_entity::CampfireBlockEntity;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct CampfireBlock;

/// What its constructor takes: whether it throws lava sparks, and how much its fire hurts.
#[derive(Clone, Copy, Debug)]
pub struct CampfireBlockFields {
    pub spawn_particles: bool,
    pub fire_damage: i32,
}

pub static CLASS: BlockClass = BlockClass {
    get_ticker: Some(CampfireBlock::get_ticker),
    animate_tick: Some(CampfireBlock::animate_tick),
    get_shape: Some(CampfireBlock::get_shape),
    get_fluid_state: Some(CampfireBlock::get_fluid_state),
    ..BlockClass::new(
        "net.minecraft.world.level.block.CampfireBlock",
        &base_entity_block::CLASS,
    )
};

impl CampfireBlock {
    pub const LIT: &'static BooleanProperty = BlockStateProperties::LIT;
    pub const SIGNAL_FIRE: &'static BooleanProperty = BlockStateProperties::SIGNAL_FIRE;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;
    pub const FACING: &'static EnumProperty<Direction> = BlockStateProperties::HORIZONTAL_FACING;

    /// `CampfireBlock(spawnParticles, fireDamage, properties)`.
    pub fn new(spawn_particles: bool, fire_damage: i32, properties: Properties) -> BlockData {
        CampfireBlock::new_with(
            properties,
            &CLASS,
            CampfireBlock::create_block_state_definition,
            CampfireBlockFields {
                spawn_particles,
                fire_damage,
            },
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
        fields: CampfireBlockFields,
    ) -> BlockData {
        let mut block = BaseEntityBlock::new_with(properties, class, definition);
        block.set_fields(fields);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(CampfireBlock::LIT, true)
                .set_value(CampfireBlock::SIGNAL_FIRE, false)
                .set_value(CampfireBlock::WATERLOGGED, false)
                .set_value(CampfireBlock::FACING, Direction::NORTH),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            CampfireBlock::LIT,
            CampfireBlock::SIGNAL_FIRE,
            CampfireBlock::WATERLOGGED,
            CampfireBlock::FACING,
        ]);
    }
}

impl CampfireBlock {
    /// `getFluidState`: still water when it's waterlogged.
    fn get_fluid_state(_block: &BlockData, state: BlockState) -> FluidState {
        if state.get_value(BlockStateProperties::WATERLOGGED) {
            Fluids::WATER.get_source(false)
        } else {
            Fluids::EMPTY.default_fluid_state()
        }
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(16.0, 0.0, 7.0));

impl CampfireBlock {
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

impl CampfireBlock {
    /// `makeParticles`: a puff of smoke rising from it, cosy or a signal's, with smoke on it while
    /// it's `smoking`, from the level's random.
    pub fn make_particles(
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        is_signal_fire: bool,
        smoking: bool,
    ) {
        let draw = |f: &mut dyn FnMut(&mut dyn RandomSource) -> f64| f(&mut *level.get_random());
        let sign = |random: &mut dyn RandomSource| if random.next_boolean() { 1.0 } else { -1.0 };
        let smoke_particle = if is_signal_fire {
            ParticleTypes::CAMPFIRE_SIGNAL_SMOKE
        } else {
            ParticleTypes::CAMPFIRE_COSY_SMOKE
        };
        let x = f64::from(pos.get_x())
            + 0.5
            + draw(&mut |random| random.next_double() / 3.0 * sign(random));
        let y = f64::from(pos.get_y())
            + draw(&mut |random| random.next_double())
            + draw(&mut |random| random.next_double());
        let z = f64::from(pos.get_z())
            + 0.5
            + draw(&mut |random| random.next_double() / 3.0 * sign(random));
        level.add_always_visible_particle_limited(smoke_particle, true, x, y, z, 0.0, 0.07, 0.0);
        if smoking {
            let x = f64::from(pos.get_x())
                + 0.5
                + draw(&mut |random| random.next_double() / 4.0 * sign(random));
            let z = f64::from(pos.get_z())
                + 0.5
                + draw(&mut |random| random.next_double() / 4.0 * sign(random));
            level.add_particle(
                ParticleTypes::SMOKE,
                x,
                f64::from(pos.get_y()) + 0.4,
                z,
                0.0,
                0.005,
                0.0,
            );
        }
    }

    /// `animateTick`: while lit, its crackle now and then, and, if it makes them (`spawnParticles`), sparks of lava.
    fn animate_tick(
        block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        if state.get_value(CampfireBlock::LIT) {
            if random.next_int_bounded(10) == 0 {
                let volume = 0.5 + random.next_float();
                let pitch = random.next_float() * 0.7 + 0.6;
                level.play_local_sound(
                    f64::from(pos.get_x()) + 0.5,
                    f64::from(pos.get_y()) + 0.5,
                    f64::from(pos.get_z()) + 0.5,
                    SoundEvents::CAMPFIRE_CRACKLE,
                    SoundSource::BLOCKS,
                    volume,
                    pitch,
                    false,
                );
            }

            if block.fields::<CampfireBlockFields>().spawn_particles
                && random.next_int_bounded(5) == 0
            {
                // Java's loop draws its bound again each time round.
                let mut i = 0;
                while i < random.next_int_bounded(1) + 1 {
                    let xd = f64::from(random.next_float() / 2.0);
                    let zd = f64::from(random.next_float() / 2.0);
                    level.add_particle(
                        ParticleTypes::LAVA,
                        f64::from(pos.get_x()) + 0.5,
                        f64::from(pos.get_y()) + 0.5,
                        f64::from(pos.get_z()) + 0.5,
                        xd,
                        5.0E-5,
                        zd,
                    );
                    i += 1;
                }
            }
        }
    }
}

impl CampfireBlock {
    /// `getTicker`: on the client, the smoke of a lit campfire (`particleTick`).
    fn get_ticker(
        _block: &BlockData,
        state: BlockState,
        r#type: BlockEntityType,
    ) -> Option<BlockEntityTicker> {
        if state.get_value(CampfireBlock::LIT) {
            base_entity_block::create_ticker_helper(
                r#type,
                BlockEntityTypes::CAMPFIRE,
                CampfireBlockEntity::particle_tick,
            )
        } else {
            None
        }
    }
}
