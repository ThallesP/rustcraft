//! net/minecraft/world/level/block/state/properties/WallSide.java

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// How a wall connects on a side.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WallSide {
    NONE,
    LOW,
    TALL,
}

impl WallSide {
    pub const VALUES: [WallSide; 3] = [WallSide::NONE, WallSide::LOW, WallSide::TALL];

    pub fn values() -> [WallSide; 3] {
        WallSide::VALUES
    }
}

impl Enum for WallSide {
    const ENUM_CONSTANTS: &'static [WallSide] = &WallSide::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for WallSide {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            WallSide::NONE => "none",
            WallSide::LOW => "low",
            WallSide::TALL => "tall",
        }
    }
}

/// Java's `toString`, the serialized name.
impl fmt::Display for WallSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.get_serialized_name())
    }
}

impl PropertyEnum for WallSide {
    const CLASS: &'static str = "net.minecraft.world.level.block.state.properties.WallSide";
}
