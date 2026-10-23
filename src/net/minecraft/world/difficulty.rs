//! net/minecraft/world/Difficulty.java
//!
//! Not ported yet: `CODEC`, which waits for DataFixerUpper's codecs, and `getDisplayName` and
//! `getInfo`, which wait for text components. `byName` looks the serialized names up the way
//! `CODEC`'s `byName` does, the constant whose name it is or none.

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::util::by_id_map::{ByIdMap, OutOfBoundsStrategy};
use crate::net::minecraft::util::string_representable::StringRepresentable;

/// How hard the game is, which the server sets for every player.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Difficulty {
    PEACEFUL,
    EASY,
    NORMAL,
    HARD,
}

// By id, wrapping round for any other.
static BY_ID: LazyLock<Box<dyn Fn(i32) -> Difficulty + Send + Sync>> = LazyLock::new(|| {
    Box::new(ByIdMap::continuous(
        Difficulty::get_id,
        &Difficulty::VALUES,
        OutOfBoundsStrategy::WRAP,
    ))
});

static STREAM_CODEC: StaticStreamCodec<ByteBuf, Difficulty> =
    LazyLock::new(|| Box::new(ByteBufCodecs::id_mapper(BY_ID.as_ref(), Difficulty::get_id)));

impl Difficulty {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, Difficulty> = &STREAM_CODEC;

    pub const VALUES: [Difficulty; 4] = [
        Difficulty::PEACEFUL,
        Difficulty::EASY,
        Difficulty::NORMAL,
        Difficulty::HARD,
    ];

    pub fn values() -> [Difficulty; 4] {
        Difficulty::VALUES
    }

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> Difficulty {
        Difficulty::VALUES
            .into_iter()
            .find(|difficulty| format!("{difficulty:?}") == name)
            .unwrap_or_else(|| panic!("IllegalArgumentException: no Difficulty constant {name}"))
    }

    pub fn get_id(&self) -> i32 {
        match self {
            Difficulty::PEACEFUL => 0,
            Difficulty::EASY => 1,
            Difficulty::NORMAL => 2,
            Difficulty::HARD => 3,
        }
    }

    pub fn by_id(id: i32) -> Difficulty {
        BY_ID(id)
    }

    pub fn by_name(name: &str) -> Option<Difficulty> {
        Difficulty::VALUES
            .into_iter()
            .find(|difficulty| difficulty.get_serialized_name() == name)
    }
}

impl StringRepresentable for Difficulty {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            Difficulty::PEACEFUL => "peaceful",
            Difficulty::EASY => "easy",
            Difficulty::NORMAL => "normal",
            Difficulty::HARD => "hard",
        }
    }
}

/// The constant's name, like any enum's `toString`.
impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::Difficulty;
    use crate::golden::{Golden, encode_names};
    use crate::net::minecraft::util::string_representable::StringRepresentable;

    encode_names!(Difficulty {
        PEACEFUL,
        EASY,
        NORMAL,
        HARD
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.Difficulty");
        golden.pending("CODEC", "waits for DataFixerUpper's codecs");
        golden.pending(
            "getDisplayName()Lnet/minecraft/network/chat/Component;",
            "waits for text components",
        );
        golden.pending(
            "getInfo()Lnet/minecraft/network/chat/Component;",
            "waits for text components",
        );
        golden.check_codec("STREAM_CODEC", || Difficulty::STREAM_CODEC);
        for difficulty in Difficulty::VALUES {
            golden.field(&format!("{difficulty:?}"), difficulty);
        }
        golden.check(
            "values()[Lnet/minecraft/world/Difficulty;",
            Difficulty::values,
        );
        golden.check(
            "valueOf(Ljava/lang/String;)Lnet/minecraft/world/Difficulty;",
            |name: String| Difficulty::value_of(&name),
        );
        golden.check_method("getId()I", |this: &mut Difficulty| this.get_id());
        golden.check("byId(I)Lnet/minecraft/world/Difficulty;", Difficulty::by_id);
        golden.check(
            "byName(Ljava/lang/String;)Lnet/minecraft/world/Difficulty;",
            |name: String| Difficulty::by_name(&name),
        );
        golden.check_method(
            "getSerializedName()Ljava/lang/String;",
            |this: &mut Difficulty| this.get_serialized_name().to_owned(),
        );
        golden.finish();
    }
}
