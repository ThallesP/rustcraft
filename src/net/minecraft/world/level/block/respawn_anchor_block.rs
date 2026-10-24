//! net/minecraft/world/level/block/RespawnAnchorBlock.java
//!
//! What decides its blocks' states: its properties and its default state. Not ported yet: the rest
//! of its behaviour.
//!
//! Of its behaviour, its particles and sound (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::sounds::sound_events::SoundEvents;
use crate::net::minecraft::sounds::sound_source::SoundSource;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct RespawnAnchorBlock;

pub static CLASS: BlockClass = BlockClass {
    animate_tick: Some(RespawnAnchorBlock::animate_tick),
    ..BlockClass::new(
        "net.minecraft.world.level.block.RespawnAnchorBlock",
        &block::CLASS,
    )
};

impl RespawnAnchorBlock {
    pub const CHARGE: &'static IntegerProperty = BlockStateProperties::RESPAWN_ANCHOR_CHARGES;

    pub fn new(properties: Properties) -> BlockData {
        RespawnAnchorBlock::new_with(
            properties,
            &CLASS,
            RespawnAnchorBlock::create_block_state_definition,
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
                .get_state_definition()
                .any()
                .set_value(RespawnAnchorBlock::CHARGE, 0),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[RespawnAnchorBlock::CHARGE]);
    }

    /// Its charge above the least (0) out of 4, scaled to `maximum` and rounded down.
    pub fn get_scaled_charge_level(state: BlockState, maximum: i32) -> i32 {
        Mth::floor_f32(state.get_value(RespawnAnchorBlock::CHARGE) as f32 / 4.0 * maximum as f32)
    }
}

impl RespawnAnchorBlock {
    /// `animateTick`: charged, its sound now and then, and a reverse portal particle over it.
    fn animate_tick(
        _block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        if state.get_value(RespawnAnchorBlock::CHARGE) != 0 {
            if random.next_int_bounded(100) == 0 {
                level.play_local_sound_at(
                    pos,
                    SoundEvents::RESPAWN_ANCHOR_AMBIENT,
                    SoundSource::BLOCKS,
                    1.0,
                    1.0,
                    false,
                );
            }

            let x = f64::from(pos.get_x()) + 0.5 + (0.5 - random.next_double());
            let y = f64::from(pos.get_y()) + 1.0;
            let z = f64::from(pos.get_z()) + 0.5 + (0.5 - random.next_double());
            let ya = f64::from(random.next_float()) * 0.04;
            level.add_particle(ParticleTypes::REVERSE_PORTAL, x, y, z, 0.0, ya, 0.0);
        }
    }
}
