//! net/minecraft/world/level/block/MossyCarpetBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.
//!
//! Of its behaviour, `propagatesSkylightDown` is ported, for the light engine.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::wall_side::WallSide;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct MossyCarpetBlock;

pub static CLASS: BlockClass = BlockClass {
    propagates_skylight_down: Some(MossyCarpetBlock::propagates_skylight_down),
    ..BlockClass::new(
        "net.minecraft.world.level.block.MossyCarpetBlock",
        &block::CLASS,
    )
};

impl MossyCarpetBlock {
    pub const BASE: &'static BooleanProperty = BlockStateProperties::BOTTOM;
    pub const NORTH: &'static EnumProperty<WallSide> = BlockStateProperties::NORTH_WALL;
    pub const EAST: &'static EnumProperty<WallSide> = BlockStateProperties::EAST_WALL;
    pub const SOUTH: &'static EnumProperty<WallSide> = BlockStateProperties::SOUTH_WALL;
    pub const WEST: &'static EnumProperty<WallSide> = BlockStateProperties::WEST_WALL;

    pub fn new(properties: Properties) -> BlockData {
        MossyCarpetBlock::new_with(
            properties,
            &CLASS,
            MossyCarpetBlock::create_block_state_definition,
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
                .set_value(MossyCarpetBlock::BASE, true)
                .set_value(MossyCarpetBlock::NORTH, WallSide::NONE)
                .set_value(MossyCarpetBlock::EAST, WallSide::NONE)
                .set_value(MossyCarpetBlock::SOUTH, WallSide::NONE)
                .set_value(MossyCarpetBlock::WEST, WallSide::NONE),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            MossyCarpetBlock::BASE,
            MossyCarpetBlock::NORTH,
            MossyCarpetBlock::EAST,
            MossyCarpetBlock::SOUTH,
            MossyCarpetBlock::WEST,
        ]);
    }
}

impl MossyCarpetBlock {
    /// `propagatesSkylightDown`.
    fn propagates_skylight_down(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}
