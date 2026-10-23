//! net/minecraft/network/protocol/game/ClientboundTeleportEntityPacket.java
//!
//! Not ported yet: the record's `hashCode`, which hashes the set of relatives, an `EnumSet` whose
//! `hashCode` is its constants' identity hashes added up.

use std::fmt;
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

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundTeleportEntityPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite4(
            ByteBufCodecs::VAR_INT,
            ClientboundTeleportEntityPacket::id,
            PositionMoveRotation::STREAM_CODEC,
            ClientboundTeleportEntityPacket::change,
            Relative::SET_STREAM_CODEC,
            ClientboundTeleportEntityPacket::relatives,
            ByteBufCodecs::BOOL,
            ClientboundTeleportEntityPacket::on_ground,
            ClientboundTeleportEntityPacket::new,
        ))
    });

/// The server putting another entity somewhere, its values absolute or relative to the entity's.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundTeleportEntityPacket {
    id: i32,
    change: PositionMoveRotation,
    relatives: Vec<Relative>,
    on_ground: bool,
}

impl ClientboundTeleportEntityPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundTeleportEntityPacket,
    > = &STREAM_CODEC;

    pub fn new(
        id: i32,
        change: PositionMoveRotation,
        relatives: Vec<Relative>,
        on_ground: bool,
    ) -> ClientboundTeleportEntityPacket {
        ClientboundTeleportEntityPacket {
            id,
            change,
            relatives,
            on_ground,
        }
    }

    pub fn teleport(
        id: i32,
        values: PositionMoveRotation,
        relatives: Vec<Relative>,
        on_ground: bool,
    ) -> ClientboundTeleportEntityPacket {
        ClientboundTeleportEntityPacket::new(id, values, relatives, on_ground)
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

    pub fn on_ground(&self) -> bool {
        self.on_ground
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundTeleportEntityPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_TELEPORT_ENTITY
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_teleport_entity(self);
    }
}

/// A record's, the set like an `EnumSet`'s: `ClientboundTeleportEntityPacket[id=1, change=…,
/// relatives=[X, Y], onGround=true]`.
impl fmt::Display for ClientboundTeleportEntityPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut relatives = self.relatives.clone();
        relatives.sort_by_key(|relative| *relative as u8);
        let relatives: Vec<String> = relatives
            .iter()
            .map(|relative| format!("{relative:?}"))
            .collect();
        write!(
            f,
            "ClientboundTeleportEntityPacket[id={}, change={}, relatives=[{}], onGround={}]",
            self.id,
            self.change,
            relatives.join(", "),
            self.on_ground
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundTeleportEntityPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;
    use crate::net::minecraft::world::entity::position_move_rotation::PositionMoveRotation;
    use crate::net::minecraft::world::entity::relative::Relative;

    encode_fields!(ClientboundTeleportEntityPacket {
        id: i32,
        change: PositionMoveRotation,
        relatives: Vec<Relative>,
        on_ground: bool,
    });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundTeleportEntityPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundTeleportEntityPacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundTeleportEntityPacket::STREAM_CODEC
        });
        golden.check(
            "<init>(ILnet/minecraft/world/entity/PositionMoveRotation;Ljava/util/Set;Z)V",
            ClientboundTeleportEntityPacket::new,
        );
        golden.check(
            "teleport(ILnet/minecraft/world/entity/PositionMoveRotation;Ljava/util/Set;Z)Lnet/minecraft/network/protocol/game/ClientboundTeleportEntityPacket;",
            ClientboundTeleportEntityPacket::teleport,
        );
        golden.check_method("id()I", |this: &mut This| this.id());
        golden.check_method(
            "change()Lnet/minecraft/world/entity/PositionMoveRotation;",
            |this: &mut This| this.change(),
        );
        golden.check_method("relatives()Ljava/util/Set;", |this: &mut This| {
            this.relatives()
        });
        golden.check_method("onGround()Z", |this: &mut This| this.on_ground());
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut This, other: This| *this == other,
        );
        golden.pending(
            "hashCode()I",
            "hashes its EnumSet, which adds its constants' identity hashes",
        );
        golden.check_method("toString()Ljava/lang/String;", |this: &mut This| {
            this.to_string()
        });
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
