//! net/minecraft/world/level/block/ChorusFlowerBlock.java
//!
//! Only what decides its blocks' states so far: its properties, its default state and which of its
//! states tick. Not ported yet: its behaviour, and its constructor's `plant`.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct ChorusFlowerBlock;

pub static CLASS: BlockClass = BlockClass {
    is_randomly_ticking: Some(ChorusFlowerBlock::is_randomly_ticking),
    ..BlockClass::new(
        "net.minecraft.world.level.block.ChorusFlowerBlock",
        &block::CLASS,
    )
};

impl ChorusFlowerBlock {
    pub const AGE: &'static IntegerProperty = BlockStateProperties::AGE_5;

    pub fn new(properties: Properties) -> BlockData {
        ChorusFlowerBlock::new_with(
            properties,
            &CLASS,
            ChorusFlowerBlock::create_block_state_definition,
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
                .set_value(ChorusFlowerBlock::AGE, 0),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[ChorusFlowerBlock::AGE]);
    }

    fn is_randomly_ticking(_block: &BlockData, state: BlockState) -> bool {
        state.get_value(ChorusFlowerBlock::AGE) < 5
    }
}
