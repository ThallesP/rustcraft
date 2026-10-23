//! net/minecraft/network/protocol/game/ClientboundSetHeldSlotPacket.java

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::Packet;
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, ClientboundSetHeldSlotPacket> =
    LazyLock::new(|| {
        Box::new(stream_codec::composite1(
            ByteBufCodecs::VAR_INT,
            ClientboundSetHeldSlotPacket::slot,
            ClientboundSetHeldSlotPacket::new,
        ))
    });

/// The hotbar slot the server puts the player's hand on.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundSetHeldSlotPacket {
    slot: i32,
}

impl ClientboundSetHeldSlotPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, ClientboundSetHeldSlotPacket> =
        &STREAM_CODEC;

    pub fn new(slot: i32) -> ClientboundSetHeldSlotPacket {
        ClientboundSetHeldSlotPacket { slot }
    }

    pub fn slot(&self) -> i32 {
        self.slot
    }

    /// A record's: its one component's hash.
    pub fn hash_code(&self) -> i32 {
        self.slot
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundSetHeldSlotPacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_SET_HELD_SLOT
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_set_held_slot(self);
    }
}

/// A record's: `ClientboundSetHeldSlotPacket[slot=4]`.
impl fmt::Display for ClientboundSetHeldSlotPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ClientboundSetHeldSlotPacket[slot={}]", self.slot)
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundSetHeldSlotPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    encode_fields!(ClientboundSetHeldSlotPacket { slot: i32 });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundSetHeldSlotPacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundSetHeldSlotPacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundSetHeldSlotPacket::STREAM_CODEC
        });
        golden.check("<init>(I)V", ClientboundSetHeldSlotPacket::new);
        golden.check_method("slot()I", |this: &mut This| this.slot());
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut This, other: This| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut This| this.hash_code());
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
