//! net/minecraft/world/level/block/IronBarsBlock.java
//!
//! What decides its blocks' states (its properties and its default state), and the sizes its
//! parent's shapes are made with (2, 16, 2, 16 and 16 sixteenths). Not ported yet: the rest of its
//! behaviour.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::cross_collision_block::{
    self, CrossCollisionBlock,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct IronBarsBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.IronBarsBlock",
    &cross_collision_block::CLASS,
);

impl IronBarsBlock {
    pub fn new(properties: Properties) -> BlockData {
        IronBarsBlock::new_with(
            properties,
            &CLASS,
            IronBarsBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = CrossCollisionBlock::new_with(
            2.0, 16.0, 2.0, 16.0, 16.0, properties, class, definition,
        );
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(CrossCollisionBlock::NORTH, false)
                .set_value(CrossCollisionBlock::EAST, false)
                .set_value(CrossCollisionBlock::SOUTH, false)
                .set_value(CrossCollisionBlock::WEST, false)
                .set_value(CrossCollisionBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            CrossCollisionBlock::NORTH,
            CrossCollisionBlock::EAST,
            CrossCollisionBlock::WEST,
            CrossCollisionBlock::SOUTH,
            CrossCollisionBlock::WATERLOGGED,
        ]);
    }
}
