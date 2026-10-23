//! net/minecraft/core/FrontAndTop.java

use std::fmt;

use crate::java::lang::r#enum::Enum;
use crate::net::minecraft::core::direction::Direction;
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

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> FrontAndTop {
        FrontAndTop::VALUES
            .into_iter()
            .find(|value| format!("{value:?}") == name)
            .unwrap_or_else(|| panic!("IllegalArgumentException: no FrontAndTop constant {name}"))
    }

    /// The constant facing `front` with `top` up, or `None` for a pair no constant has
    /// (`BY_TOP_FRONT`'s nulls).
    pub fn from_front_and_top(front: Direction, top: Direction) -> Option<FrontAndTop> {
        FrontAndTop::VALUES
            .into_iter()
            .find(|value| value.front() == front && value.top() == top)
    }

    pub fn front(&self) -> Direction {
        self.front_and_top().0
    }

    pub fn top(&self) -> Direction {
        self.front_and_top().1
    }

    fn front_and_top(&self) -> (Direction, Direction) {
        match self {
            FrontAndTop::DOWN_EAST => (Direction::DOWN, Direction::EAST),
            FrontAndTop::DOWN_NORTH => (Direction::DOWN, Direction::NORTH),
            FrontAndTop::DOWN_SOUTH => (Direction::DOWN, Direction::SOUTH),
            FrontAndTop::DOWN_WEST => (Direction::DOWN, Direction::WEST),
            FrontAndTop::UP_EAST => (Direction::UP, Direction::EAST),
            FrontAndTop::UP_NORTH => (Direction::UP, Direction::NORTH),
            FrontAndTop::UP_SOUTH => (Direction::UP, Direction::SOUTH),
            FrontAndTop::UP_WEST => (Direction::UP, Direction::WEST),
            FrontAndTop::WEST_UP => (Direction::WEST, Direction::UP),
            FrontAndTop::EAST_UP => (Direction::EAST, Direction::UP),
            FrontAndTop::NORTH_UP => (Direction::NORTH, Direction::UP),
            FrontAndTop::SOUTH_UP => (Direction::SOUTH, Direction::UP),
        }
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

#[cfg(test)]
mod tests {
    use super::FrontAndTop;
    use crate::golden::{Golden, encode_names};
    use crate::net::minecraft::core::direction::Direction;
    use crate::net::minecraft::util::string_representable::StringRepresentable;

    encode_names!(FrontAndTop {
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
        SOUTH_UP
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.core.FrontAndTop");
        for value in FrontAndTop::VALUES {
            golden.field(&format!("{value:?}"), value);
        }
        golden.check(
            "values()[Lnet/minecraft/core/FrontAndTop;",
            FrontAndTop::values,
        );
        golden.check(
            "valueOf(Ljava/lang/String;)Lnet/minecraft/core/FrontAndTop;",
            |name: String| FrontAndTop::value_of(&name),
        );
        golden.check_method(
            "getSerializedName()Ljava/lang/String;",
            |this: &mut FrontAndTop| this.get_serialized_name().to_owned(),
        );
        golden.check(
            "fromFrontAndTop(Lnet/minecraft/core/Direction;Lnet/minecraft/core/Direction;)Lnet/minecraft/core/FrontAndTop;",
            |front: Direction, top: Direction| FrontAndTop::from_front_and_top(front, top),
        );
        golden.check_method(
            "front()Lnet/minecraft/core/Direction;",
            |this: &mut FrontAndTop| this.front(),
        );
        golden.check_method(
            "top()Lnet/minecraft/core/Direction;",
            |this: &mut FrontAndTop| this.top(),
        );
        golden.finish();
    }
}
