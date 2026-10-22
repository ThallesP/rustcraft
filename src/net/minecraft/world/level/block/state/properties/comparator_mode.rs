//! net/minecraft/world/level/block/state/properties/ComparatorMode.java

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// Whether a comparator compares or subtracts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ComparatorMode {
    COMPARE,
    SUBTRACT,
}

impl ComparatorMode {
    pub const VALUES: [ComparatorMode; 2] = [ComparatorMode::COMPARE, ComparatorMode::SUBTRACT];

    pub fn values() -> [ComparatorMode; 2] {
        ComparatorMode::VALUES
    }
}

impl Enum for ComparatorMode {
    const ENUM_CONSTANTS: &'static [ComparatorMode] = &ComparatorMode::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for ComparatorMode {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            ComparatorMode::COMPARE => "compare",
            ComparatorMode::SUBTRACT => "subtract",
        }
    }
}

/// Java's `toString`, the serialized name.
impl fmt::Display for ComparatorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.get_serialized_name())
    }
}

impl PropertyEnum for ComparatorMode {
    const CLASS: &'static str = "net.minecraft.world.level.block.state.properties.ComparatorMode";
}
