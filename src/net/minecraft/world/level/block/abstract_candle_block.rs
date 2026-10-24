//! net/minecraft/world/level/block/AbstractCandleBlock.java
//!
//! Its blocks' states are its parent's. Not ported yet: the rest of its behaviour.
//!
//! Of its behaviour, its particles and sounds (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::sounds::sound_events::SoundEvents;
use crate::net::minecraft::sounds::sound_source::SoundSource;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block::{candle_block, candle_cake_block};
use crate::net::minecraft::world::phys::vec3::Vec3;

pub struct AbstractCandleBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(AbstractCandleBlock::animate_tick),
    ..BlockClass::new(
        "net.minecraft.world.level.block.AbstractCandleBlock",
        &block::CLASS,
    )
};

impl AbstractCandleBlock {
    pub const LIT: &'static BooleanProperty = BlockStateProperties::LIT;

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}

impl AbstractCandleBlock {
    /// `animateTick`: while lit, smoke now and then and a small flame at each wick, its subclass's offsets (`getParticleOffsets`).
    fn animate_tick(
        _block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        if state.get_value(AbstractCandleBlock::LIT) {
            for particle_pos in AbstractCandleBlock::get_particle_offsets(state) {
                AbstractCandleBlock::add_particles_and_sound(
                    level,
                    particle_pos.add_xyz(
                        f64::from(pos.get_x()),
                        f64::from(pos.get_y()),
                        f64::from(pos.get_z()),
                    ),
                    random,
                );
            }
        }
    }

    /// `getParticleOffsets(state)`, an abstract method its two subclasses implement.
    fn get_particle_offsets(state: BlockState) -> &'static [Vec3] {
        if state.get_block().class().name == candle_cake_block::CLASS.name {
            candle_cake_block::PARTICLE_OFFSETS
        } else {
            candle_block::particle_offsets(state)
        }
    }

    fn add_particles_and_sound(
        level: &LevelWithEffects<'_>,
        pos: Vec3,
        random: &mut dyn RandomSource,
    ) {
        let chance = random.next_float();
        if chance < 0.3 {
            level.add_particle(ParticleTypes::SMOKE, pos.x, pos.y, pos.z, 0.0, 0.0, 0.0);
            if chance < 0.17 {
                let volume = 1.0 + random.next_float();
                let pitch = random.next_float() * 0.7 + 0.3;
                level.play_local_sound(
                    pos.x + 0.5,
                    pos.y + 0.5,
                    pos.z + 0.5,
                    SoundEvents::CANDLE_AMBIENT,
                    SoundSource::BLOCKS,
                    volume,
                    pitch,
                    false,
                );
            }
        }

        level.add_particle(
            ParticleTypes::SMALL_FLAME,
            pos.x,
            pos.y,
            pos.z,
            0.0,
            0.0,
            0.0,
        );
    }
}
