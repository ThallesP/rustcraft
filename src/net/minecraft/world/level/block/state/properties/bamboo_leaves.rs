//! net/minecraft/world/level/block/state/properties/BambooLeaves.java

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// How leafy a bamboo stalk is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BambooLeaves {
    NONE,
    SMALL,
    LARGE,
}

impl BambooLeaves {
    pub const VALUES: [BambooLeaves; 3] =
        [BambooLeaves::NONE, BambooLeaves::SMALL, BambooLeaves::LARGE];

    pub fn values() -> [BambooLeaves; 3] {
        BambooLeaves::VALUES
    }
}

impl Enum for BambooLeaves {
    const ENUM_CONSTANTS: &'static [BambooLeaves] = &BambooLeaves::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for BambooLeaves {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            BambooLeaves::NONE => "none",
            BambooLeaves::SMALL => "small",
            BambooLeaves::LARGE => "large",
        }
    }
}

/// Java's `toString`, the serialized name.
impl fmt::Display for BambooLeaves {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.get_serialized_name())
    }
}

impl PropertyEnum for BambooLeaves {
    const CLASS: &'static str = "net.minecraft.world.level.block.state.properties.BambooLeaves";
}
