//! net/minecraft/world/level/block/WetSpongeBlock.java
//!
//! Its blocks' states are its parent's. Not ported yet: the rest of its behaviour.
//!
//! Of its behaviour, its drips (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::{Axis, Direction};
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;

pub struct WetSpongeBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(WetSpongeBlock::animate_tick),
    ..BlockClass::new(
        "net.minecraft.world.level.block.WetSpongeBlock",
        &block::CLASS,
    )
};

impl WetSpongeBlock {
    pub fn new(properties: Properties) -> BlockData {
        WetSpongeBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}

impl WetSpongeBlock {
    /// `animateTick`: water dripping from a side that's open, or under it.
    fn animate_tick(
        _block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        let direction = Direction::get_random(random);
        if direction != Direction::UP {
            let relative_pos = pos.relative(direction);
            let block_state = level.get_block_state(&relative_pos);
            if !state.can_occlude()
                || !block_state.is_face_sturdy(&**level, &relative_pos, direction.get_opposite())
            {
                let mut xx = f64::from(pos.get_x());
                let mut yy = f64::from(pos.get_y());
                let mut zz = f64::from(pos.get_z());
                if direction == Direction::DOWN {
                    yy -= 0.05;
                    xx += random.next_double();
                    zz += random.next_double();
                } else {
                    yy += random.next_double() * 0.8;
                    if direction.get_axis() == Axis::X {
                        zz += random.next_double();
                        if direction == Direction::EAST {
                            xx += 1.1;
                        } else {
                            xx += 0.05;
                        }
                    } else {
                        xx += random.next_double();
                        if direction == Direction::SOUTH {
                            zz += 1.1;
                        } else {
                            zz += 0.05;
                        }
                    }
                }

                level.add_particle(ParticleTypes::DRIPPING_WATER, xx, yy, zz, 0.0, 0.0, 0.0);
            }
        }
    }
}
