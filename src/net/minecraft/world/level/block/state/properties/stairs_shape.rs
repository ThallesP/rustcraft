//! net/minecraft/world/level/block/state/properties/StairsShape.java

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// Whether a stair is straight or turns a corner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StairsShape {
    STRAIGHT,
    INNER_LEFT,
    INNER_RIGHT,
    OUTER_LEFT,
    OUTER_RIGHT,
}

impl StairsShape {
    pub const VALUES: [StairsShape; 5] = [
        StairsShape::STRAIGHT,
        StairsShape::INNER_LEFT,
        StairsShape::INNER_RIGHT,
        StairsShape::OUTER_LEFT,
        StairsShape::OUTER_RIGHT,
    ];

    pub fn values() -> [StairsShape; 5] {
        StairsShape::VALUES
    }
}

impl Enum for StairsShape {
    const ENUM_CONSTANTS: &'static [StairsShape] = &StairsShape::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for StairsShape {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            StairsShape::STRAIGHT => "straight",
            StairsShape::INNER_LEFT => "inner_left",
            StairsShape::INNER_RIGHT => "inner_right",
            StairsShape::OUTER_LEFT => "outer_left",
            StairsShape::OUTER_RIGHT => "outer_right",
        }
    }
}

/// Java's `toString`, the serialized name.
impl fmt::Display for StairsShape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.get_serialized_name())
    }
}

impl PropertyEnum for StairsShape {
    const CLASS: &'static str = "net.minecraft.world.level.block.state.properties.StairsShape";
}
