//! net/minecraft/network/protocol/game/ClientboundLoginPacket.java
//!
//! Not ported yet: the record's `hashCode`, which hashes the keys' identity hashes, and its
//! `toString`, which prints the spawn info's holder the way `Holder.Reference` does.
//!
//! Java reads the levels into a `HashSet` (`Sets.newHashSetWithExpectedSize`), which iterates in
//! the order of the keys' identity hashes, so differently every run; the listener shuffles them
//! anyway. Here they're a `LevelSet` of the keys in the order they came, each once.

use std::sync::LazyLock;

use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec, StreamCodec};
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::common_player_spawn_info::CommonPlayerSpawnInfo;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;
use crate::net::minecraft::resources::resource_key::ResourceKey;

static STREAM_CODEC: StaticStreamCodec<RegistryFriendlyByteBuf, ClientboundLoginPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite12(
            ByteBufCodecs::INT,
            ClientboundLoginPacket::player_id,
            ByteBufCodecs::BOOL,
            ClientboundLoginPacket::hardcore,
            ResourceKey::stream_codec(Registries::DIMENSION)
                .apply(ByteBufCodecs::collection_operation(|_| LevelSet::default())),
            ClientboundLoginPacket::levels,
            ByteBufCodecs::VAR_INT,
            ClientboundLoginPacket::max_players,
            ByteBufCodecs::VAR_INT,
            ClientboundLoginPacket::chunk_radius,
            ByteBufCodecs::VAR_INT,
            ClientboundLoginPacket::simulation_distance,
            ByteBufCodecs::BOOL,
            ClientboundLoginPacket::reduced_debug_info,
            ByteBufCodecs::BOOL,
            ClientboundLoginPacket::show_death_screen,
            ByteBufCodecs::BOOL,
            ClientboundLoginPacket::do_limited_crafting,
            CommonPlayerSpawnInfo::STREAM_CODEC,
            ClientboundLoginPacket::common_player_spawn_info,
            ByteBufCodecs::BOOL,
            ClientboundLoginPacket::online_mode,
            ByteBufCodecs::BOOL,
            ClientboundLoginPacket::enforces_secure_chat,
            ClientboundLoginPacket::new,
        ))
    });

/// The dimensions' keys a login lists, each once, in the order they came.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LevelSet(Vec<ResourceKey>);

impl LevelSet {
    pub fn keys(&self) -> &[ResourceKey] {
        &self.0
    }
}

impl Extend<ResourceKey> for LevelSet {
    fn extend<I: IntoIterator<Item = ResourceKey>>(&mut self, keys: I) {
        for key in keys {
            if !self.0.contains(&key) {
                self.0.push(key);
            }
        }
    }
}

impl<'a> IntoIterator for &'a LevelSet {
    type Item = &'a ResourceKey;
    type IntoIter = std::slice::Iter<'a, ResourceKey>;

    fn into_iter(self) -> std::slice::Iter<'a, ResourceKey> {
        self.0.iter()
    }
}

/// The first packet of the game protocol: the player's entity id, the server's dimensions and
/// settings, and the level the player spawns in.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundLoginPacket {
    player_id: i32,
    hardcore: bool,
    levels: LevelSet,
    max_players: i32,
    chunk_radius: i32,
    simulation_distance: i32,
    reduced_debug_info: bool,
    show_death_screen: bool,
    do_limited_crafting: bool,
    common_player_spawn_info: CommonPlayerSpawnInfo,
    online_mode: bool,
    enforces_secure_chat: bool,
}

impl ClientboundLoginPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        RegistryFriendlyByteBuf,
        ClientboundLoginPacket,
    > = &STREAM_CODEC;

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        player_id: i32,
        hardcore: bool,
        levels: LevelSet,
        max_players: i32,
        chunk_radius: i32,
        simulation_distance: i32,
        reduced_debug_info: bool,
        show_death_screen: bool,
        do_limited_crafting: bool,
        common_player_spawn_info: CommonPlayerSpawnInfo,
        online_mode: bool,
        enforces_secure_chat: bool,
    ) -> ClientboundLoginPacket {
        ClientboundLoginPacket {
            player_id,
            hardcore,
            levels,
            max_players,
            chunk_radius,
            simulation_distance,
            reduced_debug_info,
            show_death_screen,
            do_limited_crafting,
            common_player_spawn_info,
            online_mode,
            enforces_secure_chat,
        }
    }

    pub fn player_id(&self) -> i32 {
        self.player_id
    }

    pub fn hardcore(&self) -> bool {
        self.hardcore
    }

    pub fn levels(&self) -> LevelSet {
        self.levels.clone()
    }

    pub fn max_players(&self) -> i32 {
        self.max_players
    }

    pub fn chunk_radius(&self) -> i32 {
        self.chunk_radius
    }

    pub fn simulation_distance(&self) -> i32 {
        self.simulation_distance
    }

    pub fn reduced_debug_info(&self) -> bool {
        self.reduced_debug_info
    }

    pub fn show_death_screen(&self) -> bool {
        self.show_death_screen
    }

    pub fn do_limited_crafting(&self) -> bool {
        self.do_limited_crafting
    }

    pub fn common_player_spawn_info(&self) -> CommonPlayerSpawnInfo {
        self.common_player_spawn_info.clone()
    }

    pub fn online_mode(&self) -> bool {
        self.online_mode
    }

    pub fn enforces_secure_chat(&self) -> bool {
        self.enforces_secure_chat
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundLoginPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_LOGIN
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_login(self);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::ClientboundLoginPacket;
    use crate::io::netty::buffer::unpooled;
    use crate::net::minecraft::core::holder::Holder;
    use crate::net::minecraft::core::registries::registries::Registries;
    use crate::net::minecraft::core::registry::Registry;
    use crate::net::minecraft::core::registry_access::RegistryAccess;
    use crate::net::minecraft::network::codec::stream_codec::StreamCodec;
    use crate::net::minecraft::network::registry_friendly_byte_buf::RegistryFriendlyByteBuf;
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::resources::resource_key::ResourceKey;
    use crate::net::minecraft::world::level::game_type::GameType;

    /// The login vanilla's server sent in the walk scenario's capture
    /// (`vanilla/26.3/oracle/walk/capture.jsonl`), which the golden harness can't make yet: it
    /// reads a `RegistryFriendlyByteBuf`.
    const VANILLA_LOGIN: &str = "000000010003136d696e6563726166743a6f766572776f726c64146d696e6563726166743a7468655f6e6574686572116d696e6563726166743a7468655f656e6414040400010000136d696e6563726166743a6f766572776f726c647a0b81a1f57055af000000010000c1ffffff0f0000";

    /// The dimension types that capture's registry data sent, in its order.
    fn registries() -> Arc<RegistryAccess> {
        let keys = ["overworld", "overworld_caves", "the_end", "the_nether"].map(|name| {
            ResourceKey::create(
                Registries::DIMENSION_TYPE,
                Identifier::with_default_namespace(name),
            )
        });
        Arc::new(RegistryAccess::new(vec![Registry::new(
            (**Registries::DIMENSION_TYPE).clone(),
            keys.to_vec(),
        )]))
    }

    fn bytes(hex: &str) -> Vec<i8> {
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap() as i8)
            .collect()
    }

    #[test]
    fn reads_and_writes_vanilla_s_login() {
        let mut buffer = unpooled::buffer();
        buffer.write_bytes_array(&bytes(VANILLA_LOGIN));
        let mut input = RegistryFriendlyByteBuf::new(buffer, registries());
        let packet = ClientboundLoginPacket::STREAM_CODEC.decode(&mut input);
        assert_eq!(input.readable_bytes(), 0);

        assert_eq!(packet.player_id(), 1);
        assert_eq!(packet.levels().keys().len(), 3);
        assert_eq!(
            (
                packet.max_players(),
                packet.chunk_radius(),
                packet.simulation_distance()
            ),
            (20, 4, 4)
        );
        let spawn_info = packet.common_player_spawn_info();
        assert_eq!(
            spawn_info.dimension_type(),
            Holder::Reference(ResourceKey::create(
                Registries::DIMENSION_TYPE,
                Identifier::with_default_namespace("overworld")
            ))
        );
        assert_eq!(
            spawn_info.dimension().identifier().to_string(),
            "minecraft:overworld"
        );
        assert_eq!(spawn_info.game_type(), GameType::SURVIVAL);
        assert_eq!(spawn_info.previous_game_type(), None);
        assert!(spawn_info.is_flat() && !spawn_info.is_debug());
        assert_eq!(
            (spawn_info.portal_cooldown(), spawn_info.sea_level()),
            (0, -63)
        );

        let mut output = RegistryFriendlyByteBuf::new(unpooled::buffer(), registries());
        ClientboundLoginPacket::STREAM_CODEC.encode(&mut output, &packet);
        let mut written = vec![0; output.readable_bytes() as usize];
        output.read_bytes_array(&mut written);
        assert_eq!(written, bytes(VANILLA_LOGIN));
    }
}
