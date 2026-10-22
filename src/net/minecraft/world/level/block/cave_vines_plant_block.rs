//! net/minecraft/world/level/block/CaveVinesPlantBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::cave_vines::CaveVines;
use crate::net::minecraft::world::level::block::growing_plant_body_block::{
    self, GrowingPlantBodyBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct CaveVinesPlantBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.CaveVinesPlantBlock",
    &growing_plant_body_block::CLASS,
);

impl CaveVinesPlantBlock {
    pub fn new(properties: Properties) -> BlockData {
        CaveVinesPlantBlock::new_with(
            properties,
            &CLASS,
            CaveVinesPlantBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = GrowingPlantBodyBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(CaveVines::BERRIES, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[CaveVines::BERRIES]);
    }
}
