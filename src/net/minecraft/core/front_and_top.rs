//! net/minecraft/core/FrontAndTop.java
//!
//! Not ported yet: `fromFrontAndTop`, `front`, `top`.

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::util::string_representable::StringRepresentable;
use crate::net::minecraft::world::level::block::state::properties::property::PropertyEnum;

/// Which ways a jigsaw or crafter faces: its front, then its top.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FrontAndTop {
    DOWN_EAST,
    DOWN_NORTH,
    DOWN_SOUTH,
    DOWN_WEST,
    UP_EAST,
    UP_NORTH,
    UP_SOUTH,
    UP_WEST,
    WEST_UP,
    EAST_UP,
    NORTH_UP,
    SOUTH_UP,
}

impl FrontAndTop {
    pub const VALUES: [FrontAndTop; 12] = [
        FrontAndTop::DOWN_EAST,
        FrontAndTop::DOWN_NORTH,
        FrontAndTop::DOWN_SOUTH,
        FrontAndTop::DOWN_WEST,
        FrontAndTop::UP_EAST,
        FrontAndTop::UP_NORTH,
        FrontAndTop::UP_SOUTH,
        FrontAndTop::UP_WEST,
        FrontAndTop::WEST_UP,
        FrontAndTop::EAST_UP,
        FrontAndTop::NORTH_UP,
        FrontAndTop::SOUTH_UP,
    ];

    pub fn values() -> [FrontAndTop; 12] {
        FrontAndTop::VALUES
    }
}

impl Enum for FrontAndTop {
    const ENUM_CONSTANTS: &'static [FrontAndTop] = &FrontAndTop::VALUES;

    fn ordinal(&self) -> i32 {
        *self as i32
    }
}

impl StringRepresentable for FrontAndTop {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            FrontAndTop::DOWN_EAST => "down_east",
            FrontAndTop::DOWN_NORTH => "down_north",
            FrontAndTop::DOWN_SOUTH => "down_south",
            FrontAndTop::DOWN_WEST => "down_west",
            FrontAndTop::UP_EAST => "up_east",
            FrontAndTop::UP_NORTH => "up_north",
            FrontAndTop::UP_SOUTH => "up_south",
            FrontAndTop::UP_WEST => "up_west",
            FrontAndTop::WEST_UP => "west_up",
            FrontAndTop::EAST_UP => "east_up",
            FrontAndTop::NORTH_UP => "north_up",
            FrontAndTop::SOUTH_UP => "south_up",
        }
    }
}

/// The constant's name, like any enum's `toString`.
impl fmt::Display for FrontAndTop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl PropertyEnum for FrontAndTop {
    const CLASS: &'static str = "net.minecraft.core.FrontAndTop";
}
