//! net/minecraft/world/level/block/state/properties/DoubleBlockHalf.java
//!
//! Not ported yet: `getDirectionToOther`, `getOtherHalf`.

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// Which half of a two-block-tall block a block is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DoubleBlockHalf {
    UPPER,
    LOWER,
}

impl DoubleBlockHalf {
    pub const VALUES: [DoubleBlockHalf; 2] = [DoubleBlockHalf::UPPER, DoubleBlockHalf::LOWER];

    pub fn values() -> [DoubleBlockHalf; 2] {
        DoubleBlockHalf::VALUES
    }
}

impl Enum for DoubleBlockHalf {
    const ENUM_CONSTANTS: &'static [DoubleBlockHalf] = &DoubleBlockHalf::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for DoubleBlockHalf {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            DoubleBlockHalf::UPPER => "upper",
            DoubleBlockHalf::LOWER => "lower",
        }
    }
}

/// Java's `toString`, the serialized name.
impl fmt::Display for DoubleBlockHalf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.get_serialized_name())
    }
}

impl PropertyEnum for DoubleBlockHalf {
    const CLASS: &'static str = "net.minecraft.world.level.block.state.properties.DoubleBlockHalf";
}
