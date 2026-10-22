//! net/minecraft/world/level/block/SlabBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::slab_type::SlabType;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct SlabBlock;

pub static CLASS: BlockClass =
    BlockClass::new("net.minecraft.world.level.block.SlabBlock", &block::CLASS);

impl SlabBlock {
    pub const TYPE: &'static EnumProperty<SlabType> = BlockStateProperties::SLAB_TYPE;
    pub const WATERLOGGED: &'static BooleanProperty = BlockStateProperties::WATERLOGGED;

    pub fn new(properties: Properties) -> BlockData {
        SlabBlock::new_with(properties, &CLASS, SlabBlock::create_block_state_definition)
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = Block::new_with(properties, class, definition);
        block.register_default_state(
            block
                .default_block_state()
                .set_value(SlabBlock::TYPE, SlabType::BOTTOM)
                .set_value(SlabBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[SlabBlock::TYPE, SlabBlock::WATERLOGGED]);
    }
}
