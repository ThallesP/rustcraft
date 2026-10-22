//! net/minecraft/world/level/block/state/properties/DoorHingeSide.java

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// Which side a door's hinge is on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DoorHingeSide {
    LEFT,
    RIGHT,
}

impl DoorHingeSide {
    pub const VALUES: [DoorHingeSide; 2] = [DoorHingeSide::LEFT, DoorHingeSide::RIGHT];

    pub fn values() -> [DoorHingeSide; 2] {
        DoorHingeSide::VALUES
    }
}

impl Enum for DoorHingeSide {
    const ENUM_CONSTANTS: &'static [DoorHingeSide] = &DoorHingeSide::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for DoorHingeSide {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            DoorHingeSide::LEFT => "left",
            DoorHingeSide::RIGHT => "right",
        }
    }
}

/// Java's `toString`, the serialized name.
impl fmt::Display for DoorHingeSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.get_serialized_name())
    }
}

impl PropertyEnum for DoorHingeSide {
    const CLASS: &'static str = "net.minecraft.world.level.block.state.properties.DoorHingeSide";
}
