//! net/minecraft/world/level/block/EnchantingTableBlock.java
//!
//! Only the class so far: its blocks' states are its parent's. Not ported yet: its behaviour.
//!
//! Of its behaviour, `useShapeForLightOcclusion` is ported, for the light engine.

use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

pub struct EnchantingTableBlock;

pub static CLASS: BlockClass = BlockClass {
    use_shape_for_light_occlusion: Some(EnchantingTableBlock::use_shape_for_light_occlusion),
    ..BlockClass::new(
        "net.minecraft.world.level.block.EnchantingTableBlock",
        &base_entity_block::CLASS,
    )
};

impl EnchantingTableBlock {
    pub fn new(properties: Properties) -> BlockData {
        EnchantingTableBlock::new_with(properties, &CLASS, Block::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        BaseEntityBlock::new_with(properties, class, definition)
    }
}

impl EnchantingTableBlock {
    /// `useShapeForLightOcclusion`.
    fn use_shape_for_light_occlusion(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}
