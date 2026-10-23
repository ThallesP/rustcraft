//! net/minecraft/core/SectionPos.java
//!
//! Only what the chunk packets and the client's chunk storage use: making one, packing it into a
//! long and back (`STREAM_CODEC`), the coordinate conversions and a section-relative position's
//! block. Not ported yet: the streams (`blocksInside`, `cube`, `betweenClosedStream`), `offset`
//! and the entity and `Position` factories, which come with what calls them. Java's extends
//! `Vec3i`, which this one holds and derefs to.

use std::ops::Deref;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::vec3i::Vec3i;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{StaticStreamCodec, StreamCodec};
use crate::net::minecraft::world::level::chunk_pos::ChunkPos;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, SectionPos> = LazyLock::new(|| {
    Box::new(ByteBufCodecs::LONG.map(SectionPos::of_long, |pos: &SectionPos| pos.as_long()))
});

/// A section of 16³ blocks, in sections.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SectionPos {
    vec3i: Vec3i,
}

impl SectionPos {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, SectionPos> = &STREAM_CODEC;

    pub fn of(x: i32, y: i32, z: i32) -> SectionPos {
        SectionPos {
            vec3i: Vec3i::new(x, y, z),
        }
    }

    /// `of(BlockPos)`: the section the block is in.
    pub fn of_block_pos(pos: &BlockPos) -> SectionPos {
        SectionPos::of(
            SectionPos::block_to_section_coord(pos.get_x()),
            SectionPos::block_to_section_coord(pos.get_y()),
            SectionPos::block_to_section_coord(pos.get_z()),
        )
    }

    /// `of(ChunkPos, int)`.
    pub fn of_chunk_pos(pos: &ChunkPos, section_y: i32) -> SectionPos {
        SectionPos::of(pos.x(), section_y, pos.z())
    }

    /// `of(long)`: the section a packed long stands for.
    pub fn of_long(section_node: i64) -> SectionPos {
        SectionPos::of(
            SectionPos::x_of(section_node),
            SectionPos::y_of(section_node),
            SectionPos::z_of(section_node),
        )
    }

    pub fn block_to_section_coord(block_coord: i32) -> i32 {
        block_coord >> 4
    }

    pub fn section_relative(block_coord: i32) -> i32 {
        block_coord & 15
    }

    /// `sectionRelativePos(BlockPos)`: x, z and y within the section, in that order from the high
    /// bits.
    pub fn section_relative_pos(pos: &BlockPos) -> i16 {
        let x = SectionPos::section_relative(pos.get_x());
        let y = SectionPos::section_relative(pos.get_y());
        let z = SectionPos::section_relative(pos.get_z());
        (x << 8 | z << 4 | y) as i16
    }

    pub fn section_relative_x(relative: i16) -> i32 {
        (relative as i32 as u32 >> 8) as i32 & 15
    }

    pub fn section_relative_y(relative: i16) -> i32 {
        relative as i32 & 15
    }

    pub fn section_relative_z(relative: i16) -> i32 {
        (relative as i32 as u32 >> 4) as i32 & 15
    }

    pub fn relative_to_block_x(&self, relative: i16) -> i32 {
        self.min_block_x()
            .wrapping_add(SectionPos::section_relative_x(relative))
    }

    pub fn relative_to_block_y(&self, relative: i16) -> i32 {
        self.min_block_y()
            .wrapping_add(SectionPos::section_relative_y(relative))
    }

    pub fn relative_to_block_z(&self, relative: i16) -> i32 {
        self.min_block_z()
            .wrapping_add(SectionPos::section_relative_z(relative))
    }

    pub fn relative_to_block_pos(&self, relative: i16) -> BlockPos {
        BlockPos::new(
            self.relative_to_block_x(relative),
            self.relative_to_block_y(relative),
            self.relative_to_block_z(relative),
        )
    }

    pub fn section_to_block_coord(section_coord: i32) -> i32 {
        section_coord.wrapping_shl(4)
    }

    /// `sectionToBlockCoord(int, int)`.
    pub fn section_to_block_coord_offset(section_coord: i32, offset: i32) -> i32 {
        SectionPos::section_to_block_coord(section_coord).wrapping_add(offset)
    }

    /// `x(long)`: 22 bits from the top.
    pub fn x_of(section_node: i64) -> i32 {
        (section_node >> 42) as i32
    }

    /// `y(long)`: the low 20 bits.
    pub fn y_of(section_node: i64) -> i32 {
        (section_node << 44 >> 44) as i32
    }

    /// `z(long)`: the 22 bits below x's.
    pub fn z_of(section_node: i64) -> i32 {
        (section_node << 22 >> 42) as i32
    }

    pub fn x(&self) -> i32 {
        self.get_x()
    }

    pub fn y(&self) -> i32 {
        self.get_y()
    }

    pub fn z(&self) -> i32 {
        self.get_z()
    }

    pub fn min_block_x(&self) -> i32 {
        SectionPos::section_to_block_coord(self.x())
    }

    pub fn min_block_y(&self) -> i32 {
        SectionPos::section_to_block_coord(self.y())
    }

    pub fn min_block_z(&self) -> i32 {
        SectionPos::section_to_block_coord(self.z())
    }

    pub fn chunk(&self) -> ChunkPos {
        ChunkPos::new(self.x(), self.z())
    }

    /// `asLong(int, int, int)`: x in the top 22 bits, z in the next 22, y in the low 20.
    pub fn as_long_xyz(x: i32, y: i32, z: i32) -> i64 {
        let mut node = 0i64;
        node |= (x as i64 & 4194303) << 42;
        node |= y as i64 & 1048575;
        node | (z as i64 & 4194303) << 20
    }

    pub fn as_long(&self) -> i64 {
        SectionPos::as_long_xyz(self.x(), self.y(), self.z())
    }
}

impl Deref for SectionPos {
    type Target = Vec3i;

    fn deref(&self) -> &Vec3i {
        &self.vec3i
    }
}
