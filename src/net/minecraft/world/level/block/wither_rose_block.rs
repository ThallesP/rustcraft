//! net/minecraft/world/level/block/WitherRoseBlock.java
//!
//! Its blocks' states are its parent's. Not ported yet: the rest of its behaviour, and its
//! constructor's `mobEffect` and `effectSeconds`.
//!
//! Of its behaviour, its smoke (`animateTick`) is ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::flower_block::{self, FlowerBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::phys::shapes::collision_context;

pub struct WitherRoseBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(WitherRoseBlock::animate_tick),
    ..BlockClass::new(
        "net.minecraft.world.level.block.WitherRoseBlock",
        &flower_block::CLASS,
    )
};

impl WitherRoseBlock {
    pub fn new(properties: Properties) -> BlockData {
        WitherRoseBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        FlowerBlock::new_with(properties, class, definition)
    }
}

impl WitherRoseBlock {
    /// `animateTick`: smoke rising from its middle, now and then.
    fn animate_tick(
        _block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        let shape = state.get_shape_in(&**level, pos, &collision_context::empty());
        let shape_center = shape.bounds().get_center();
        let x = f64::from(pos.get_x()) + shape_center.x;
        let z = f64::from(pos.get_z()) + shape_center.z;

        for _ in 0..3 {
            if random.next_boolean() {
                let px = x + random.next_double() / 5.0;
                let py = f64::from(pos.get_y()) + (0.5 - random.next_double());
                let pz = z + random.next_double() / 5.0;
                level.add_particle(ParticleTypes::SMOKE, px, py, pz, 0.0, 0.0, 0.0);
            }
        }
    }
}
