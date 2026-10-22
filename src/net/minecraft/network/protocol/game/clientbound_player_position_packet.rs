//! net/minecraft/network/protocol/game/ClientboundPlayerPositionPacket.java
//!
//! Not ported yet: the record's `toString`, which prints its floats with Java's `Float.toString`,
//! and its `hashCode`, which hashes the set of relatives, an `EnumSet` whose `hashCode` is its
//! constants' identity hashes added up.

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;
use crate::net::minecraft::world::entity::position_move_rotation::PositionMoveRotation;
use crate::net::minecraft::world::entity::relative::Relative;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundPlayerPositionPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite3(
            ByteBufCodecs::VAR_INT,
            ClientboundPlayerPositionPacket::id,
            PositionMoveRotation::STREAM_CODEC,
            ClientboundPlayerPositionPacket::change,
            Relative::SET_STREAM_CODEC,
            ClientboundPlayerPositionPacket::relatives,
            ClientboundPlayerPositionPacket::new,
        ))
    });

/// The server teleporting the player: the client moves it there, then answers with the
/// teleport's id (`accept_teleportation`).
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundPlayerPositionPacket {
    id: i32,
    change: PositionMoveRotation,
    relatives: Vec<Relative>,
}

impl ClientboundPlayerPositionPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundPlayerPositionPacket,
    > = &STREAM_CODEC;

    pub fn new(
        id: i32,
        change: PositionMoveRotation,
        relatives: Vec<Relative>,
    ) -> ClientboundPlayerPositionPacket {
        ClientboundPlayerPositionPacket {
            id,
            change,
            relatives,
        }
    }

    pub fn of(
        id: i32,
        values: PositionMoveRotation,
        relatives: Vec<Relative>,
    ) -> ClientboundPlayerPositionPacket {
        ClientboundPlayerPositionPacket::new(id, values, relatives)
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn change(&self) -> PositionMoveRotation {
        self.change
    }

    pub fn relatives(&self) -> Vec<Relative> {
        self.relatives.clone()
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundPlayerPositionPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_PLAYER_POSITION
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_move_player(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundPlayerPositionPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::world::entity::position_move_rotation::PositionMoveRotation;
    use crate::net::minecraft::world::entity::relative::Relative;

    encode_fields!(ClientboundPlayerPositionPacket {
        id: i32,
        change: PositionMoveRotation,
        relatives: Vec<Relative>,
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundPlayerPositionPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundPlayerPositionPacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundPlayerPositionPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(ILnet/minecraft/world/entity/PositionMoveRotation;Ljava/util/Set;)V",
            ClientboundPlayerPositionPacket::new,
        );
        golden.check(
            "of(ILnet/minecraft/world/entity/PositionMoveRotation;Ljava/util/Set;)Lnet/minecraft/network/protocol/game/ClientboundPlayerPositionPacket;",
            ClientboundPlayerPositionPacket::of,
        );
        golden.check_method("id()I", |this: &mut This| this.id());
        golden.check_method(
            "change()Lnet/minecraft/world/entity/PositionMoveRotation;",
            |this: &mut This| this.change(),
        );
        golden.check_method("relatives()Ljava/util/Set;", |this: &mut This| {
            this.relatives()
        });
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut This, other: This| *this == other,
        );
        golden.pending(
            "hashCode()I",
            "hashes its EnumSet, which adds its constants' identity hashes",
        );
        golden.pending(
            "toString()Ljava/lang/String;",
            "waits for Java's Float.toString",
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
