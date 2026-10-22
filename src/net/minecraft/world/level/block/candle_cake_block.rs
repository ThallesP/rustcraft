//! net/minecraft/world/level/block/CandleCakeBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `block`.

use crate::net::minecraft::world::level::block::abstract_candle_block::{
    self, AbstractCandleBlock,
};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct CandleCakeBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.CandleCakeBlock",
    &abstract_candle_block::CLASS,
);

impl CandleCakeBlock {
    pub const LIT: &'static BooleanProperty = AbstractCandleBlock::LIT;

    pub fn new(properties: Properties) -> BlockData {
        CandleCakeBlock::new_with(
            properties,
            &CLASS,
            CandleCakeBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = AbstractCandleBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(CandleCakeBlock::LIT, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[CandleCakeBlock::LIT]);
    }
}
