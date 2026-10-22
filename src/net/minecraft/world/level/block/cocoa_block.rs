//! net/minecraft/world/level/block/CocoaBlock.java
//!
//! Only what decides its blocks' states so far: its properties, its default state and which of its
//! states tick. Not ported yet: its behaviour.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::horizontal_directional_block::{
    self, HorizontalDirectionalBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct CocoaBlock;

pub static CLASS: BlockClass = BlockClass {
    is_randomly_ticking: Some(CocoaBlock::is_randomly_ticking),
    ..BlockClass::new(
        "net.minecraft.world.level.block.CocoaBlock",
        &horizontal_directional_block::CLASS,
    )
};

impl CocoaBlock {
    pub const AGE: &'static IntegerProperty = BlockStateProperties::AGE_2;

    pub fn new(properties: Properties) -> BlockData {
        CocoaBlock::new_with(
            properties,
            &CLASS,
            CocoaBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = HorizontalDirectionalBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(HorizontalDirectionalBlock::FACING, Direction::NORTH)
                .set_value(CocoaBlock::AGE, 0),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[HorizontalDirectionalBlock::FACING, CocoaBlock::AGE]);
    }

    fn is_randomly_ticking(_block: &BlockData, state: BlockState) -> bool {
        state.get_value(CocoaBlock::AGE) < 2
    }
}
