//! net/minecraft/world/level/block/state/properties/PotentSulfurState.java

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// How wet potent sulfur is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PotentSulfurState {
    DRY,
    WET,
    DORMANT,
    ERUPTING,
    CONTINUOUS,
}

impl PotentSulfurState {
    pub const VALUES: [PotentSulfurState; 5] = [
        PotentSulfurState::DRY,
        PotentSulfurState::WET,
        PotentSulfurState::DORMANT,
        PotentSulfurState::ERUPTING,
        PotentSulfurState::CONTINUOUS,
    ];

    pub fn values() -> [PotentSulfurState; 5] {
        PotentSulfurState::VALUES
    }
}

impl Enum for PotentSulfurState {
    const ENUM_CONSTANTS: &'static [PotentSulfurState] = &PotentSulfurState::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for PotentSulfurState {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            PotentSulfurState::DRY => "dry",
            PotentSulfurState::WET => "wet",
            PotentSulfurState::DORMANT => "dormant",
            PotentSulfurState::ERUPTING => "erupting",
            PotentSulfurState::CONTINUOUS => "continuous",
        }
    }
}

/// Java's `toString`, the serialized name.
impl fmt::Display for PotentSulfurState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.get_serialized_name())
    }
}

impl PropertyEnum for PotentSulfurState {
    const CLASS: &'static str =
        "net.minecraft.world.level.block.state.properties.PotentSulfurState";
}
