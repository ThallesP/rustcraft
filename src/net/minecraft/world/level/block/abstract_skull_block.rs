//! net/minecraft/world/level/block/AbstractSkullBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour. Its constructor's `type` is kept, which its shapes read (`getType`).

use crate::net::minecraft::world::level::block::base_entity_block::{self, BaseEntityBlock};
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block::entity::block_entity_ticker::BlockEntityTicker;
use crate::net::minecraft::world::level::block::entity::block_entity_type::BlockEntityType;
use crate::net::minecraft::world::level::block::entity::block_entity_types::BlockEntityTypes;
use crate::net::minecraft::world::level::block::entity::skull_block_entity::SkullBlockEntity;
use crate::net::minecraft::world::level::block::skull_block::Types;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct AbstractSkullBlock {
    skull_type: Types,
}

pub static CLASS: BlockClass = BlockClass {
    get_ticker: Some(AbstractSkullBlock::get_ticker),
    ..BlockClass::new(
        "net.minecraft.world.level.block.AbstractSkullBlock",
        &base_entity_block::CLASS,
    )
};

impl AbstractSkullBlock {
    pub const POWERED: &'static BooleanProperty = BlockStateProperties::POWERED;

    pub(crate) fn new_with(
        skull_type: Types,
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = BaseEntityBlock::new_with(properties, class, definition);
        block.set_fields(AbstractSkullBlock { skull_type });
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(AbstractSkullBlock::POWERED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[AbstractSkullBlock::POWERED]);
    }

    /// `getType()`: what kind of head it is.
    pub fn get_type(block: &BlockData) -> Types {
        block.fields::<AbstractSkullBlock>().skull_type
    }
}

impl AbstractSkullBlock {
    /// `getTicker`: on the client, a dragon's or a piglin's head's `animation`.
    fn get_ticker(
        _block: &BlockData,
        state: BlockState,
        r#type: BlockEntityType,
    ) -> Option<BlockEntityTicker> {
        let is_animated = state.is(Blocks::DRAGON_HEAD)
            || state.is(Blocks::DRAGON_WALL_HEAD)
            || state.is(Blocks::PIGLIN_HEAD)
            || state.is(Blocks::PIGLIN_WALL_HEAD);
        if is_animated {
            base_entity_block::create_ticker_helper(
                r#type,
                BlockEntityTypes::SKULL,
                SkullBlockEntity::animation,
            )
        } else {
            None
        }
    }
}
