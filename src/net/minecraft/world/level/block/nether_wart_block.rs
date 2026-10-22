//! net/minecraft/world/level/block/NetherWartBlock.java
//!
//! Only what decides its blocks' states so far: its properties, its default state and which of its
//! states tick. Not ported yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block::vegetation_block::{self, VegetationBlock};

pub struct NetherWartBlock;

pub static CLASS: BlockClass = BlockClass {
    is_randomly_ticking: Some(NetherWartBlock::is_randomly_ticking),
    ..BlockClass::new(
        "net.minecraft.world.level.block.NetherWartBlock",
        &vegetation_block::CLASS,
    )
};

impl NetherWartBlock {
    pub const AGE: &'static IntegerProperty = BlockStateProperties::AGE_3;

    pub fn new(properties: Properties) -> BlockData {
        NetherWartBlock::new_with(
            properties,
            &CLASS,
            NetherWartBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = VegetationBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(NetherWartBlock::AGE, 0),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[NetherWartBlock::AGE]);
    }

    fn is_randomly_ticking(_block: &BlockData, state: BlockState) -> bool {
        state.get_value(NetherWartBlock::AGE) < 3
    }
}
