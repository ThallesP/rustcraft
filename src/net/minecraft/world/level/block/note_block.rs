//! net/minecraft/world/level/block/NoteBlock.java
//!
//! Only what decides its blocks' states so far: its properties and its default state. Not ported
//! yet: its behaviour.

use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::properties::note_block_instrument::NoteBlockInstrument;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct NoteBlock;

pub static CLASS: BlockClass =
    BlockClass::new("net.minecraft.world.level.block.NoteBlock", &block::CLASS);

impl NoteBlock {
    pub const INSTRUMENT: &'static EnumProperty<NoteBlockInstrument> =
        BlockStateProperties::NOTEBLOCK_INSTRUMENT;
    pub const POWERED: &'static BooleanProperty = BlockStateProperties::POWERED;
    pub const NOTE: &'static IntegerProperty = BlockStateProperties::NOTE;

    pub fn new(properties: Properties) -> BlockData {
        NoteBlock::new_with(properties, &CLASS, NoteBlock::create_block_state_definition)
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
                .set_value(NoteBlock::INSTRUMENT, NoteBlockInstrument::HARP)
                .set_value(NoteBlock::NOTE, 0)
                .set_value(NoteBlock::POWERED, false),
        );
        block
    }

    pub fn create_block_state_definition(builder: &mut Builder) {
        builder.add(&[NoteBlock::INSTRUMENT, NoteBlock::POWERED, NoteBlock::NOTE]);
    }
}
