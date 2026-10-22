//! net/minecraft/world/level/block/state/properties/SculkSensorPhase.java

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// What a sculk sensor is doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SculkSensorPhase {
    INACTIVE,
    ACTIVE,
    COOLDOWN,
}

impl SculkSensorPhase {
    pub const VALUES: [SculkSensorPhase; 3] = [
        SculkSensorPhase::INACTIVE,
        SculkSensorPhase::ACTIVE,
        SculkSensorPhase::COOLDOWN,
    ];

    pub fn values() -> [SculkSensorPhase; 3] {
        SculkSensorPhase::VALUES
    }
}

impl Enum for SculkSensorPhase {
    const ENUM_CONSTANTS: &'static [SculkSensorPhase] = &SculkSensorPhase::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for SculkSensorPhase {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            SculkSensorPhase::INACTIVE => "inactive",
            SculkSensorPhase::ACTIVE => "active",
            SculkSensorPhase::COOLDOWN => "cooldown",
        }
    }
}

/// Java's `toString`, the serialized name.
impl fmt::Display for SculkSensorPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.get_serialized_name())
    }
}

impl PropertyEnum for SculkSensorPhase {
    const CLASS: &'static str = "net.minecraft.world.level.block.state.properties.SculkSensorPhase";
}
