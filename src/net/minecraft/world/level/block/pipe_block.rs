//! net/minecraft/world/level/block/PipeBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour, and
//! its constructor's `size`.
//!
//! Of its behaviour, `propagatesSkylightDown` is ported, for the light engine.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;

pub struct PipeBlock;

pub static CLASS: BlockClass = BlockClass {
    propagates_skylight_down: Some(PipeBlock::propagates_skylight_down),
    ..BlockClass::new("net.minecraft.world.level.block.PipeBlock", &block::CLASS)
};

impl PipeBlock {
    /// `PROPERTY_BY_DIRECTION`, an `EnumMap`: each direction's property, by the direction's
    /// ordinal.
    pub const PROPERTY_BY_DIRECTION: [&'static BooleanProperty; 6] = [
        PipeBlock::DOWN,
        PipeBlock::UP,
        PipeBlock::NORTH,
        PipeBlock::SOUTH,
        PipeBlock::WEST,
        PipeBlock::EAST,
    ];

    pub const NORTH: &'static BooleanProperty = BlockStateProperties::NORTH;
    pub const EAST: &'static BooleanProperty = BlockStateProperties::EAST;
    pub const SOUTH: &'static BooleanProperty = BlockStateProperties::SOUTH;
    pub const WEST: &'static BooleanProperty = BlockStateProperties::WEST;
    pub const UP: &'static BooleanProperty = BlockStateProperties::UP;
    pub const DOWN: &'static BooleanProperty = BlockStateProperties::DOWN;

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        Block::new_with(properties, class, definition)
    }
}

impl PipeBlock {
    /// `propagatesSkylightDown`.
    fn propagates_skylight_down(_block: &BlockData, _state: BlockState) -> bool {
        false
    }
}
