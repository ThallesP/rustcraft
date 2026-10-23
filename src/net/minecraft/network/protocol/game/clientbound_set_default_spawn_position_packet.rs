//! net/minecraft/network/protocol/game/ClientboundSetDefaultSpawnPositionPacket.java
//!
//! Its record's `hashCode` hashes the dimension's identity hash, which changes from run to run.

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::world::level::storage::level_data::RespawnData;
use std::fmt;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundSetDefaultSpawnPositionPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite1(
            RespawnData::STREAM_CODEC,
            ClientboundSetDefaultSpawnPositionPacket::respawn_data,
            ClientboundSetDefaultSpawnPositionPacket::new,
        ))
    });

/// Where the world's players spawn, which the compass points to.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundSetDefaultSpawnPositionPacket {
    respawn_data: RespawnData,
}

impl ClientboundSetDefaultSpawnPositionPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundSetDefaultSpawnPositionPacket,
    > = &STREAM_CODEC;

    pub fn new(respawn_data: RespawnData) -> ClientboundSetDefaultSpawnPositionPacket {
        ClientboundSetDefaultSpawnPositionPacket { respawn_data }
    }

    pub fn respawn_data(&self) -> RespawnData {
        self.respawn_data.clone()
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundSetDefaultSpawnPositionPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_SET_DEFAULT_SPAWN_POSITION
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_set_spawn(self);
    }
}

/// A record's: `ClientboundSetDefaultSpawnPositionPacket[respawnData=RespawnData[…]]`.
impl fmt::Display for ClientboundSetDefaultSpawnPositionPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClientboundSetDefaultSpawnPositionPacket[respawnData={}]",
            self.respawn_data
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundSetDefaultSpawnPositionPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::world::level::storage::level_data::RespawnData;

    encode_fields!(ClientboundSetDefaultSpawnPositionPacket {
        respawn_data: RespawnData,
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundSetDefaultSpawnPositionPacket;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load(
            "net.minecraft.network.protocol.game.ClientboundSetDefaultSpawnPositionPacket",
        );
        golden.pending("hashCode()I", "hashes the dimension's identity hash");
        golden.check_method("toString()Ljava/lang/String;", |this: &mut This| {
            this.to_string()
        });
        golden.check_codec("STREAM_CODEC", || {
            ClientboundSetDefaultSpawnPositionPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(Lnet/minecraft/world/level/storage/LevelData$RespawnData;)V",
            ClientboundSetDefaultSpawnPositionPacket::new,
        );
        golden.check_method(
            "respawnData()Lnet/minecraft/world/level/storage/LevelData$RespawnData;",
            |this: &mut This| this.respawn_data(),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut This, other: This| *this == other,
        );
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut This| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/game/ClientGamePacketListener;)V",
            |this: &mut This, listener: &mut Recorder| this.handle(listener),
        );
        golden.finish();
    }
}
