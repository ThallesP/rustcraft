//! net/minecraft/world/level/block/state/properties/ChestType.java
//!
//! Not ported yet: `getOpposite`, `CODEC`.

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// Whether a chest is single or which half of a double one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChestType {
    SINGLE,
    LEFT,
    RIGHT,
}

impl ChestType {
    pub const VALUES: [ChestType; 3] = [ChestType::SINGLE, ChestType::LEFT, ChestType::RIGHT];

    pub fn values() -> [ChestType; 3] {
        ChestType::VALUES
    }
}

impl Enum for ChestType {
    const ENUM_CONSTANTS: &'static [ChestType] = &ChestType::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for ChestType {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            ChestType::SINGLE => "single",
            ChestType::LEFT => "left",
            ChestType::RIGHT => "right",
        }
    }
}

/// The constant's name, like any enum's `toString`.
impl fmt::Display for ChestType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl PropertyEnum for ChestType {
    const CLASS: &'static str = "net.minecraft.world.level.block.state.properties.ChestType";
}
