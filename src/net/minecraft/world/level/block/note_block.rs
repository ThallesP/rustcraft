//! net/minecraft/world/level/block/NoteBlock.java
//!
//! Its properties and its default state, and its block event (`triggerEvent`), which the server
//! sends when it plays: the client's note particle, and the sound's seed from the level's random
//! (the sound itself is the server's to send: a client's `playSeededSound` plays only for its own
//! player). Not ported yet: the rest of its behaviour.

use crate::net::minecraft::client::multiplayer::client_level::LevelTicking;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::particles::particle_types::ParticleTypes;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::block::block::{
    self, Block, BlockClass, BlockData, CreateBlockStateDefinition,
};
use crate::net::minecraft::world::level::block::entity::block_entity::BlockEntityData;
use crate::net::minecraft::world::level::block::state::block_behaviour::Properties;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::boolean_property::BooleanProperty;
use crate::net::minecraft::world::level::block::state::properties::enum_property::EnumProperty;
use crate::net::minecraft::world::level::block::state::properties::integer_property::IntegerProperty;
use crate::net::minecraft::world::level::block::state::properties::note_block_instrument::NoteBlockInstrument;
use crate::net::minecraft::world::level::block::state::state_definition::Builder;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

pub struct NoteBlock;

pub static CLASS: BlockClass = BlockClass {
    trigger_event: Some(NoteBlock::trigger_event),
    ..BlockClass::new("net.minecraft.world.level.block.NoteBlock", &block::CLASS)
};

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

impl NoteBlock {
    /// `triggerEvent`: a note particle of its pitch's colour for a tunable instrument, and the
    /// sound, seeded from the level's random, unless a custom head names no sound.
    fn trigger_event(
        _block: &BlockData,
        state: BlockState,
        level: &mut LevelTicking<'_>,
        pos: &BlockPos,
        _b0: i32,
        _b1: i32,
    ) -> bool {
        let instrument = state.get_value(NoteBlock::INSTRUMENT);
        if instrument.is_tunable() {
            let note = state.get_value(NoteBlock::NOTE);
            level.effects_level().add_particle(
                ParticleTypes::NOTE,
                f64::from(pos.get_x()) + 0.5,
                f64::from(pos.get_y()) + 1.2,
                f64::from(pos.get_z()) + 0.5,
                f64::from(note) / 24.0,
                0.0,
                0.0,
            );
        }

        if instrument.has_custom_sound() && NoteBlock::get_custom_sound_id(level, pos).is_none() {
            return false;
        }

        // `playSeededSound(null, …)`: no sound for the client's player, but the seed is drawn.
        level.level.get_random().next_long();
        true
    }

    /// `getCustomSoundId`: the note block sound of the head above it, made there if its block
    /// makes one (`getBlockEntity`, `IMMEDIATE`).
    fn get_custom_sound_id(level: &mut LevelTicking<'_>, pos: &BlockPos) -> Option<Identifier> {
        match &level.level.get_block_entity(&pos.above())?.data {
            BlockEntityData::Skull(head) => head.get_note_block_sound().cloned(),
            _ => None,
        }
    }
}
