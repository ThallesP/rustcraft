//! net/minecraft/commands/arguments/coordinates/WorldCoordinate.java
//!
//! Not ported yet: `get` and `parseDouble`, which come with their callers.

use std::sync::LazyLock;

use crate::com::mojang::brigadier::exceptions::command_syntax_exception::CommandSyntaxException;
use crate::com::mojang::brigadier::exceptions::simple_command_exception_type::SimpleCommandExceptionType;
use crate::com::mojang::brigadier::string_reader::StringReader;
use crate::net::minecraft::commands::arguments::coordinates::vec3_argument;
use crate::net::minecraft::network::chat::component::Component;

pub static ERROR_EXPECTED_DOUBLE: LazyLock<SimpleCommandExceptionType> = LazyLock::new(|| {
    SimpleCommandExceptionType::new(Component::from(Component::translatable(
        "argument.pos.missing.double",
    )))
});
pub static ERROR_EXPECTED_INT: LazyLock<SimpleCommandExceptionType> = LazyLock::new(|| {
    SimpleCommandExceptionType::new(Component::from(Component::translatable(
        "argument.pos.missing.int",
    )))
});

const PREFIX_RELATIVE: u16 = b'~' as u16;
const PREFIX_LOCAL: u16 = b'^' as u16;
const SPACE: u16 = b' ' as u16;

/// A coordinate, absolute or relative to the source's (`~`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldCoordinate {
    pub relative: bool,
    pub value: f64,
}

impl WorldCoordinate {
    /// A block coordinate: an int, or `~` and an optional double.
    pub fn parse_int(reader: &mut StringReader) -> Result<WorldCoordinate, CommandSyntaxException> {
        if reader.can_read() && reader.peek() == PREFIX_LOCAL {
            return Err(vec3_argument::ERROR_MIXED_TYPE.create_with_context(reader));
        }
        if !reader.can_read() {
            return Err(ERROR_EXPECTED_INT.create_with_context(reader));
        }
        let relative = WorldCoordinate::is_relative_at(reader);
        let value = if reader.can_read() && reader.peek() != SPACE {
            if relative {
                reader.read_double()?
            } else {
                f64::from(reader.read_int()?)
            }
        } else {
            0.0
        };
        Ok(WorldCoordinate { relative, value })
    }

    /// `isRelative(StringReader)`: skips a `~`, saying whether there was one.
    pub fn is_relative_at(reader: &mut StringReader) -> bool {
        if reader.peek() == PREFIX_RELATIVE {
            reader.skip();
            true
        } else {
            false
        }
    }

    pub fn is_relative(&self) -> bool {
        self.relative
    }
}
