//! net/minecraft/world/level/block/state/properties/CreakingHeartState.java

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// Whether a creaking heart is uprooted, dormant or awake.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CreakingHeartState {
    UPROOTED,
    DORMANT,
    AWAKE,
}

impl CreakingHeartState {
    pub const VALUES: [CreakingHeartState; 3] = [
        CreakingHeartState::UPROOTED,
        CreakingHeartState::DORMANT,
        CreakingHeartState::AWAKE,
    ];

    pub fn values() -> [CreakingHeartState; 3] {
        CreakingHeartState::VALUES
    }
}

impl Enum for CreakingHeartState {
    const ENUM_CONSTANTS: &'static [CreakingHeartState] = &CreakingHeartState::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for CreakingHeartState {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            CreakingHeartState::UPROOTED => "uprooted",
            CreakingHeartState::DORMANT => "dormant",
            CreakingHeartState::AWAKE => "awake",
        }
    }
}

/// Java's `toString`, the serialized name.
impl fmt::Display for CreakingHeartState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.get_serialized_name())
    }
}

impl PropertyEnum for CreakingHeartState {
    const CLASS: &'static str =
        "net.minecraft.world.level.block.state.properties.CreakingHeartState";
}
