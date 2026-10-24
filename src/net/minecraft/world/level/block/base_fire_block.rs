//! net/minecraft/world/level/block/BaseFireBlock.java
//!
//! Its blocks' states are its parent's. Not ported yet: the rest of its behaviour, and its
//! constructor's `fireDamage`.
//!
//! Of its behaviour, its shape (a sixteenth on the ground), and its particles and sounds
//! (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::sounds::sound_events::SoundEvents;
use crate::net::minecraft::sounds::sound_source::SoundSource;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::{fire_block, soul_fire_block};
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct BaseFireBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(BaseFireBlock::animate_tick),
    get_shape: Some(BaseFireBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.BaseFireBlock",
        &block::CLASS,
    )
};

impl BaseFireBlock {
    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}

/// `SHAPE`, which `FireBlock` uses too.
pub static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(16.0, 0.0, 1.0));

impl BaseFireBlock {
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

impl BaseFireBlock {
    /// `animateTick`: its sound now and then, and large smoke at the burning blocks around it, or rising from it on a sturdy floor.
    fn animate_tick(
        block: &BlockData,
        _state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        if random.next_int_bounded(24) == 0 {
            let volume = 1.0 + random.next_float();
            let pitch = random.next_float() * 0.7 + 0.3;
            level.play_local_sound(
                f64::from(pos.get_x()) + 0.5,
                f64::from(pos.get_y()) + 0.5,
                f64::from(pos.get_z()) + 0.5,
                SoundEvents::FIRE_AMBIENT,
                SoundSource::BLOCKS,
                volume,
                pitch,
                false,
            );
        }

        let can_burn = |state: BlockState| BaseFireBlock::can_burn(block, state);
        let tenth = f64::from(0.1_f32);
        let below = pos.below();
        let below_state = level.get_block_state(&below);
        if !can_burn(below_state) && !below_state.is_face_sturdy(&**level, &below, Direction::UP) {
            if can_burn(level.get_block_state(&pos.west())) {
                for _ in 0..2 {
                    let xx = f64::from(pos.get_x()) + random.next_double() * tenth;
                    let yy = f64::from(pos.get_y()) + random.next_double();
                    let zz = f64::from(pos.get_z()) + random.next_double();
                    level.add_particle(ParticleTypes::LARGE_SMOKE, xx, yy, zz, 0.0, 0.0, 0.0);
                }
            }

            if can_burn(level.get_block_state(&pos.east())) {
                for _ in 0..2 {
                    let xx = f64::from(pos.get_x() + 1) - random.next_double() * tenth;
                    let yy = f64::from(pos.get_y()) + random.next_double();
                    let zz = f64::from(pos.get_z()) + random.next_double();
                    level.add_particle(ParticleTypes::LARGE_SMOKE, xx, yy, zz, 0.0, 0.0, 0.0);
                }
            }

            if can_burn(level.get_block_state(&pos.north())) {
                for _ in 0..2 {
                    let xx = f64::from(pos.get_x()) + random.next_double();
                    let yy = f64::from(pos.get_y()) + random.next_double();
                    let zz = f64::from(pos.get_z()) + random.next_double() * tenth;
                    level.add_particle(ParticleTypes::LARGE_SMOKE, xx, yy, zz, 0.0, 0.0, 0.0);
                }
            }

            if can_burn(level.get_block_state(&pos.south())) {
                for _ in 0..2 {
                    let xx = f64::from(pos.get_x()) + random.next_double();
                    let yy = f64::from(pos.get_y()) + random.next_double();
                    let zz = f64::from(pos.get_z() + 1) - random.next_double() * tenth;
                    level.add_particle(ParticleTypes::LARGE_SMOKE, xx, yy, zz, 0.0, 0.0, 0.0);
                }
            }

            if can_burn(level.get_block_state(&pos.above())) {
                for _ in 0..2 {
                    let xx = f64::from(pos.get_x()) + random.next_double();
                    let yy = f64::from(pos.get_y() + 1) - random.next_double() * tenth;
                    let zz = f64::from(pos.get_z()) + random.next_double();
                    level.add_particle(ParticleTypes::LARGE_SMOKE, xx, yy, zz, 0.0, 0.0, 0.0);
                }
            }
        } else {
            for _ in 0..3 {
                let xx = f64::from(pos.get_x()) + random.next_double();
                let yy = f64::from(pos.get_y()) + random.next_double() * 0.5 + 0.5;
                let zz = f64::from(pos.get_z()) + random.next_double();
                level.add_particle(ParticleTypes::LARGE_SMOKE, xx, yy, zz, 0.0, 0.0, 0.0);
            }
        }
    }

    /// `canBurn(state)`, an abstract method its two subclasses implement: fire burns what catches
    /// fire, soul fire anything.
    fn can_burn(block: &BlockData, state: BlockState) -> bool {
        if block.class().name == soul_fire_block::CLASS.name {
            true
        } else {
            fire_block::FireBlock::can_burn(state)
        }
    }
}
