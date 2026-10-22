//! net/minecraft/world/level/block/CeilingHangingSignBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour, and its constructor's `type`.

use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::sign_block::{self, SignBlock};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct CeilingHangingSignBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.CeilingHangingSignBlock",
    &sign_block::CLASS,
);

impl CeilingHangingSignBlock {
    pub const ROTATION: &'static IntegerProperty = BlockStateProperties::ROTATION_16;
    pub const ATTACHED: &'static BooleanProperty = BlockStateProperties::ATTACHED;

    pub fn new(properties: Properties) -> BlockData {
        CeilingHangingSignBlock::new_with(
            properties,
            &CLASS,
            CeilingHangingSignBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = SignBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(CeilingHangingSignBlock::ROTATION, 8)
                .set_value(CeilingHangingSignBlock::ATTACHED, false)
                .set_value(SignBlock::WATERLOGGED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            CeilingHangingSignBlock::ROTATION,
            CeilingHangingSignBlock::ATTACHED,
            SignBlock::WATERLOGGED,
        ]);
    }
}
