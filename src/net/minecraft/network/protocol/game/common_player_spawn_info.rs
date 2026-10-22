//! net/minecraft/network/protocol/game/CommonPlayerSpawnInfo.java
//!
//! Not ported yet: the record's `hashCode`, which hashes the keys' identity hashes, and its
//! `toString`, which prints the holder the way `Holder.Reference` does.

use std::sync::LazyLock;

use crate::net::minecraft::core::global_pos::GlobalPos;
use crate::net::minecraft::core::holder::Holder;
use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::world::level::dimension::dimension_type::DimensionType;
use crate::net::minecraft::world::level::game_type::GameType;

static STREAM_CODEC: StaticStreamCodec<RegistryFriendlyByteBuf, CommonPlayerSpawnInfo> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite10(
            DimensionType::STREAM_CODEC,
            CommonPlayerSpawnInfo::dimension_type,
            ResourceKey::stream_codec(Registries::DIMENSION),
            CommonPlayerSpawnInfo::dimension,
            ByteBufCodecs::LONG,
            CommonPlayerSpawnInfo::seed,
            GameType::STREAM_CODEC,
            CommonPlayerSpawnInfo::game_type,
            GameType::OPTIONAL_STREAM_CODEC,
            CommonPlayerSpawnInfo::previous_game_type,
            ByteBufCodecs::BOOL,
            CommonPlayerSpawnInfo::is_debug,
            ByteBufCodecs::BOOL,
            CommonPlayerSpawnInfo::is_flat,
            ByteBufCodecs::optional(GlobalPos::STREAM_CODEC),
            CommonPlayerSpawnInfo::last_death_location,
            ByteBufCodecs::VAR_INT,
            CommonPlayerSpawnInfo::portal_cooldown,
            ByteBufCodecs::VAR_INT,
            CommonPlayerSpawnInfo::sea_level,
            CommonPlayerSpawnInfo::new,
        ))
    });

/// What a player joining a level needs to know about it, in the login and respawn packets.
#[derive(Clone, Debug, PartialEq)]
pub struct CommonPlayerSpawnInfo {
    dimension_type: Holder,
    dimension: ResourceKey,
    seed: i64,
    game_type: GameType,
    previous_game_type: Option<GameType>,
    is_debug: bool,
    is_flat: bool,
    last_death_location: Option<GlobalPos>,
    portal_cooldown: i32,
    sea_level: i32,
}

impl CommonPlayerSpawnInfo {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        RegistryFriendlyByteBuf,
        CommonPlayerSpawnInfo,
    > = &STREAM_CODEC;

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        dimension_type: Holder,
        dimension: ResourceKey,
        seed: i64,
        game_type: GameType,
        previous_game_type: Option<GameType>,
        is_debug: bool,
        is_flat: bool,
        last_death_location: Option<GlobalPos>,
        portal_cooldown: i32,
        sea_level: i32,
    ) -> CommonPlayerSpawnInfo {
        CommonPlayerSpawnInfo {
            dimension_type,
            dimension,
            seed,
            game_type,
            previous_game_type,
            is_debug,
            is_flat,
            last_death_location,
            portal_cooldown,
            sea_level,
        }
    }

    pub fn dimension_type(&self) -> Holder {
        self.dimension_type.clone()
    }

    pub fn dimension(&self) -> ResourceKey {
        self.dimension.clone()
    }

    pub fn seed(&self) -> i64 {
        self.seed
    }

    pub fn game_type(&self) -> GameType {
        self.game_type
    }

    pub fn previous_game_type(&self) -> Option<GameType> {
        self.previous_game_type
    }

    pub fn is_debug(&self) -> bool {
        self.is_debug
    }

    pub fn is_flat(&self) -> bool {
        self.is_flat
    }

    pub fn last_death_location(&self) -> Option<GlobalPos> {
        self.last_death_location.clone()
    }

    pub fn portal_cooldown(&self) -> i32 {
        self.portal_cooldown
    }

    pub fn sea_level(&self) -> i32 {
        self.sea_level
    }
}
