//! net/minecraft/world/level/block/state/properties/Tilt.java
//!
//! Not ported yet: `causesVibration`.

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// How far a big dripleaf has tipped.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tilt {
    NONE,
    UNSTABLE,
    PARTIAL,
    FULL,
}

impl Tilt {
    pub const VALUES: [Tilt; 4] = [Tilt::NONE, Tilt::UNSTABLE, Tilt::PARTIAL, Tilt::FULL];

    pub fn values() -> [Tilt; 4] {
        Tilt::VALUES
    }
}

impl Enum for Tilt {
    const ENUM_CONSTANTS: &'static [Tilt] = &Tilt::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for Tilt {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            Tilt::NONE => "none",
            Tilt::UNSTABLE => "unstable",
            Tilt::PARTIAL => "partial",
            Tilt::FULL => "full",
        }
    }
}

/// The constant's name, like any enum's `toString`.
impl fmt::Display for Tilt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl PropertyEnum for Tilt {
    const CLASS: &'static str = "net.minecraft.world.level.block.state.properties.Tilt";
}
