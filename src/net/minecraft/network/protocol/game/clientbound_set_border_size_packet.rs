//! net/minecraft/network/protocol/game/ClientboundSetBorderSizePacket.java
//!
//! Not ported yet: its public constructor, which reads a server's `WorldBorder` (`getLerpTarget`);
//! the client only reads the packet.

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
use crate::net::minecraft::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundSetBorderSizePacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundSetBorderSizePacket::write,
            ClientboundSetBorderSizePacket::from_friendly_byte_buf,
        ))
    });

/// The world border's new size, at once.
#[derive(Clone, Debug)]
pub struct ClientboundSetBorderSizePacket {
    size: f64,
}

impl ClientboundSetBorderSizePacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundSetBorderSizePacket,
    > = &STREAM_CODEC;

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundSetBorderSizePacket {
        let size = input.read_double();
        ClientboundSetBorderSizePacket { size }
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_double(self.size);
    }

    pub fn get_size(&self) -> f64 {
        self.size
    }
}

impl<L: ClientGamePacketListener + ?Sized> Packet<L> for ClientboundSetBorderSizePacket {
    fn r#type(&self) -> &'static PacketType {
        GamePacketTypes::CLIENTBOUND_SET_BORDER_SIZE
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_set_border_size(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundSetBorderSizePacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::game::client_game_packet_listener::ClientGamePacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // The fields it reads.
    encode_fields!(ClientboundSetBorderSizePacket { size: f64 });

    type Listener = dyn ClientGamePacketListener;
    type This = ClientboundSetBorderSizePacket;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.game.ClientboundSetBorderSizePacket");
        golden.check_codec("STREAM_CODEC", || {
            ClientboundSetBorderSizePacket::STREAM_CODEC
        });
        golden.pending(
            "<init>(Lnet/minecraft/world/level/border/WorldBorder;)V",
            "reads a server's WorldBorder; the client only reads the packet",
        );
        golden.check_method("getSize()D", |this: &mut This| this.get_size());
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
