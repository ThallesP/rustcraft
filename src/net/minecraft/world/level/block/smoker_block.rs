//! net/minecraft/world/level/block/SmokerBlock.java
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
use crate::net::minecraft::world::level::block::abstract_furnace_block::{
    self, AbstractFurnaceBlock,
};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct SmokerBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(SmokerBlock::animate_tick),
    ..BlockClass::new(
        "net.minecraft.world.level.block.SmokerBlock",
        &abstract_furnace_block::CLASS,
    )
};

impl SmokerBlock {
    pub fn new(properties: Properties) -> BlockData {
        SmokerBlock::new_with(
            properties,
            &CLASS,
            AbstractFurnaceBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        AbstractFurnaceBlock::new_with(properties, class, definition)
    }
}

impl SmokerBlock {
    /// `animateTick`: while lit, its sound now and then, and smoke above it.
    fn animate_tick(
        _block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        if state.get_value(AbstractFurnaceBlock::LIT) {
            let x = f64::from(pos.get_x()) + 0.5;
            let y = f64::from(pos.get_y());
            let z = f64::from(pos.get_z()) + 0.5;
            if random.next_double() < 0.1 {
                level.play_local_sound(
                    x,
                    y,
                    z,
                    SoundEvents::SMOKER_SMOKE,
                    SoundSource::BLOCKS,
                    1.0,
                    1.0,
                    false,
                );
            }

            level.add_particle(ParticleTypes::SMOKE, x, y + 1.1, z, 0.0, 0.0, 0.0);
        }
    }
}
