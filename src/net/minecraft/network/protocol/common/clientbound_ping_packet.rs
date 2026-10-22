//! net/minecraft/network/protocol/common/ClientboundPingPacket.java

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
use crate::net::minecraft::network::protocol::common::common_packet_types::CommonPacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundPingPacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundPingPacket::write,
            ClientboundPingPacket::from_friendly_byte_buf,
        ))
    });

/// A server's ping: the client answers with a `ServerboundPongPacket` of the same id.
#[derive(Clone, Debug)]
pub struct ClientboundPingPacket {
    id: i32,
}

impl ClientboundPingPacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<FriendlyByteBuf, ClientboundPingPacket> =
        &STREAM_CODEC;

    pub fn new(id: i32) -> ClientboundPingPacket {
        ClientboundPingPacket { id }
    }

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundPingPacket {
        ClientboundPingPacket::new(input.read_int())
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_int(self.id);
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }
}

impl<L: ClientCommonPacketListener + ?Sized> Packet<L> for ClientboundPingPacket {
    fn r#type(&self) -> &'static PacketType {
        CommonPacketTypes::CLIENTBOUND_PING
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_ping(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundPingPacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // The field its public constructor sets.
    encode_fields!(ClientboundPingPacket { id: i32 });

    type Listener = dyn ClientCommonPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.common.ClientboundPingPacket");
        golden.check_codec("STREAM_CODEC", || ClientboundPingPacket::STREAM_CODEC);
        golden.check("<init>(I)V", ClientboundPingPacket::new);
        golden.check_method("getId()I", |this: &mut ClientboundPingPacket| this.get_id());
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ClientboundPingPacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/common/ClientCommonPacketListener;)V",
            |this: &mut ClientboundPingPacket, listener: &mut Recorder| this.handle(listener),
        );
        golden.finish();
    }
}
