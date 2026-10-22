//! net/minecraft/world/level/block/state/properties/BellAttachType.java

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// What a bell hangs from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BellAttachType {
    FLOOR,
    CEILING,
    SINGLE_WALL,
    DOUBLE_WALL,
}

impl BellAttachType {
    pub const VALUES: [BellAttachType; 4] = [
        BellAttachType::FLOOR,
        BellAttachType::CEILING,
        BellAttachType::SINGLE_WALL,
        BellAttachType::DOUBLE_WALL,
    ];

    pub fn values() -> [BellAttachType; 4] {
        BellAttachType::VALUES
    }
}

impl Enum for BellAttachType {
    const ENUM_CONSTANTS: &'static [BellAttachType] = &BellAttachType::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for BellAttachType {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            BellAttachType::FLOOR => "floor",
            BellAttachType::CEILING => "ceiling",
            BellAttachType::SINGLE_WALL => "single_wall",
            BellAttachType::DOUBLE_WALL => "double_wall",
        }
    }
}

/// The constant's name, like any enum's `toString`.
impl fmt::Display for BellAttachType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl PropertyEnum for BellAttachType {
    const CLASS: &'static str = "net.minecraft.world.level.block.state.properties.BellAttachType";
}
