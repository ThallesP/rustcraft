//! net/minecraft/world/level/block/MyceliumBlock.java
//!
//! Its blocks' states are its parent's. Not ported yet: the rest of its behaviour.
//!
//! Of its behaviour, its spores (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::snowy_block::SnowyBlock;
use crate::net::minecraft::world::level::block::spreading_snowy_block::{
    self, SpreadingSnowyBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

pub struct MyceliumBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(MyceliumBlock::animate_tick),
    ..BlockClass::new(
        "net.minecraft.world.level.block.MyceliumBlock",
        &spreading_snowy_block::CLASS,
    )
};

impl MyceliumBlock {
    pub fn new(properties: Properties) -> BlockData {
        MyceliumBlock::new_with(
            properties,
            &CLASS,
            SnowyBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        SpreadingSnowyBlock::new_with(properties, class, definition)
    }
}

impl MyceliumBlock {
    /// `animateTick`: its parent's (nothing), and now and then a spore above it.
    fn animate_tick(
        block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        Block::super_animate_tick(block, &CLASS, state, level, pos, random);
        if random.next_int_bounded(10) == 0 {
            let x = f64::from(pos.get_x()) + random.next_double();
            let z = f64::from(pos.get_z()) + random.next_double();
            level.add_particle(
                ParticleTypes::MYCELIUM,
                x,
                f64::from(pos.get_y()) + 1.1,
                z,
                0.0,
                0.0,
                0.0,
            );
        }
    }
}
