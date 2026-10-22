//! net/minecraft/world/level/block/state/properties/RedstoneSide.java
//!
//! Not ported yet: `isConnected`.

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// How redstone dust connects on a side.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RedstoneSide {
    UP,
    SIDE,
    NONE,
}

impl RedstoneSide {
    pub const VALUES: [RedstoneSide; 3] =
        [RedstoneSide::UP, RedstoneSide::SIDE, RedstoneSide::NONE];

    pub fn values() -> [RedstoneSide; 3] {
        RedstoneSide::VALUES
    }
}

impl Enum for RedstoneSide {
    const ENUM_CONSTANTS: &'static [RedstoneSide] = &RedstoneSide::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for RedstoneSide {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            RedstoneSide::UP => "up",
            RedstoneSide::SIDE => "side",
            RedstoneSide::NONE => "none",
        }
    }
}

/// Java's `toString`, the serialized name.
impl fmt::Display for RedstoneSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.get_serialized_name())
    }
}

impl PropertyEnum for RedstoneSide {
    const CLASS: &'static str = "net.minecraft.world.level.block.state.properties.RedstoneSide";
}
