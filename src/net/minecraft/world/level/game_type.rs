//! net/minecraft/world/level/GameType.java
//!
//! Not ported yet: `CODEC` and `LEGACY_ID_CODEC`, which wait for DataFixerUpper's codecs, the
//! display names, which wait for text components, `updatePlayerAbilities`, which comes with
//! `Abilities`. `byName` is the codec's lookup by serialized name (`StringRepresentable`'s
//! `EnumCodec.byName`), written out.

use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{StaticStreamCodec, StreamCodec};
use crate::net::minecraft::util::by_id_map::{ByIdMap, OutOfBoundsStrategy};

/// A player's game mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameType {
    SURVIVAL,
    CREATIVE,
    ADVENTURE,
    SPECTATOR,
}

// By id, and survival for any other.
static BY_ID: LazyLock<Box<dyn Fn(i32) -> GameType + Send + Sync>> = LazyLock::new(|| {
    Box::new(ByIdMap::continuous(
        GameType::get_id,
        &GameType::VALUES,
        OutOfBoundsStrategy::ZERO,
    ))
});

static STREAM_CODEC: StaticStreamCodec<ByteBuf, GameType> =
    LazyLock::new(|| Box::new(ByteBufCodecs::id_mapper(BY_ID.as_ref(), GameType::get_id)));

static OPTIONAL_STREAM_CODEC: StaticStreamCodec<ByteBuf, Option<GameType>> = LazyLock::new(|| {
    Box::new(ByteBufCodecs::OPTIONAL_VAR_INT.map(
        |id| id.map(GameType::by_id),
        |game_type: &Option<GameType>| game_type.map(|game_type| game_type.get_id()),
    ))
});

impl GameType {
    pub const VALUES: [GameType; 4] = [
        GameType::SURVIVAL,
        GameType::CREATIVE,
        GameType::ADVENTURE,
        GameType::SPECTATOR,
    ];
    pub const DEFAULT_MODE: GameType = GameType::SURVIVAL;
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, GameType> = &STREAM_CODEC;
    pub const OPTIONAL_STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, Option<GameType>> =
        &OPTIONAL_STREAM_CODEC;

    pub fn values() -> [GameType; 4] {
        GameType::VALUES
    }

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> GameType {
        match name {
            "SURVIVAL" => GameType::SURVIVAL,
            "CREATIVE" => GameType::CREATIVE,
            "ADVENTURE" => GameType::ADVENTURE,
            "SPECTATOR" => GameType::SPECTATOR,
            _ => panic!("IllegalArgumentException: no GameType constant {name}"),
        }
    }

    pub fn get_id(&self) -> i32 {
        *self as i32
    }

    /// `byName(String, GameType)` with no default: the mode whose serialized name it is.
    pub fn by_name(name: &str) -> Option<GameType> {
        GameType::VALUES
            .into_iter()
            .find(|game_type| game_type.get_serialized_name() == name)
    }

    pub fn get_name(&self) -> &'static str {
        match self {
            GameType::SURVIVAL => "survival",
            GameType::CREATIVE => "creative",
            GameType::ADVENTURE => "adventure",
            GameType::SPECTATOR => "spectator",
        }
    }

    pub fn get_serialized_name(&self) -> &'static str {
        self.get_name()
    }

    pub fn is_block_placing_restricted(&self) -> bool {
        *self == GameType::ADVENTURE || *self == GameType::SPECTATOR
    }

    pub fn is_creative(&self) -> bool {
        *self == GameType::CREATIVE
    }

    pub fn is_survival(&self) -> bool {
        *self == GameType::SURVIVAL || *self == GameType::ADVENTURE
    }

    pub fn by_id(id: i32) -> GameType {
        BY_ID(id)
    }

    pub fn is_valid_id(id: i32) -> bool {
        GameType::VALUES
            .iter()
            .any(|game_type| game_type.get_id() == id)
    }
}
