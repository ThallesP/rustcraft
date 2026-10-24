//! net/minecraft/commands/arguments/coordinates/BlockPosArgument.java
//!
//! Only `parse`, which the NBT contents' block data source reads its coordinates with. The
//! argument's lookups in a command's context come with commands.

use crate::com::mojang::brigadier::exceptions::command_syntax_exception::CommandSyntaxException;
use crate::com::mojang::brigadier::string_reader::StringReader;
use crate::net::minecraft::commands::arguments::coordinates::coordinates::Coordinates;
use crate::net::minecraft::commands::arguments::coordinates::local_coordinates::LocalCoordinates;
use crate::net::minecraft::commands::arguments::coordinates::world_coordinates::WorldCoordinates;

pub struct BlockPosArgument;

impl BlockPosArgument {
    pub fn block_pos() -> BlockPosArgument {
        BlockPosArgument
    }

    /// Local coordinates when they start with `^`, else world coordinates.
    pub fn parse(&self, reader: &mut StringReader) -> Result<Coordinates, CommandSyntaxException> {
        if reader.can_read() && reader.peek() == u16::from(b'^') {
            LocalCoordinates::parse(reader).map(Coordinates::Local)
        } else {
            WorldCoordinates::parse_int(reader).map(Coordinates::World)
        }
    }
}
