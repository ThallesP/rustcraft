//! net/minecraft/world/level/block/ComparatorBlock.java
//!
//! What decides its blocks' states: its properties and its default state, and its block event,
//! which its block entity handles (`triggerEvent`). Not ported yet: the rest of its behaviour.

use crate::net::minecraft::client::multiplayer::client_level::LevelTicking;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::world::level::block::base_entity_block;
use crate::net::minecraft::world::level::block::block::{
    BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::diode_block::{self, DiodeBlock};
use crate::net::minecraft::world::level::block::horizontal_directional_block::HorizontalDirectionalBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::comparator_mode::ComparatorMode;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct ComparatorBlock;

pub static CLASS: BlockClass = BlockClass {
    entity_block: true,
    trigger_event: Some(ComparatorBlock::trigger_event),
    ..BlockClass::new(
        "net.minecraft.world.level.block.ComparatorBlock",
        &diode_block::CLASS,
    )
};

impl ComparatorBlock {
    pub const MODE: &'static EnumProperty<ComparatorMode> = BlockStateProperties::MODE_COMPARATOR;

    pub fn new(properties: Properties) -> BlockData {
        ComparatorBlock::new_with(
            properties,
            &CLASS,
            ComparatorBlock::create_block_state_definition,
        )
    }

    pub(crate) fn new_with(
        properties: Properties,
        class: &'static BlockClass,
        definition: CreateBlockStateDefinition,
    ) -> BlockData {
        let mut block = DiodeBlock::new_with(properties, class, definition);
        block.register_default_state(
            block
                .get_state_definition()
                .any()
                .set_value(HorizontalDirectionalBlock::FACING, Direction::NORTH)
                .set_value(DiodeBlock::POWERED, false)
                .set_value(ComparatorBlock::MODE, ComparatorMode::COMPARE),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[
            HorizontalDirectionalBlock::FACING,
            ComparatorBlock::MODE,
            DiodeBlock::POWERED,
        ]);
    }
}

impl ComparatorBlock {
    /// `triggerEvent`: its block entity's (made there if it has none), which handles nothing.
    fn trigger_event(
        _block: &BlockData,
        _state: BlockState,
        level: &mut LevelTicking<'_>,
        pos: &BlockPos,
        b0: i32,
        b1: i32,
    ) -> bool {
        base_entity_block::block_entity_trigger_event(level, pos, b0, b1)
    }
}
