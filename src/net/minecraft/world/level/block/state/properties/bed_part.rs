//! net/minecraft/world/level/block/state/properties/BedPart.java
//!
//! Not ported yet: `CODEC`.

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// Which half of a bed a block is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BedPart {
    HEAD,
    FOOT,
}

impl BedPart {
    pub const VALUES: [BedPart; 2] = [BedPart::HEAD, BedPart::FOOT];

    pub fn values() -> [BedPart; 2] {
        BedPart::VALUES
    }
}

impl Enum for BedPart {
    const ENUM_CONSTANTS: &'static [BedPart] = &BedPart::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for BedPart {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            BedPart::HEAD => "head",
            BedPart::FOOT => "foot",
        }
    }
}

/// Java's `toString`, the serialized name.
impl fmt::Display for BedPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.get_serialized_name())
    }
}

impl PropertyEnum for BedPart {
    const CLASS: &'static str = "net.minecraft.world.level.block.state.properties.BedPart";
}
