//! net/minecraft/world/level/block/RespawnAnchorBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct RespawnAnchorBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.RespawnAnchorBlock",
    &block::CLASS,
);

impl RespawnAnchorBlock {
    pub const CHARGE: &'static IntegerProperty = BlockStateProperties::RESPAWN_ANCHOR_CHARGES;

    pub fn new(properties: Properties) -> BlockData {
        RespawnAnchorBlock::new_with(
            properties,
            &CLASS,
            RespawnAnchorBlock::create_block_state_definition,
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
                .set_value(RespawnAnchorBlock::CHARGE, 0),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[RespawnAnchorBlock::CHARGE]);
    }

    /// Its charge above the least (0) out of 4, scaled to `maximum` and rounded down.
    pub fn get_scaled_charge_level(state: BlockState, maximum: i32) -> i32 {
        Mth::floor_f32(state.get_value(RespawnAnchorBlock::CHARGE) as f32 / 4.0 * maximum as f32)
    }
}
