//! net/minecraft/world/level/block/TorchBlock.java
//!
//! Its blocks' states are its parent's. Not ported yet: the rest of its behaviour.
//!
//! Of its behaviour, its smoke and flame (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::particles::particle_type::ParticleType;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::base_torch_block::{self, BaseTorchBlock};
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

pub struct TorchBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(TorchBlock::animate_tick),
    ..BlockClass::new(
        "net.minecraft.world.level.block.TorchBlock",
        &base_torch_block::CLASS,
    )
};

impl TorchBlock {
    /// `TorchBlock(flameParticle, properties)`: its flame is the block's field.
    pub fn new(flame_particle: ParticleType, properties: Properties) -> BlockData {
        TorchBlock::new_with(
            properties,
            &CLASS,
            Block::create_block_state_definition,
            flame_particle,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
        flame_particle: ParticleType,
    ) -> BlockData {
        let mut block = BaseTorchBlock::new_with(properties, class, definition);
        block.set_fields(flame_particle);
        block
    }
}

impl TorchBlock {
    /// `animateTick`: smoke and its flame (`flameParticle`) at its tip.
    fn animate_tick(
        block: &BlockData,
        _state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        _random: &mut dyn RandomSource,
    ) {
        let x = f64::from(pos.get_x()) + 0.5;
        let y = f64::from(pos.get_y()) + 0.7;
        let z = f64::from(pos.get_z()) + 0.5;
        level.add_particle(ParticleTypes::SMOKE, x, y, z, 0.0, 0.0, 0.0);
        level.add_particle(*block.fields::<ParticleType>(), x, y, z, 0.0, 0.0, 0.0);
    }
}
