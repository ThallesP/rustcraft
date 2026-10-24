//! net/minecraft/world/level/block/CryingObsidianBlock.java
//!
//! Its blocks' states are its parent's. Not ported yet: the rest of its behaviour.
//!
//! Of its behaviour, its tears (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;

pub struct CryingObsidianBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(CryingObsidianBlock::animate_tick),
    ..BlockClass::new(
        "net.minecraft.world.level.block.CryingObsidianBlock",
        &block::CLASS,
    )
};

impl CryingObsidianBlock {
    pub fn new(properties: Properties) -> BlockData {
        CryingObsidianBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}

impl CryingObsidianBlock {
    /// `animateTick`: now and then a tear on a side that's open.
    fn animate_tick(
        _block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        if random.next_int_bounded(5) == 0 {
            let dir = Direction::get_random(random);
            if dir != Direction::UP {
                let relative_pos = pos.relative(dir);
                let block_state = level.get_block_state(&relative_pos);
                if !state.can_occlude()
                    || !block_state.is_face_sturdy(&**level, &relative_pos, dir.get_opposite())
                {
                    let x_offset = if dir.get_step_x() == 0 {
                        random.next_double()
                    } else {
                        0.5 + f64::from(dir.get_step_x()) * 0.6
                    };
                    let y_offset = if dir.get_step_y() == 0 {
                        random.next_double()
                    } else {
                        0.5 + f64::from(dir.get_step_y()) * 0.6
                    };
                    let z_offset = if dir.get_step_z() == 0 {
                        random.next_double()
                    } else {
                        0.5 + f64::from(dir.get_step_z()) * 0.6
                    };
                    level.add_particle(
                        ParticleTypes::DRIPPING_OBSIDIAN_TEAR,
                        f64::from(pos.get_x()) + x_offset,
                        f64::from(pos.get_y()) + y_offset,
                        f64::from(pos.get_z()) + z_offset,
                        0.0,
                        0.0,
                        0.0,
                    );
                }
            }
        }
    }
}
