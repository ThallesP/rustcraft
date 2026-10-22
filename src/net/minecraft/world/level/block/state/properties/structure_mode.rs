//! net/minecraft/world/level/block/state/properties/StructureMode.java
//!
//! Not ported yet: `getDisplayName`, `LEGACY_CODEC`.

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// What a structure block does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StructureMode {
    SAVE,
    LOAD,
    CORNER,
    DATA,
}

impl StructureMode {
    pub const VALUES: [StructureMode; 4] = [
        StructureMode::SAVE,
        StructureMode::LOAD,
        StructureMode::CORNER,
        StructureMode::DATA,
    ];

    pub fn values() -> [StructureMode; 4] {
        StructureMode::VALUES
    }
}

impl Enum for StructureMode {
    const ENUM_CONSTANTS: &'static [StructureMode] = &StructureMode::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for StructureMode {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            StructureMode::SAVE => "save",
            StructureMode::LOAD => "load",
            StructureMode::CORNER => "corner",
            StructureMode::DATA => "data",
        }
    }
}

/// The constant's name, like any enum's `toString`.
impl fmt::Display for StructureMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl PropertyEnum for StructureMode {
    const CLASS: &'static str = "net.minecraft.world.level.block.state.properties.StructureMode";
}
