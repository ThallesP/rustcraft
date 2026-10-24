//! net/minecraft/world/level/block/FallingParticlesLeavesBlock.java
//!
//! Its blocks' states are its parent's. Not ported yet: the rest of its behaviour.
//!
//! Of its behaviour, its falling leaves (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::particles::color_particle_option::ColorParticleOption;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::util::particle_utils::ParticleUtils;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::leaves_block::LeavesBlockFields;
use crate::net::minecraft::world::level::block::leaves_block::{self, LeavesBlock};
use crate::net::minecraft::world::level::block::sounds::ambient_leaves_block_sound_player::AmbientLeavesBlockSoundPlayer;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;

pub struct FallingParticlesLeavesBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(FallingParticlesLeavesBlock::animate_tick),
    ..BlockClass::new(
        "net.minecraft.world.level.block.FallingParticlesLeavesBlock",
        &leaves_block::CLASS,
    )
};

impl FallingParticlesLeavesBlock {
    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
        leaf_particle_chance: f32,
        ambient_leaves_block_sound_player: AmbientLeavesBlockSoundPlayer,
    ) -> BlockData {
        let mut block = LeavesBlock::new_with(
            properties,
            class,
            definition,
            ambient_leaves_block_sound_player,
        );
        let mut fields = *block.fields::<LeavesBlockFields>();
        fields.leaf_particle_chance = leaf_particle_chance;
        block.set_fields(fields);
        block
    }
}

impl FallingParticlesLeavesBlock {
    /// `animateTick`: its parent's, then now and then a leaf falling from it (`makeFallingLeavesParticles`).
    fn animate_tick(
        block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        Block::super_animate_tick(block, &CLASS, state, level, pos, random);
        FallingParticlesLeavesBlock::make_falling_leaves_particles(block, level, pos, random);
    }

    fn make_falling_leaves_particles(
        block: &BlockData,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        let below = pos.below();
        let below_state = level.get_block_state(&below);
        let fields = block.fields::<LeavesBlockFields>();
        if random.next_float() < fields.leaf_particle_chance
            && !Block::is_face_full(
                &below_state.get_collision_shape(&**level, &below),
                Direction::UP,
            )
        {
            FallingParticlesLeavesBlock::spawn_falling_leaves_particle(block, level, pos, random);
        }
    }

    /// `spawnFallingLeavesParticle`, an abstract method its subclasses implement: a tinted leaf
    /// of the block's colour there (`TintedParticleLeavesBlock`, and the mangrove's), or the
    /// particle the block was made with (`UntintedParticleLeavesBlock`).
    fn spawn_falling_leaves_particle(
        block: &BlockData,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        let particle = match block.fields::<LeavesBlockFields>().leaf_particle {
            Some(leaf_particle) => leaf_particle,
            None => ColorParticleOption::create(
                ParticleTypes::TINTED_LEAVES,
                level.get_client_leaf_tint_color(pos),
            )
            .into(),
        };
        ParticleUtils::spawn_particle_below(level, pos, random, particle);
    }
}
