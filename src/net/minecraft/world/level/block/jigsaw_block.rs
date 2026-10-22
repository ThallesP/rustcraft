//! net/minecraft/world/level/block/JigsawBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::core::front_and_top::FrontAndTop;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct JigsawBlock;

pub static CLASS: BlockClass = BlockClass {
    entity_block: true,
    ..BlockClass::new("net.minecraft.world.level.block.JigsawBlock", &block::CLASS)
};

impl JigsawBlock {
    pub const ORIENTATION: &'static EnumProperty<FrontAndTop> = BlockStateProperties::ORIENTATION;

    pub fn new(properties: Properties) -> BlockData {
        JigsawBlock::new_with(
            properties,
            &CLASS,
            JigsawBlock::create_block_state_definition,
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
                .set_value(JigsawBlock::ORIENTATION, FrontAndTop::NORTH_UP),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[JigsawBlock::ORIENTATION]);
    }
}
