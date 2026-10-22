//! net/minecraft/world/level/block/LayeredCauldronBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `precipitationType` and `interactionMap`.

use crate::net::minecraft::world::level::block::abstract_cauldron_block::{
    self, AbstractCauldronBlock,
};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct LayeredCauldronBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.LayeredCauldronBlock",
    &abstract_cauldron_block::CLASS,
);

impl LayeredCauldronBlock {
    pub const LEVEL: &'static IntegerProperty = BlockStateProperties::LEVEL_CAULDRON;

    pub fn new(properties: Properties) -> BlockData {
        LayeredCauldronBlock::new_with(
            properties,
            &CLASS,
            LayeredCauldronBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = AbstractCauldronBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(LayeredCauldronBlock::LEVEL, 1),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[LayeredCauldronBlock::LEVEL]);
    }
}
