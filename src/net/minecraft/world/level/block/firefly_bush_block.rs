//! net/minecraft/world/level/block/FireflyBushBlock.java
//!
//! Its blocks' states are its parent's. Not ported yet: the rest of its behaviour.
//!
//! Of its behaviour, its fireflies and sound (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::sounds::sound_events::SoundEvents;
use crate::net::minecraft::sounds::sound_source::SoundSource;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::attribute::environment_attributes::EnvironmentAttributes;
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::vegetation_block::{self, VegetationBlock};
use crate::net::minecraft::world::level::levelgen::heightmap::Types;

pub struct FireflyBushBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(FireflyBushBlock::animate_tick),
    ..BlockClass::new(
        "net.minecraft.world.level.block.FireflyBushBlock",
        &vegetation_block::CLASS,
    )
};

impl FireflyBushBlock {
    pub fn new(properties: Properties) -> BlockData {
        FireflyBushBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        VegetationBlock::new_with(properties, class, definition)
    }
}

impl FireflyBushBlock {
    /// `animateTick`: its sound now and then when fireflies are about and it's open to the sky, and in the dark, fireflies around it.
    fn animate_tick(
        _block: &BlockData,
        _state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        if random.next_int_bounded(30) == 0
            && level.get_environment_value(&EnvironmentAttributes::FIREFLY_BUSH_SOUNDS, pos)
            && level.get_height_at(Types::MOTION_BLOCKING_NO_LEAVES, pos.get_x(), pos.get_z())
                <= pos.get_y()
        {
            level.play_local_sound_at(
                pos,
                SoundEvents::FIREFLY_BUSH_IDLE,
                SoundSource::AMBIENT,
                1.0,
                1.0,
                false,
            );
        }

        if level.get_max_local_raw_brightness(pos) <= 13 && random.next_double() <= 0.7 {
            let firefly_x = f64::from(pos.get_x()) + random.next_double() * 10.0 - 5.0;
            let firefly_y = f64::from(pos.get_y()) + random.next_double() * 5.0;
            let firefly_z = f64::from(pos.get_z()) + random.next_double() * 10.0 - 5.0;
            level.add_particle(
                ParticleTypes::FIREFLY,
                firefly_x,
                firefly_y,
                firefly_z,
                0.0,
                0.0,
                0.0,
            );
        }
    }
}
