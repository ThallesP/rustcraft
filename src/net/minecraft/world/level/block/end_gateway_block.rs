//! net/minecraft/world/level/block/EndGatewayBlock.java
//!
//! Its blocks' states are its parent's. Not ported yet: the rest of its behaviour.
//!
//! Of its behaviour, its render shape (invisible: its block entity draws it), and its portal
//! particles (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::entity::block_entity_ticker::BlockEntityTicker;
use crate::net::minecraft::world::level::block::entity::block_entity_type::BlockEntityType;
use crate::net::minecraft::world::level::block::entity::block_entity_types::BlockEntityTypes;
use crate::net::minecraft::world::level::block::entity::the_end_gateway_block_entity::TheEndGatewayBlockEntity;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_behaviour::RenderShape;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block_getter::BlockGetter;

pub struct EndGatewayBlock;

pub static CLASS: BlockClass = BlockClass {
    get_ticker: Some(EndGatewayBlock::get_ticker),
    animate_tick: Some(EndGatewayBlock::animate_tick),
    get_render_shape: Some(EndGatewayBlock::get_render_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.EndGatewayBlock",
        &base_entity_block::CLASS,
    )
};

impl EndGatewayBlock {
    pub fn new(properties: Properties) -> BlockData {
        EndGatewayBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        BaseEntityBlock::new_with(properties, class, definition)
    }
}

impl EndGatewayBlock {
    fn get_render_shape(_block: &BlockData, _state: BlockState) -> RenderShape {
        RenderShape::INVISIBLE
    }
}

impl EndGatewayBlock {
    /// `animateTick`: a portal particle for each face of its block entity that shows (`TheEndGatewayBlockEntity.getParticleAmount`, its faces' `shouldRenderFace`).
    fn animate_tick(
        _block: &BlockData,
        _state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        let Some(block_entity) = level.get_existing_block_entity(pos) else {
            return;
        };
        if block_entity.get_type() != BlockEntityTypes::END_GATEWAY {
            return;
        }
        let block_entity_state = block_entity.get_block_state();
        let particle_count = Direction::values()
            .into_iter()
            .filter(|&direction| {
                Block::should_render_face(
                    block_entity_state,
                    level.get_block_state(&pos.relative(direction)),
                    direction,
                )
            })
            .count();

        for _ in 0..particle_count {
            let mut x = f64::from(pos.get_x()) + random.next_double();
            let y = f64::from(pos.get_y()) + random.next_double();
            let mut z = f64::from(pos.get_z()) + random.next_double();
            let mut xa = (random.next_double() - 0.5) * 0.5;
            let ya = (random.next_double() - 0.5) * 0.5;
            let mut za = (random.next_double() - 0.5) * 0.5;
            let flip = random.next_int_bounded(2) * 2 - 1;
            if random.next_boolean() {
                z = f64::from(pos.get_z()) + 0.5 + 0.25 * f64::from(flip);
                za = f64::from(random.next_float() * 2.0 * flip as f32);
            } else {
                x = f64::from(pos.get_x()) + 0.5 + 0.25 * f64::from(flip);
                xa = f64::from(random.next_float() * 2.0 * flip as f32);
            }

            level.add_particle(ParticleTypes::PORTAL, x, y, z, xa, ya, za);
        }
    }
}

impl EndGatewayBlock {
    /// `getTicker`: on the client, `beamAnimationTick`.
    fn get_ticker(
        _block: &BlockData,
        _state: BlockState,
        r#type: BlockEntityType,
    ) -> Option<BlockEntityTicker> {
        base_entity_block::create_ticker_helper(
            r#type,
            BlockEntityTypes::END_GATEWAY,
            TheEndGatewayBlockEntity::beam_animation_tick,
        )
    }
}
