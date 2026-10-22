//! net/minecraft/world/level/block/PotentSulfurBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::potent_sulfur_state::PotentSulfurState;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct PotentSulfurBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.PotentSulfurBlock",
    &base_entity_block::CLASS,
);

impl PotentSulfurBlock {
    pub const STATE: &'static EnumProperty<PotentSulfurState> =
        BlockStateProperties::POTENT_SULFUR_STATE;

    pub fn new(properties: Properties) -> BlockData {
        PotentSulfurBlock::new_with(
            properties,
            &CLASS,
            PotentSulfurBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = BaseEntityBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .default_block_state()
                .set_value(PotentSulfurBlock::STATE, PotentSulfurState::DRY),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[PotentSulfurBlock::STATE]);
    }
}
