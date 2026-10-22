//! net/minecraft/world/level/block/state/properties/RailShape.java
//!
//! Not ported yet: `getName`, `isSlope`.

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// Which way a rail runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RailShape {
    NORTH_SOUTH,
    EAST_WEST,
    ASCENDING_EAST,
    ASCENDING_WEST,
    ASCENDING_NORTH,
    ASCENDING_SOUTH,
    SOUTH_EAST,
    SOUTH_WEST,
    NORTH_WEST,
    NORTH_EAST,
}

impl RailShape {
    pub const VALUES: [RailShape; 10] = [
        RailShape::NORTH_SOUTH,
        RailShape::EAST_WEST,
        RailShape::ASCENDING_EAST,
        RailShape::ASCENDING_WEST,
        RailShape::ASCENDING_NORTH,
        RailShape::ASCENDING_SOUTH,
        RailShape::SOUTH_EAST,
        RailShape::SOUTH_WEST,
        RailShape::NORTH_WEST,
        RailShape::NORTH_EAST,
    ];

    pub fn values() -> [RailShape; 10] {
        RailShape::VALUES
    }
}

impl Enum for RailShape {
    const ENUM_CONSTANTS: &'static [RailShape] = &RailShape::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for RailShape {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            RailShape::NORTH_SOUTH => "north_south",
            RailShape::EAST_WEST => "east_west",
            RailShape::ASCENDING_EAST => "ascending_east",
            RailShape::ASCENDING_WEST => "ascending_west",
            RailShape::ASCENDING_NORTH => "ascending_north",
            RailShape::ASCENDING_SOUTH => "ascending_south",
            RailShape::SOUTH_EAST => "south_east",
            RailShape::SOUTH_WEST => "south_west",
            RailShape::NORTH_WEST => "north_west",
            RailShape::NORTH_EAST => "north_east",
        }
    }
}

/// Java's `toString`, the serialized name.
impl fmt::Display for RailShape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.get_serialized_name())
    }
}

impl PropertyEnum for RailShape {
    const CLASS: &'static str = "net.minecraft.world.level.block.state.properties.RailShape";
}
