//! net/minecraft/world/level/block/state/properties/Half.java

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// The top or bottom half of a block, like a stair's or a trapdoor's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Half {
    TOP,
    BOTTOM,
}

impl Half {
    pub const VALUES: [Half; 2] = [Half::TOP, Half::BOTTOM];

    pub fn values() -> [Half; 2] {
        Half::VALUES
    }
}

impl Enum for Half {
    const ENUM_CONSTANTS: &'static [Half] = &Half::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for Half {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            Half::TOP => "top",
            Half::BOTTOM => "bottom",
        }
    }
}

/// Java's `toString`, the serialized name.
impl fmt::Display for Half {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.get_serialized_name())
    }
}

impl PropertyEnum for Half {
    const CLASS: &'static str = "net.minecraft.world.level.block.state.properties.Half";
}
