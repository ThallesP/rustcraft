//! net/minecraft/world/level/ChunkPos.java
//!
//! Only what the chunk packets and the client's chunk storage use: the record, `pack`, `unpack`,
//! `containing`, its blocks' bounds and `STREAM_CODEC`. Not ported yet: `CODEC`
//! (DataFixerUpper), the regions, `hashCode`'s mixer (`hash`), the iterators and distances.

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ChunkPos> = LazyLock::new(|| {
    Box::new(stream_codec::of(
        |output: &mut ByteBuf, value: &ChunkPos| FriendlyByteBuf::write_chunk_pos_to(output, value),
        FriendlyByteBuf::read_chunk_pos_from,
    ))
});

/// A chunk's column, in chunks: block x and z over 16.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    x: i32,
    z: i32,
}

impl ChunkPos {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ChunkPos> = &STREAM_CODEC;
    pub const INVALID_CHUNK_POS: i64 = ChunkPos::pack_xz(1875066, 1875066);
    pub const ZERO: ChunkPos = ChunkPos::new(0, 0);

    pub const fn new(x: i32, z: i32) -> ChunkPos {
        ChunkPos { x, z }
    }

    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn z(&self) -> i32 {
        self.z
    }

    pub fn containing(pos: &BlockPos) -> ChunkPos {
        ChunkPos::new(
            SectionPos::block_to_section_coord(pos.get_x()),
            SectionPos::block_to_section_coord(pos.get_z()),
        )
    }

    /// `unpack(long)`: x in the low 32 bits, z in the high.
    pub fn unpack(key: i64) -> ChunkPos {
        ChunkPos::new(key as i32, (key >> 32) as i32)
    }

    pub fn pack(&self) -> i64 {
        ChunkPos::pack_xz(self.x, self.z)
    }

    /// `pack(int, int)`.
    pub const fn pack_xz(x: i32, z: i32) -> i64 {
        (x as i64 & 4294967295) | (z as i64 & 4294967295) << 32
    }

    pub fn get_min_block_x(&self) -> i32 {
        SectionPos::section_to_block_coord(self.x)
    }

    pub fn get_min_block_z(&self) -> i32 {
        SectionPos::section_to_block_coord(self.z)
    }

    pub fn get_max_block_x(&self) -> i32 {
        SectionPos::section_to_block_coord_offset(self.x, 15)
    }

    pub fn get_max_block_z(&self) -> i32 {
        SectionPos::section_to_block_coord_offset(self.z, 15)
    }
}

/// A record's: `ChunkPos[x=1, z=-2]`.
impl fmt::Display for ChunkPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ChunkPos[x={}, z={}]", self.x, self.z)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::ChunkPos;
    use crate::golden::encode_fields;

    encode_fields!(ChunkPos { x: i32, z: i32 });
}
