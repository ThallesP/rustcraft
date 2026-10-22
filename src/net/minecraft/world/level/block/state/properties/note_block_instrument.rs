//! net/minecraft/world/level/block/state/properties/NoteBlockInstrument.java
//!
//! Not ported yet: `getSoundEvent`, `isTunable`, `hasCustomSound`, `worksAboveNoteBlock`.

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// The instrument a note block plays.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NoteBlockInstrument {
    HARP,
    BASEDRUM,
    SNARE,
    HAT,
    BASS,
    FLUTE,
    BELL,
    GUITAR,
    CHIME,
    XYLOPHONE,
    IRON_XYLOPHONE,
    COW_BELL,
    DIDGERIDOO,
    BIT,
    BANJO,
    PLING,
    TRUMPET,
    TRUMPET_EXPOSED,
    TRUMPET_OXIDIZED,
    TRUMPET_WEATHERED,
    ZOMBIE,
    SKELETON,
    CREEPER,
    DRAGON,
    WITHER_SKELETON,
    PIGLIN,
    CUSTOM_HEAD,
}

impl NoteBlockInstrument {
    pub const VALUES: [NoteBlockInstrument; 27] = [
        NoteBlockInstrument::HARP,
        NoteBlockInstrument::BASEDRUM,
        NoteBlockInstrument::SNARE,
        NoteBlockInstrument::HAT,
        NoteBlockInstrument::BASS,
        NoteBlockInstrument::FLUTE,
        NoteBlockInstrument::BELL,
        NoteBlockInstrument::GUITAR,
        NoteBlockInstrument::CHIME,
        NoteBlockInstrument::XYLOPHONE,
        NoteBlockInstrument::IRON_XYLOPHONE,
        NoteBlockInstrument::COW_BELL,
        NoteBlockInstrument::DIDGERIDOO,
        NoteBlockInstrument::BIT,
        NoteBlockInstrument::BANJO,
        NoteBlockInstrument::PLING,
        NoteBlockInstrument::TRUMPET,
        NoteBlockInstrument::TRUMPET_EXPOSED,
        NoteBlockInstrument::TRUMPET_OXIDIZED,
        NoteBlockInstrument::TRUMPET_WEATHERED,
        NoteBlockInstrument::ZOMBIE,
        NoteBlockInstrument::SKELETON,
        NoteBlockInstrument::CREEPER,
        NoteBlockInstrument::DRAGON,
        NoteBlockInstrument::WITHER_SKELETON,
        NoteBlockInstrument::PIGLIN,
        NoteBlockInstrument::CUSTOM_HEAD,
    ];

    pub fn values() -> [NoteBlockInstrument; 27] {
        NoteBlockInstrument::VALUES
    }
}

impl Enum for NoteBlockInstrument {
    const ENUM_CONSTANTS: &'static [NoteBlockInstrument] = &NoteBlockInstrument::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for NoteBlockInstrument {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            NoteBlockInstrument::HARP => "harp",
            NoteBlockInstrument::BASEDRUM => "basedrum",
            NoteBlockInstrument::SNARE => "snare",
            NoteBlockInstrument::HAT => "hat",
            NoteBlockInstrument::BASS => "bass",
            NoteBlockInstrument::FLUTE => "flute",
            NoteBlockInstrument::BELL => "bell",
            NoteBlockInstrument::GUITAR => "guitar",
            NoteBlockInstrument::CHIME => "chime",
            NoteBlockInstrument::XYLOPHONE => "xylophone",
            NoteBlockInstrument::IRON_XYLOPHONE => "iron_xylophone",
            NoteBlockInstrument::COW_BELL => "cow_bell",
            NoteBlockInstrument::DIDGERIDOO => "didgeridoo",
            NoteBlockInstrument::BIT => "bit",
            NoteBlockInstrument::BANJO => "banjo",
            NoteBlockInstrument::PLING => "pling",
            NoteBlockInstrument::TRUMPET => "trumpet",
            NoteBlockInstrument::TRUMPET_EXPOSED => "trumpet_exposed",
            NoteBlockInstrument::TRUMPET_OXIDIZED => "trumpet_oxidized",
            NoteBlockInstrument::TRUMPET_WEATHERED => "trumpet_weathered",
            NoteBlockInstrument::ZOMBIE => "zombie",
            NoteBlockInstrument::SKELETON => "skeleton",
            NoteBlockInstrument::CREEPER => "creeper",
            NoteBlockInstrument::DRAGON => "dragon",
            NoteBlockInstrument::WITHER_SKELETON => "wither_skeleton",
            NoteBlockInstrument::PIGLIN => "piglin",
            NoteBlockInstrument::CUSTOM_HEAD => "custom_head",
        }
    }
}

/// The constant's name, like any enum's `toString`.
impl fmt::Display for NoteBlockInstrument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl PropertyEnum for NoteBlockInstrument {
    const CLASS: &'static str =
        "net.minecraft.world.level.block.state.properties.NoteBlockInstrument";
}
