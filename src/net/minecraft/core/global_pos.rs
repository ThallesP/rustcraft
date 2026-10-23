//! net/minecraft/core/GlobalPos.java
//!
//! Not ported yet: `MAP_CODEC` and `CODEC`, which wait for DataFixerUpper's codecs. The record's
//! `hashCode` hashes its key's identity hash, which changes from run to run.

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

#[cfg(test)]
mod tests {
    use super::GlobalPos;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::core::block_pos::BlockPos;
    use crate::net::minecraft::resources::resource_key::ResourceKey;

    encode_fields!(GlobalPos {
        dimension: ResourceKey,
        pos: BlockPos,
    });

    type This = GlobalPos;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.core.GlobalPos");
        for name in ["CODEC", "MAP_CODEC"] {
            golden.pending(name, "waits for DataFixerUpper's codecs");
        }
        golden.pending("hashCode()I", "hashes the key's identity hash");
        golden.check_codec("STREAM_CODEC", || GlobalPos::STREAM_CODEC);
        golden.check(
            "<init>(Lnet/minecraft/resources/ResourceKey;Lnet/minecraft/core/BlockPos;)V",
            GlobalPos::new,
        );
        golden.check(
            "of(Lnet/minecraft/resources/ResourceKey;Lnet/minecraft/core/BlockPos;)Lnet/minecraft/core/GlobalPos;",
            GlobalPos::of,
        );
        golden.check_method(
            "dimension()Lnet/minecraft/resources/ResourceKey;",
            |this: &mut This| this.dimension(),
        );
        golden.check_method("pos()Lnet/minecraft/core/BlockPos;", |this: &mut This| {
            this.pos()
        });
        golden.check_method(
            "isCloseEnough(Lnet/minecraft/resources/ResourceKey;Lnet/minecraft/core/BlockPos;I)Z",
            |this: &mut This, dimension: ResourceKey, pos: BlockPos, max_distance: i32| {
                this.is_close_enough(&dimension, &pos, max_distance)
            },
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut This, other: This| *this == other,
        );
        golden.check_method("toString()Ljava/lang/String;", |this: &mut This| {
            this.to_string()
        });
        golden.finish();
    }
}
