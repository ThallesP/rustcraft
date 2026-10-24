//! net/minecraft/world/level/block/EndRodBlock.java
//!
//! What decides its blocks' states: its properties and its default state. Not ported yet: the rest
//! of its behaviour.
//!
//! Of its behaviour, its glints (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::directional_block::DirectionalBlock;
use crate::net::minecraft::world::level::block::rod_block::{self, RodBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct EndRodBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(EndRodBlock::animate_tick),
    ..BlockClass::new(
        "net.minecraft.world.level.block.EndRodBlock",
        &rod_block::CLASS,
    )
};

impl EndRodBlock {
    pub fn new(properties: Properties) -> BlockData {
        EndRodBlock::new_with(
            properties,
            &CLASS,
            EndRodBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = RodBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(DirectionalBlock::FACING, Direction::UP),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[DirectionalBlock::FACING]);
    }
}

impl EndRodBlock {
    /// `animateTick`: now and then a glint along it.
    fn animate_tick(
        _block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        let direction = state.get_value(DirectionalBlock::FACING);
        let x = f64::from(pos.get_x()) + 0.55 - f64::from(random.next_float() * 0.1);
        let y = f64::from(pos.get_y()) + 0.55 - f64::from(random.next_float() * 0.1);
        let z = f64::from(pos.get_z()) + 0.55 - f64::from(random.next_float() * 0.1);
        let r = f64::from(0.4 - (random.next_float() + random.next_float()) * 0.4);
        if random.next_int_bounded(5) == 0 {
            let xd = random.next_gaussian() * 0.005;
            let yd = random.next_gaussian() * 0.005;
            let zd = random.next_gaussian() * 0.005;
            level.add_particle(
                ParticleTypes::END_ROD,
                x + f64::from(direction.get_step_x()) * r,
                y + f64::from(direction.get_step_y()) * r,
                z + f64::from(direction.get_step_z()) * r,
                xd,
                yd,
                zd,
            );
        }
    }
}
