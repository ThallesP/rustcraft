//! net/minecraft/world/level/block/HangingMossBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct HangingMossBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.HangingMossBlock",
    &block::CLASS,
);

impl HangingMossBlock {
    pub const TIP: &'static BooleanProperty = BlockStateProperties::TIP;

    pub fn new(properties: Properties) -> BlockData {
        HangingMossBlock::new_with(
            properties,
            &CLASS,
            HangingMossBlock::create_block_state_definition,
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
                .set_value(HangingMossBlock::TIP, true),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[HangingMossBlock::TIP]);
    }
}
