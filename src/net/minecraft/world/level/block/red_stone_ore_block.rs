//! net/minecraft/world/level/block/RedStoneOreBlock.java
//!
//! Only what decides its blocks' states so far: its properties, its default state and which of its
//! states tick. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::redstone_torch_block::RedstoneTorchBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct RedStoneOreBlock;

pub static CLASS: BlockClass = BlockClass {
    is_randomly_ticking: Some(RedStoneOreBlock::is_randomly_ticking),
    ..BlockClass::new(
        "net.minecraft.world.level.block.RedStoneOreBlock",
        &block::CLASS,
    )
};

impl RedStoneOreBlock {
    pub const LIT: &'static BooleanProperty = RedstoneTorchBlock::LIT;

    pub fn new(properties: Properties) -> BlockData {
        RedStoneOreBlock::new_with(
            properties,
            &CLASS,
            RedStoneOreBlock::create_block_state_definition,
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
                .default_block_state()
                .set_value(RedStoneOreBlock::LIT, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[RedStoneOreBlock::LIT]);
    }

    fn is_randomly_ticking(_block: &BlockData, state: BlockState) -> bool {
        state.get_value(RedStoneOreBlock::LIT)
    }
}
