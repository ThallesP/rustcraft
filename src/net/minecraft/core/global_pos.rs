//! net/minecraft/core/GlobalPos.java
//!
//! Not ported yet: `MAP_CODEC` and `CODEC`, which wait for DataFixerUpper's codecs.

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::resources::resource_key::ResourceKey;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, GlobalPos> = LazyLock::new(|| {
    Box::new(stream_codec::composite2(
        ResourceKey::stream_codec(Registries::DIMENSION),
        GlobalPos::dimension,
        BlockPos::STREAM_CODEC,
        GlobalPos::pos,
        GlobalPos::of,
    ))
});

/// A block in a dimension.
#[derive(Clone, Debug, PartialEq)]
pub struct GlobalPos {
    dimension: ResourceKey,
    pos: BlockPos,
}

impl GlobalPos {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, GlobalPos> = &STREAM_CODEC;

    pub fn new(dimension: ResourceKey, pos: BlockPos) -> GlobalPos {
        GlobalPos { dimension, pos }
    }

    pub fn of(dimension: ResourceKey, pos: BlockPos) -> GlobalPos {
        GlobalPos::new(dimension, pos)
    }

    pub fn dimension(&self) -> ResourceKey {
        self.dimension.clone()
    }

    pub fn pos(&self) -> BlockPos {
        self.pos
    }

    pub fn is_close_enough(
        &self,
        dimension: &ResourceKey,
        pos: &BlockPos,
        max_distance: i32,
    ) -> bool {
        self.dimension == *dimension && self.pos.dist_chessboard(pos) <= max_distance
    }
}

/// `ResourceKey[minecraft:dimension / minecraft:overworld] BlockPos{x=0, y=0, z=0}`.
impl fmt::Display for GlobalPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.dimension, self.pos)
    }
}
