//! net/minecraft/world/level/block/CaveVinesBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::cave_vines::CaveVines;
use crate::net::minecraft::world::level::block::growing_plant_head_block::{
    self, GrowingPlantHeadBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct CaveVinesBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.CaveVinesBlock",
    &growing_plant_head_block::CLASS,
);

impl CaveVinesBlock {
    pub fn new(properties: Properties) -> BlockData {
        CaveVinesBlock::new_with(
            properties,
            &CLASS,
            CaveVinesBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = GrowingPlantHeadBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(GrowingPlantHeadBlock::AGE, 0)
                .set_value(CaveVines::BERRIES, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        GrowingPlantHeadBlock::create_block_state_definition(builder);
        builder.add(&[CaveVines::BERRIES]);
    }
}
