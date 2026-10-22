//! net/minecraft/world/level/block/state/properties/SlabType.java

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// Which half of a block a slab fills, or both.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SlabType {
    TOP,
    BOTTOM,
    DOUBLE,
}

impl SlabType {
    pub const VALUES: [SlabType; 3] = [SlabType::TOP, SlabType::BOTTOM, SlabType::DOUBLE];

    pub fn values() -> [SlabType; 3] {
        SlabType::VALUES
    }
}

impl Enum for SlabType {
    const ENUM_CONSTANTS: &'static [SlabType] = &SlabType::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for SlabType {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            SlabType::TOP => "top",
            SlabType::BOTTOM => "bottom",
            SlabType::DOUBLE => "double",
        }
    }
}

/// Java's `toString`, the serialized name.
impl fmt::Display for SlabType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.get_serialized_name())
    }
}

impl PropertyEnum for SlabType {
    const CLASS: &'static str = "net.minecraft.world.level.block.state.properties.SlabType";
}
