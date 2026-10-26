//! net/minecraft/world/level/block/TargetBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct TargetBlock;

pub static CLASS: BlockClass = BlockClass {
    is_signal_source: Some(TargetBlock::is_signal_source),
    ..BlockClass::new("net.minecraft.world.level.block.TargetBlock", &block::CLASS)
};

impl TargetBlock {
    pub const OUTPUT_POWER: &'static IntegerProperty = BlockStateProperties::POWER;

    pub fn new(properties: Properties) -> BlockData {
        TargetBlock::new_with(
            properties,
            &CLASS,
            TargetBlock::create_block_state_definition,
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
                .set_value(TargetBlock::OUTPUT_POWER, 0),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[TargetBlock::OUTPUT_POWER]);
    }
}

impl TargetBlock {
    /// `isSignalSource`: it is one.
    fn is_signal_source(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}
