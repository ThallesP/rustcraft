//! net/minecraft/world/level/block/VaultBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::entity::vault::vault_state::VaultState;
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct VaultBlock;

pub static CLASS: BlockClass = BlockClass::new(
    "net.minecraft.world.level.block.VaultBlock",
    &base_entity_block::CLASS,
);

impl VaultBlock {
    pub const STATE: &'static EnumProperty<VaultState> = BlockStateProperties::VAULT_STATE;
    pub const FACING: &'static EnumProperty<Direction> = HorizontalDirectionalBlock::FACING;
    pub const OMINOUS: &'static BooleanProperty = BlockStateProperties::OMINOUS;

    pub fn new(properties: Properties) -> BlockData {
        VaultBlock::new_with(
            properties,
            &CLASS,
            VaultBlock::create_block_state_definition,
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
                .get_state_definition()
                .any()
                .set_value(VaultBlock::FACING, Direction::NORTH)
                .set_value(VaultBlock::STATE, VaultState::INACTIVE)
                .set_value(VaultBlock::OMINOUS, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[VaultBlock::FACING, VaultBlock::STATE, VaultBlock::OMINOUS]);
    }
}
