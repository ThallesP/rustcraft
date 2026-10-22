//! net/minecraft/world/level/block/state/properties/AttachFace.java

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// What a button, lever or grindstone hangs from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AttachFace {
    FLOOR,
    WALL,
    CEILING,
}

impl AttachFace {
    pub const VALUES: [AttachFace; 3] = [AttachFace::FLOOR, AttachFace::WALL, AttachFace::CEILING];

    pub fn values() -> [AttachFace; 3] {
        AttachFace::VALUES
    }
}

impl Enum for AttachFace {
    const ENUM_CONSTANTS: &'static [AttachFace] = &AttachFace::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for AttachFace {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            AttachFace::FLOOR => "floor",
            AttachFace::WALL => "wall",
            AttachFace::CEILING => "ceiling",
        }
    }
}

/// The constant's name, like any enum's `toString`.
impl fmt::Display for AttachFace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl PropertyEnum for AttachFace {
    const CLASS: &'static str = "net.minecraft.world.level.block.state.properties.AttachFace";
}
