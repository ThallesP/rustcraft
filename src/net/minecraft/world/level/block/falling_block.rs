//! net/minecraft/world/level/block/FallingBlock.java
//!
//! Its blocks' states are its parent's. Not ported yet: the rest of its behaviour.
//!
//! Of its behaviour, its falling dust (`animateTick`) is ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::particles::block_particle_option::BlockParticleOption;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::tags::block_tags::BlockTags;
use crate::net::minecraft::util::particle_utils::ParticleUtils;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;

pub struct FallingBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(FallingBlock::animate_tick),
    ..BlockClass::new(
        "net.minecraft.world.level.block.FallingBlock",
        &block::CLASS,
    )
};

impl FallingBlock {
    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}

impl FallingBlock {
    /// `animateTick`: now and then its dust falling, where nothing holds it from below.
    fn animate_tick(
        _block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        if random.next_int_bounded(16) == 0 {
            let below = pos.below();
            if FallingBlock::is_free(level.get_block_state(&below)) {
                ParticleUtils::spawn_particle_below(
                    level,
                    pos,
                    random,
                    BlockParticleOption::new(ParticleTypes::FALLING_DUST, state).into(),
                );
            }
        }
    }

    /// `isFree`: what a falling block falls through: air, fire, a liquid, or what can be
    /// replaced.
    pub fn is_free(state: BlockState) -> bool {
        state.is_air() || state.is_tag(BlockTags::FIRE) || state.liquid() || state.can_be_replaced()
    }
}
