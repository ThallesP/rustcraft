//! net/minecraft/world/level/block/state/properties/PistonType.java

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// Whether a piston is sticky.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PistonType {
    DEFAULT,
    STICKY,
}

impl PistonType {
    pub const VALUES: [PistonType; 2] = [PistonType::DEFAULT, PistonType::STICKY];

    pub fn values() -> [PistonType; 2] {
        PistonType::VALUES
    }
}

impl Enum for PistonType {
    const ENUM_CONSTANTS: &'static [PistonType] = &PistonType::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for PistonType {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            PistonType::DEFAULT => "normal",
            PistonType::STICKY => "sticky",
        }
    }
}

/// Java's `toString`, the serialized name.
impl fmt::Display for PistonType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.get_serialized_name())
    }
}

impl PropertyEnum for PistonType {
    const CLASS: &'static str = "net.minecraft.world.level.block.state.properties.PistonType";
}
