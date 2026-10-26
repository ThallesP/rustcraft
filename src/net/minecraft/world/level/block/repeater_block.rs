//! net/minecraft/world/level/block/RepeaterBlock.java
//!
//! What decides its blocks' states: its properties and its default state. Not ported yet: the rest
//! of its behaviour.
//!
//! Of its behaviour, its specks (`animateTick`) are ported.

use crate::net::minecraft::client::multiplayer::client_level::LevelWithEffects;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::core::particles::dust_particle_options::DustParticleOptions;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::diode_block::{self, DiodeBlock};
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;

pub struct RepeaterBlock;

pub static CLASS: BlockClass = BlockClass {
    update_shape: Some(RepeaterBlock::update_shape),
    should_redstone_wire_connect_to: Some(RepeaterBlock::should_redstone_wire_connect_to),
    animate_tick: Some(RepeaterBlock::animate_tick),
    ..BlockClass::new(
        "net.minecraft.world.level.block.RepeaterBlock",
        &diode_block::CLASS,
    )
};

impl RepeaterBlock {
    pub const LOCKED: &'static BooleanProperty = BlockStateProperties::LOCKED;
    pub const DELAY: &'static IntegerProperty = BlockStateProperties::DELAY;

    pub fn new(properties: Properties) -> BlockData {
        RepeaterBlock::new_with(
            properties,
            &CLASS,
            RepeaterBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = DiodeBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(HorizontalDirectionalBlock::FACING, Direction::NORTH)
                .set_value(RepeaterBlock::DELAY, 1)
                .set_value(RepeaterBlock::LOCKED, false)
                .set_value(DiodeBlock::POWERED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            HorizontalDirectionalBlock::FACING,
            RepeaterBlock::DELAY,
            RepeaterBlock::LOCKED,
            DiodeBlock::POWERED,
        ]);
    }
}

impl RepeaterBlock {
    /// `animateTick`: while powered, a redstone speck at one of its torches.
    fn animate_tick(
        _block: &BlockData,
        state: BlockState,
        level: &LevelWithEffects<'_>,
        pos: &BlockPos,
        random: &mut dyn RandomSource,
    ) {
        if state.get_value(DiodeBlock::POWERED) {
            let direction = state.get_value(HorizontalDirectionalBlock::FACING);
            let x = f64::from(pos.get_x()) + 0.5 + (random.next_double() - 0.5) * 0.2;
            let y = f64::from(pos.get_y()) + 0.4 + (random.next_double() - 0.5) * 0.2;
            let z = f64::from(pos.get_z()) + 0.5 + (random.next_double() - 0.5) * 0.2;
            let mut offset = -5.0_f32;
            if random.next_boolean() {
                offset = (state.get_value(RepeaterBlock::DELAY) * 2 - 1) as f32;
            }

            offset /= 16.0;
            let xo = f64::from(offset * direction.get_step_x() as f32);
            let zo = f64::from(offset * direction.get_step_z() as f32);
            level.add_particle(
                DustParticleOptions::REDSTONE,
                x + xo,
                y,
                z + zo,
                0.0,
                0.0,
                0.0,
            );
        }
    }
}

impl RepeaterBlock {
    /// `shouldRedstoneWireConnectTo`: wire in front of it or behind it joins it.
    fn should_redstone_wire_connect_to(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        direction: Option<Direction>,
    ) -> bool {
        let repeater_direction = state.get_value(HorizontalDirectionalBlock::FACING);
        direction == Some(repeater_direction)
            || direction == Some(repeater_direction.get_opposite())
    }
}

impl RepeaterBlock {
    /// `updateShape`: air (which a client leaves) when what's below can't hold it; its parent's
    /// otherwise. Whether it's locked by the diodes beside it is the server's to work out
    /// (`!level.isClientSide()`).
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
        if direction_to_neighbour == Direction::DOWN
            && !DiodeBlock::can_survive_on(level, neighbour_pos, neighbour_state)
        {
            Blocks::AIR.default_block_state()
        } else {
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
}
