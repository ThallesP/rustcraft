//! net/minecraft/network/protocol/common/ClientboundKeepAlivePacket.java

use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::StaticStreamCodec;
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
use crate::net::minecraft::network::protocol::common::common_packet_types::CommonPacketTypes;
use crate::net::minecraft::network::protocol::packet::{self, Packet};
use crate::net::minecraft::network::protocol::packet_type::PacketType;

static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, ClientboundKeepAlivePacket> =
    LazyLock::new(|| {
        Box::new(packet::codec(
            ClientboundKeepAlivePacket::write,
            ClientboundKeepAlivePacket::from_friendly_byte_buf,
        ))
    });

/// A server checking the connection is alive: the client answers with a `ServerboundKeepAlivePacket`
/// of the same id.
#[derive(Clone, Debug)]
pub struct ClientboundKeepAlivePacket {
    id: i64,
}

impl ClientboundKeepAlivePacket {
    pub const STREAM_CODEC: &'static StaticStreamCodec<
        FriendlyByteBuf,
        ClientboundKeepAlivePacket,
    > = &STREAM_CODEC;

    pub fn new(id: i64) -> ClientboundKeepAlivePacket {
        ClientboundKeepAlivePacket { id }
    }

    fn from_friendly_byte_buf(input: &mut FriendlyByteBuf) -> ClientboundKeepAlivePacket {
        ClientboundKeepAlivePacket::new(input.read_long())
    }

    fn write(&self, output: &mut FriendlyByteBuf) {
        output.write_long(self.id);
    }

    pub fn get_id(&self) -> i64 {
        self.id
    }
}

impl<L: ClientCommonPacketListener + ?Sized> Packet<L> for ClientboundKeepAlivePacket {
    fn r#type(&self) -> &'static PacketType {
        CommonPacketTypes::CLIENTBOUND_KEEP_ALIVE
    }

    fn handle(&self, listener: &mut L) {
        listener.handle_keep_alive(self);
    }
}

#[cfg(test)]
mod tests {
    use super::ClientboundKeepAlivePacket;
    use crate::golden::{Golden, encode_fields};
    use crate::net::minecraft::network::packet_listener::tests::Recorder;
    use crate::net::minecraft::network::protocol::common::client_common_packet_listener::ClientCommonPacketListener;
    use crate::net::minecraft::network::protocol::packet::Packet;

    // The field its public constructor sets.
    encode_fields!(ClientboundKeepAlivePacket { id: i64 });

    type Listener = dyn ClientCommonPacketListener;

    #[test]
    fn matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.network.protocol.common.ClientboundKeepAlivePacket");
        golden.check_codec("STREAM_CODEC", || ClientboundKeepAlivePacket::STREAM_CODEC);
        golden.check("<init>(J)V", ClientboundKeepAlivePacket::new);
        golden.check_method("getId()J", |this: &mut ClientboundKeepAlivePacket| {
            this.get_id()
        });
        golden.check_method(
            "type()Lnet/minecraft/network/protocol/PacketType;",
            |this: &mut ClientboundKeepAlivePacket| Packet::<Listener>::r#type(this).clone(),
        );
        golden.check_mut(
            "handle(Lnet/minecraft/network/protocol/common/ClientCommonPacketListener;)V",
            |this: &mut ClientboundKeepAlivePacket, listener: &mut Recorder| this.handle(listener),
        );
        golden.finish();
    }
}
