//! net/minecraft/world/level/block/ShortDryGrassBlock.java
//!
//! Its blocks' states are its parent's. Not ported yet: the rest of its behaviour.
//!
//! Of its behaviour, its shape, and its sound (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::dry_vegetation_block::{self, DryVegetationBlock};
use crate::net::minecraft::world::level::block::sounds::ambient_desert_block_sounds_player::AmbientDesertBlockSoundsPlayer;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::phys::shapes::collision_context::CollisionContext;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use std::sync::LazyLock;

pub struct ShortDryGrassBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(ShortDryGrassBlock::animate_tick),
    get_shape: Some(ShortDryGrassBlock::get_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.ShortDryGrassBlock",
        &dry_vegetation_block::CLASS,
    )
};

impl ShortDryGrassBlock {
    pub fn new(properties: Properties) -> BlockData {
        ShortDryGrassBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        DryVegetationBlock::new_with(properties, class, definition)
    }
}

static SHAPE: LazyLock<VoxelShape> = LazyLock::new(|| Block::column(12.0, 0.0, 10.0));

impl ShortDryGrassBlock {
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

impl ShortDryGrassBlock {
    /// `animateTick`: dry grass's sound now and then (`AmbientDesertBlockSoundsPlayer`).
    fn animate_tick(
        _block: &BlockData,
        _state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        AmbientDesertBlockSoundsPlayer::play_ambient_dry_grass_sounds(level, pos, random);
    }
}
