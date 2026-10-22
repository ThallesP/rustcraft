//! net/minecraft/world/level/block/BrewingStandBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct BrewingStandBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.BrewingStandBlock",
    &base_entity_block::CLASS,
);

impl BrewingStandBlock {
    pub const HAS_BOTTLE: [&'static BooleanProperty; 3] = [
        BlockStateProperties::HAS_BOTTLE_0,
        BlockStateProperties::HAS_BOTTLE_1,
        BlockStateProperties::HAS_BOTTLE_2,
    ];

    pub fn new(properties: Properties) -> BlockData {
        BrewingStandBlock::new_with(
            properties,
            &CLASS,
            BrewingStandBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = BaseEntityBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(BrewingStandBlock::HAS_BOTTLE[0], false)
                .set_value(BrewingStandBlock::HAS_BOTTLE[1], false)
                .set_value(BrewingStandBlock::HAS_BOTTLE[2], false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            BrewingStandBlock::HAS_BOTTLE[0],
            BrewingStandBlock::HAS_BOTTLE[1],
            BrewingStandBlock::HAS_BOTTLE[2],
        ]);
    }
}
