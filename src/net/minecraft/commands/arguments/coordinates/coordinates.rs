//! net/minecraft/commands/arguments/coordinates/Coordinates.java
//!
//! A position a command names, in the world's coordinates or the source's facing. Java's
//! interface of the two is an enum here; what a command source makes of them (`getPosition`,
//! `getRotation`, `getBlockPos`) comes with commands.

use crate::net::minecraft::commands::arguments::coordinates::local_coordinates::LocalCoordinates;
use crate::net::minecraft::commands::arguments::coordinates::world_coordinates::WorldCoordinates;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Coordinates {
    World(WorldCoordinates),
    Local(LocalCoordinates),
}

impl Coordinates {
    pub fn is_x_relative(&self) -> bool {
        match self {
            Coordinates::World(world) => world.x.is_relative(),
            Coordinates::Local(_) => true,
        }
    }

    pub fn is_y_relative(&self) -> bool {
        match self {
            Coordinates::World(world) => world.y.is_relative(),
            Coordinates::Local(_) => true,
        }
    }

    pub fn is_z_relative(&self) -> bool {
        match self {
            Coordinates::World(world) => world.z.is_relative(),
            Coordinates::Local(_) => true,
        }
    }
}
