//! net/minecraft/world/level/block/SandBlock.java
//!
//! Its blocks' states are its parent's. Not ported yet: the rest of its behaviour, and its
//! constructor's `dustColor`.
//!
//! Of its behaviour, its dust and sound (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::colored_falling_block::{
    self, ColoredFallingBlock,
};
use crate::net::minecraft::world::level::block::sounds::ambient_desert_block_sounds_player::AmbientDesertBlockSoundsPlayer;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

pub struct SandBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(SandBlock::animate_tick),
    ..BlockClass::new(
        "net.minecraft.world.level.block.SandBlock",
        &colored_falling_block::CLASS,
    )
};

impl SandBlock {
    pub fn new(properties: Properties) -> BlockData {
        SandBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        ColoredFallingBlock::new_with(properties, class, definition)
    }
}

impl SandBlock {
    /// `animateTick`: its parent's falling dust, and sand's sound now and then in a desert.
    fn animate_tick(
        block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        Block::super_animate_tick(block, &CLASS, state, level, pos, random);
        AmbientDesertBlockSoundsPlayer::play_ambient_sand_sounds(level, pos, random);
    }
}
