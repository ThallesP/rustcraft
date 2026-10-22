//! net/minecraft/world/level/block/SkullBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `type`.

use crate::net::minecraft::world::level::block::abstract_skull_block::{self, AbstractSkullBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct SkullBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.SkullBlock",
    &abstract_skull_block::CLASS,
);

impl SkullBlock {
    pub const ROTATION: &'static IntegerProperty = BlockStateProperties::ROTATION_16;

    pub fn new(properties: Properties) -> BlockData {
        SkullBlock::new_with(
            properties,
            &CLASS,
            SkullBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = AbstractSkullBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .default_block_state()
                .set_value(SkullBlock::ROTATION, 0),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        AbstractSkullBlock::create_block_state_definition(builder);
        builder.add(&[SkullBlock::ROTATION]);
    }
}
