//! net/minecraft/world/level/block/CrafterBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::core::front_and_top::FrontAndTop;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct CrafterBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.CrafterBlock",
    &base_entity_block::CLASS,
);

impl CrafterBlock {
    pub const CRAFTING: &'static BooleanProperty = BlockStateProperties::CRAFTING;
    pub const TRIGGERED: &'static BooleanProperty = BlockStateProperties::TRIGGERED;
    pub const ORIENTATION: &'static EnumProperty<FrontAndTop> = BlockStateProperties::ORIENTATION;

    pub fn new(properties: Properties) -> BlockData {
        CrafterBlock::new_with(
            properties,
            &CLASS,
            CrafterBlock::create_block_state_definition,
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
                .set_value(CrafterBlock::ORIENTATION, FrontAndTop::NORTH_UP)
                .set_value(CrafterBlock::TRIGGERED, false)
                .set_value(CrafterBlock::CRAFTING, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            CrafterBlock::ORIENTATION,
            CrafterBlock::TRIGGERED,
            CrafterBlock::CRAFTING,
        ]);
    }
}
