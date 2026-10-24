//! net/minecraft/commands/arguments/coordinates/LocalCoordinates.java
//!
//! Not ported yet: `getPosition`, `apply` and `getRotation`, which a command source's rotation
//! reads.

use crate::com::mojang::brigadier::exceptions::command_syntax_exception::CommandSyntaxException;
use crate::com::mojang::brigadier::string_reader::StringReader;
use crate::net::minecraft::commands::arguments::coordinates::vec3_argument;
use crate::net::minecraft::commands::arguments::coordinates::world_coordinate;

const PREFIX_LOCAL_COORDINATE: u16 = b'^' as u16;
const SPACE: u16 = b' ' as u16;

/// Coordinates along the source's facing: `^left ^up ^forwards`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalCoordinates {
    pub left: f64,
    pub up: f64,
    pub forwards: f64,
}

impl LocalCoordinates {
    pub fn parse(reader: &mut StringReader) -> Result<LocalCoordinates, CommandSyntaxException> {
        let start = reader.get_cursor();
        let left = LocalCoordinates::read_double(reader, start)?;
        if !(reader.can_read() && reader.peek() == SPACE) {
            reader.set_cursor(start);
            return Err(vec3_argument::ERROR_NOT_COMPLETE.create_with_context(reader));
        }
        reader.skip();
        let up = LocalCoordinates::read_double(reader, start)?;
        if !(reader.can_read() && reader.peek() == SPACE) {
            reader.set_cursor(start);
            return Err(vec3_argument::ERROR_NOT_COMPLETE.create_with_context(reader));
        }
        reader.skip();
        let forwards = LocalCoordinates::read_double(reader, start)?;
        Ok(LocalCoordinates { left, up, forwards })
    }

    fn read_double(reader: &mut StringReader, start: i32) -> Result<f64, CommandSyntaxException> {
        if !reader.can_read() {
            return Err(world_coordinate::ERROR_EXPECTED_DOUBLE.create_with_context(reader));
        }
        if reader.peek() != PREFIX_LOCAL_COORDINATE {
            reader.set_cursor(start);
            return Err(vec3_argument::ERROR_MIXED_TYPE.create_with_context(reader));
        }
        reader.skip();
        if reader.can_read() && reader.peek() != SPACE {
            reader.read_double()
        } else {
            Ok(0.0)
        }
    }
}
