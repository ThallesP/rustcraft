//! net/minecraft/world/level/block/BambooStalkBlock.java
//!
//! Only what decides its blocks' states so far: its properties, its default state and which of its
//! states tick. Not ported yet: its behaviour.
//!
//! Of its behaviour, `propagatesSkylightDown` is ported, for the light engine.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::bamboo_leaves::BambooLeaves;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct BambooStalkBlock;

pub static CLASS: BlockClass = BlockClass {
    propagates_skylight_down: Some(BambooStalkBlock::propagates_skylight_down),
    is_randomly_ticking: Some(BambooStalkBlock::is_randomly_ticking),
    ..BlockClass::new(
        "net.minecraft.world.level.block.BambooStalkBlock",
        &block::CLASS,
    )
};

impl BambooStalkBlock {
    pub const AGE: &'static IntegerProperty = BlockStateProperties::AGE_1;
    pub const LEAVES: &'static EnumProperty<BambooLeaves> = BlockStateProperties::BAMBOO_LEAVES;
    pub const STAGE: &'static IntegerProperty = BlockStateProperties::STAGE;

    pub fn new(properties: Properties) -> BlockData {
        BambooStalkBlock::new_with(
            properties,
            &CLASS,
            BambooStalkBlock::create_block_state_definition,
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
                .set_value(BambooStalkBlock::AGE, 0)
                .set_value(BambooStalkBlock::LEAVES, BambooLeaves::NONE)
                .set_value(BambooStalkBlock::STAGE, 0),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            BambooStalkBlock::AGE,
            BambooStalkBlock::LEAVES,
            BambooStalkBlock::STAGE,
        ]);
    }

    fn is_randomly_ticking(_block: &BlockData, state: BlockState) -> bool {
        state.get_value(BambooStalkBlock::STAGE) == 0
    }
}

impl BambooStalkBlock {
    /// `propagatesSkylightDown`.
    fn propagates_skylight_down(_block: &BlockData, _state: BlockState) -> bool {
        true
    }
}
