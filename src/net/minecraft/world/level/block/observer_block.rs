//! net/minecraft/world/level/block/ObserverBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::directional_block::{self, DirectionalBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block_getter::BlockGetter;

pub struct ObserverBlock;

pub static CLASS: BlockClass = BlockClass {
    should_redstone_wire_connect_to: Some(ObserverBlock::should_redstone_wire_connect_to),
    is_signal_source: Some(ObserverBlock::is_signal_source),
    update_shape: Some(ObserverBlock::update_shape),
    ..BlockClass::new(
        "net.minecraft.world.level.block.ObserverBlock",
        &directional_block::CLASS,
    )
};

impl ObserverBlock {
    pub const POWERED: &'static BooleanProperty = BlockStateProperties::POWERED;

    pub fn new(properties: Properties) -> BlockData {
        ObserverBlock::new_with(
            properties,
            &CLASS,
            ObserverBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = DirectionalBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(DirectionalBlock::FACING, Direction::SOUTH)
                .set_value(ObserverBlock::POWERED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[DirectionalBlock::FACING, ObserverBlock::POWERED]);
    }
}

impl ObserverBlock {
    /// `updateShape`: its parent's. The signal it starts when the block it faces changes is the
    /// server's: `startSignal` schedules nothing on the client.
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

impl ObserverBlock {
    /// `isSignalSource`: it is one.
    fn is_signal_source(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}

impl ObserverBlock {
    /// `shouldRedstoneWireConnectTo`: wire behind its face joins it.
    fn should_redstone_wire_connect_to(
        _block: &BlockData,
        state: BlockState,
        _level: &dyn BlockGetter,
        _pos: &BlockPos,
        direction: Option<Direction>,
    ) -> bool {
        direction == Some(state.get_value(DirectionalBlock::FACING))
    }
}
