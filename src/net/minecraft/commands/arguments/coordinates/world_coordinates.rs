//! net/minecraft/commands/arguments/coordinates/WorldCoordinates.java
//!
//! Not ported yet: `getPosition`, `getRotation` (a command source's), `parseDouble` and `absolute`,
//! which come with commands.

use crate::com::mojang::brigadier::exceptions::command_syntax_exception::CommandSyntaxException;
use crate::com::mojang::brigadier::string_reader::StringReader;
use crate::net::minecraft::commands::arguments::coordinates::vec3_argument;
use crate::net::minecraft::commands::arguments::coordinates::world_coordinate::WorldCoordinate;

const SPACE: u16 = b' ' as u16;

/// Three coordinates, each absolute or relative.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldCoordinates {
    pub x: WorldCoordinate,
    pub y: WorldCoordinate,
    pub z: WorldCoordinate,
}

impl WorldCoordinates {
    /// A block's three coordinates, separated by spaces.
    pub fn parse_int(
        reader: &mut StringReader,
    ) -> Result<WorldCoordinates, CommandSyntaxException> {
        let start = reader.get_cursor();
        let x = WorldCoordinate::parse_int(reader)?;
        if !(reader.can_read() && reader.peek() == SPACE) {
            reader.set_cursor(start);
            return Err(vec3_argument::ERROR_NOT_COMPLETE.create_with_context(reader));
        }
        reader.skip();
        let y = WorldCoordinate::parse_int(reader)?;
        if !(reader.can_read() && reader.peek() == SPACE) {
            reader.set_cursor(start);
            return Err(vec3_argument::ERROR_NOT_COMPLETE.create_with_context(reader));
        }
        reader.skip();
        let z = WorldCoordinate::parse_int(reader)?;
        Ok(WorldCoordinates { x, y, z })
    }
}
