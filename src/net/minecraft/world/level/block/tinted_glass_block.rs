//! net/minecraft/world/level/block/TintedGlassBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.
//!
//! Of its behaviour, `propagatesSkylightDown` and `getLightDampening` are ported, for the light
//! engine.

use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::transparent_block::{self, TransparentBlock};

pub struct TintedGlassBlock;

pub static CLASS: BlockClass = BlockClass {
    propagates_skylight_down: Some(TintedGlassBlock::propagates_skylight_down),
    get_light_dampening: Some(TintedGlassBlock::get_light_dampening),
    ..BlockClass::new(
        "net.minecraft.world.level.block.TintedGlassBlock",
        &transparent_block::CLASS,
    )
};

impl TintedGlassBlock {
    pub fn new(properties: Properties) -> BlockData {
        TintedGlassBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        TransparentBlock::new_with(properties, class, definition)
    }
}

impl TintedGlassBlock {
    /// `propagatesSkylightDown`.
    fn propagates_skylight_down(_block: &BlockData, _state: BlockState) -> bool {
        false
    }

    /// `getLightDampening`.
    fn get_light_dampening(_block: &BlockData, _state: BlockState) -> i32 {
        15
    }
}
