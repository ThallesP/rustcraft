//! net/minecraft/world/level/block/BrushableBlock.java
//!
//! What decides its blocks' states: its properties and its default state. Not ported yet: the rest
//! of its behaviour, and its constructor's `turnsInto`, `brushSound` and `brushCompletedSound`.
//!
//! Of its behaviour, its falling dust (`animateTick`) is ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::particles::block_particle_option::BlockParticleOption;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::falling_block::FallingBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;

pub struct BrushableBlock;

pub static CLASS: BlockClass = BlockClass {
    update_shape: Some(BrushableBlock::update_shape),
    animate_tick: Some(BrushableBlock::animate_tick),
    ..BlockClass::new(
        "net.minecraft.world.level.block.BrushableBlock",
        &base_entity_block::CLASS,
    )
};

impl BrushableBlock {
    pub const DUSTED: &'static IntegerProperty = BlockStateProperties::DUSTED;

    pub fn new(properties: Properties) -> BlockData {
        BrushableBlock::new_with(
            properties,
            &CLASS,
            BrushableBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = BaseEntityBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(BrushableBlock::DUSTED, 0),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[BrushableBlock::DUSTED]);
    }
}

impl BrushableBlock {
    /// `animateTick`: its dust falling now and then, where nothing holds it from below.
    fn animate_tick(
        _block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        if random.next_int_bounded(16) == 0 {
            let below = pos.below();
            if FallingBlock::is_free(level.get_block_state(&below)) {
                let xx = f64::from(pos.get_x()) + random.next_double();
                let yy = f64::from(pos.get_y()) - 0.05;
                let zz = f64::from(pos.get_z()) + random.next_double();
                level.add_particle(
                    BlockParticleOption::new(ParticleTypes::FALLING_DUST, state),
                    xx,
                    yy,
                    zz,
                    0.0,
                    0.0,
                    0.0,
                );
            }
        }
    }
}

impl BrushableBlock {
    /// `updateShape`: its parent's. The tick it schedules for itself goes nowhere on the client.
    #[allow(clippy::too_many_arguments)]
    fn update_shape(
        block: &BlockData,
        state: BlockState,
        level: &dyn BlockGetter,
        pos: &BlockPos,
        direction_to_neighbour: Direction,
        neighbour_pos: &BlockPos,
        neighbour_state: BlockState,
        random: &mut dyn RandomSource,
    ) -> BlockState {
        Block::super_update_shape(
            block,
            &CLASS,
            state,
            level,
            pos,
            direction_to_neighbour,
            neighbour_pos,
            neighbour_state,
            random,
        )
    }
}
