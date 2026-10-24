//! net/minecraft/world/level/block/RedStoneOreBlock.java
//!
//! What decides its blocks' states: its properties, its default state and which of its states tick.
//! Not ported yet: the rest of its behaviour.
//!
//! Of its behaviour, its specks (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::{Axis, Direction};
use crate::net::minecraft::core::particles::dust_particle_options::DustParticleOptions;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::redstone_torch_block::RedstoneTorchBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;

pub struct RedStoneOreBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(RedStoneOreBlock::animate_tick),
    is_randomly_ticking: Some(RedStoneOreBlock::is_randomly_ticking),
    ..BlockClass::new(
        "net.minecraft.world.level.block.RedStoneOreBlock",
        &block::CLASS,
    )
};

impl RedStoneOreBlock {
    pub const LIT: &'static BooleanProperty = RedstoneTorchBlock::LIT;

    pub fn new(properties: Properties) -> BlockData {
        RedStoneOreBlock::new_with(
            properties,
            &CLASS,
            RedStoneOreBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = Block::new_with(properties, class, definition);
        block.register_default_state(
            block
                .default_block_state()
                .set_value(RedStoneOreBlock::LIT, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[RedStoneOreBlock::LIT]);
    }

    fn is_randomly_ticking(_block: &BlockData, state: BlockState) -> bool {
        state.get_value(RedStoneOreBlock::LIT)
    }
}

impl RedStoneOreBlock {
    /// `animateTick`: while lit, redstone specks on its open faces.
    fn animate_tick(
        _block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        _random: &mut dyn RandomSource,
    ) {
        if state.get_value(RedStoneOreBlock::LIT) {
            RedStoneOreBlock::spawn_particles(level, pos);
        }
    }

    /// `spawnParticles`: a speck on each face whose neighbour isn't solid, from the level's
    /// random.
    fn spawn_particles(level: &LevelWithEffects<'_>, pos: &BlockPos) {
        let offset = 0.5625;

        for direction in Direction::values() {
            let relative = pos.relative(direction);
            if !level.get_block_state(&relative).is_solid_render() {
                let axis = direction.get_axis();
                let (dx, dy, dz) = {
                    let mut random = level.get_random();
                    let dx = if axis == Axis::X {
                        0.5 + offset * f64::from(direction.get_step_x())
                    } else {
                        f64::from(random.next_float())
                    };
                    let dy = if axis == Axis::Y {
                        0.5 + offset * f64::from(direction.get_step_y())
                    } else {
                        f64::from(random.next_float())
                    };
                    let dz = if axis == Axis::Z {
                        0.5 + offset * f64::from(direction.get_step_z())
                    } else {
                        f64::from(random.next_float())
                    };
                    (dx, dy, dz)
                };
                level.add_particle(
                    DustParticleOptions::REDSTONE,
                    f64::from(pos.get_x()) + dx,
                    f64::from(pos.get_y()) + dy,
                    f64::from(pos.get_z()) + dz,
                    0.0,
                    0.0,
                    0.0,
                );
            }
        }
    }
}
